// Loan C: council REMOVES the Sputnik proposal (Sputnik deletes it). sync must still
// release the guarantors' pledges. Also records the in-place contract upgrade.
// Usage: KEYS_DIR=/path/outside/repo node demo-removed.js
const fs = require("fs");
const path = require("path");
const { NEAR, fmt, view, call, account, returnValue, logTx, loadState, saveState } = require("./lib");
const WASM = path.join(__dirname, "..", "contract", "target", "wasm32-unknown-unknown", "release", "surety.wasm");

async function main() {
  const s = loadState(); const A = s.accounts; const S = A.surety; const DAO = s.dao;
  s.demo = s.demo || {};
  if (!s.demo.upgradedRemovedFix) {
    const out = await (await account(S)).deployContract(fs.readFileSync(WASM));
    logTx("Upgrade surety code in place: a removed (deleted) Sputnik proposal now cancels the loan and releases pledges", out, { contract: S });
    s.demo.upgradedRemovedFix = true; saveState(s);
  }
  if (s.demo.loanC === undefined) {
    const g = async () => Object.fromEntries(await Promise.all([A["guarantor-1"], A["guarantor-2"]].map(async (x) => [x, fmt((await view(S, "get_member", { account_id: x })).locked)])));
    let out = await call(A["borrower-a"], S, "request_loan", { amount: NEAR("1"), duration_sec: "3600", purpose: "tools" });
    const id = returnValue(out);
    logTx(`Loan C: ${A["borrower-a"]} requests loan #${id} of 1 NEAR (uses the 1.5 NEAR limit earned in Loan A)`, out, { loan_id: id });
    for (const who of [A["guarantor-1"], A["guarantor-2"]]) {
      out = await call(who, S, "pledge", { loan_id: id, amount: NEAR("0.5") });
      logTx(`Loan C: ${who} pledges 0.5 NEAR behind loan #${id}`, out);
    }
    out = await call(A["borrower-a"], S, "file_proposal", { loan_id: id }, NEAR("0.1"));
    const pid = (await view(S, "get_loan", { loan_id: id })).proposal_id;
    logTx(`Loan C: surety files Sputnik Transfer proposal #${pid}`, out, { proposal_id: pid, guarantor_locked: await g() });
    const p = await view(DAO, "get_proposal", { id: pid });
    out = await call(A.council, DAO, "act_proposal", { id: pid, action: "VoteRemove", proposal: p.kind });
    logTx(`Loan C: council votes Remove on proposal #${pid}; Sputnik deletes it, no payout`, out, { proposal_id: pid });
    // the DAO's get_proposal receipt fails by design here (proposal deleted); the sync tx itself succeeds
    out = await call(A["guarantor-1"], S, "sync", { loan_id: id }, "0", undefined, { allowInnerFailure: true });
    const loan = await view(S, "get_loan", { loan_id: id });
    logTx(`Loan C: sync finds the proposal gone -> loan #${id} ${loan.status}, guarantors' pledges released`, out, { status: loan.status, sync_returned: returnValue(out), guarantor_locked_after: await g() });
    s.demo.loanC = id; saveState(s);
  }
}
main().catch((e) => { console.error(e); process.exit(1); });
