// End-to-end testnet demo on ONE Sputnik DAO with TWO loans:
//   Loan A (ada): guaranteed by bola + chidi -> council approves in Sputnik -> ada repays -> limits rise.
//   Loan B (dayo): guaranteed by bola + chidi -> council approves -> dayo pays only part -> past due -> anyone calls claim
//                  -> guarantors' locked savings move to the DAO treasury in one call.
// Every step is written with its tx hash to demo/txs.json.
// Usage: KEYS_DIR=/path/outside/repo node demo.js   (after setup.js)
const { NEAR, fmt, view, call, returnValue, logTx, loadState, saveState, sleep, balance } = require("./lib");

const s = loadState();
const A = s.accounts;
const S = A.surety;
const DAO = s.dao;
const LOAN_B_SECONDS = Number(process.env.LOAN_B_SECONDS || 90);

const limit = async (who) => fmt((await view(S, "get_limit", { account_id: who })));

async function approve(pid) {
  const p = await view(DAO, "get_proposal", { id: pid });
  let out;
  try {
    out = await call(A.council, DAO, "act_proposal", { id: pid, action: "VoteApprove", proposal: p.kind });
  } catch (e) {
    // older Sputnik builds take no `proposal` argument
    out = await call(A.council, DAO, "act_proposal", { id: pid, action: "VoteApprove" });
  }
  return out;
}

async function runLoan(tag, borrower, amount, seconds, purpose) {
  const half = (BigInt(NEAR(amount)) / 2n).toString();
  let out = await call(borrower, S, "request_loan", { amount: NEAR(amount), duration_sec: String(seconds), purpose });
  const id = returnValue(out);
  logTx(`${tag}: ${borrower} requests loan #${id} of ${amount} NEAR, ${seconds}s term`, out, { loan_id: id, limit_before: await limit(borrower) });
  for (const g of [A.bola, A.chidi]) {
    out = await call(g, S, "pledge", { loan_id: id, amount: half });
    logTx(`${tag}: ${g} pledges ${fmt(half)} NEAR of savings behind loan #${id}`, out);
  }
  out = await call(borrower, S, "file_proposal", { loan_id: id }, NEAR("0.1"));
  const pid = (await view(S, "get_loan", { loan_id: id })).proposal_id;
  logTx(`${tag}: surety files Sputnik Transfer proposal #${pid} on ${DAO} (bond 0.1 NEAR)`, out, { proposal_id: pid });
  const daoBefore = await balance(DAO);
  out = await approve(pid);
  logTx(`${tag}: council approves proposal #${pid} once in Sputnik -> DAO pays ${amount} NEAR to ${borrower}`, out, {
    dao_balance_before: fmt(daoBefore),
    dao_balance_after: fmt(await balance(DAO)),
  });
  out = await call(A.ada, S, "sync", { loan_id: id });
  const loan = await view(S, "get_loan", { loan_id: id });
  logTx(`${tag}: sync reads proposal status -> loan #${id} ${loan.status}`, out, { status: loan.status, due_at_ns: loan.due_at });
  return id;
}

async function main() {
  s.demo = s.demo || {};
  if (!s.demo.deposits) {
    for (const [who, amt] of [[A.ada, "0.5"], [A.dayo, "0.5"], [A.bola, "1.5"], [A.chidi, "1.5"]]) {
      const out = await call(who, S, "deposit", {}, NEAR(amt));
      logTx(`${who} deposits ${amt} NEAR savings`, out);
    }
    s.demo.deposits = true;
    saveState(s);
  }

  if (s.demo.loanA === undefined) {
    const id = await runLoan("Loan A", A.ada, "1", 3600, "market stall restock");
    const before = await limit(A.ada);
    const out = await call(A.ada, S, "repay", { loan_id: id }, NEAR("1"));
    logTx(`Loan A: ada repays 1 NEAR -> forwarded to DAO treasury; pledges released`, out, {
      status: (await view(S, "get_loan", { loan_id: id })).status,
      ada_limit_before: before,
      ada_limit_after: await limit(A.ada),
      bola_limit_after: await limit(A.bola),
      chidi_limit_after: await limit(A.chidi),
    });
    s.demo.loanA = id;
    saveState(s);
  }

  if (s.demo.loanB === undefined) {
    const id = await runLoan("Loan B", A.dayo, "1", LOAN_B_SECONDS, "school fees");
    const out = await call(A.dayo, S, "repay", { loan_id: id }, NEAR("0.2"));
    logTx(`Loan B: dayo repays only 0.2 NEAR, then stops`, out, { outstanding: "0.8 NEAR" });
    s.demo.loanB = id;
    saveState(s);
  }

  if (!s.demo.claimed) {
    const loan = await view(S, "get_loan", { loan_id: s.demo.loanB });
    const waitMs = Number(BigInt(loan.due_at) / 1000000n) - Date.now() + 5000;
    if (waitMs > 0) {
      console.log(`waiting ${Math.ceil(waitMs / 1000)}s for loan #${loan.id} to pass due date...`);
      await sleep(waitMs);
    }
    const daoBefore = await balance(DAO);
    const g = async () => Object.fromEntries(await Promise.all([A.bola, A.chidi].map(async (x) => [x, (await view(S, "get_member", { account_id: x })).locked])));
    const lockedBefore = await g();
    const out = await call(A.council, S, "claim", { loan_id: s.demo.loanB });
    await sleep(3000);
    logTx(`Loan B: past due -> claim moves guarantors' locked savings (0.8 NEAR) to DAO treasury in one call`, out, {
      status: (await view(S, "get_loan", { loan_id: s.demo.loanB })).status,
      dao_balance_before: fmt(daoBefore),
      dao_balance_after: fmt(await balance(DAO)),
      guarantor_locked_before: Object.fromEntries(Object.entries(lockedBefore).map(([k, v]) => [k, fmt(v)])),
      dayo_limit_after: await limit(A.dayo),
    });
    s.demo.claimed = true;
    saveState(s);
  }

  console.log(JSON.stringify(await view(S, "get_members"), null, 1));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
