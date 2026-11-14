// One-time testnet setup:
//  1. faucet-funded named accounts: council, ada, bola, chidi, dayo (+ surety contract account)
//  2. Sputnik DAO v2 via the official factory sputnik-v2.testnet (council = council account, bond 0.1 NEAR)
//  3. deploy + init the surety contract against that DAO
//  4. council tops up the DAO treasury
// Usage: KEYS_DIR=/path/outside/repo SUFFIX=abc node setup.js
const fs = require("fs");
const path = require("path");
const { NEAR, account, view, call, faucetAccount, logTx, loadState, saveState, sleep, KEYS_DIR, balance, fmt } = require("./lib");

const FACTORY = "sputnik-v2.testnet";
const WASM = path.join(__dirname, "..", "contract", "target", "wasm32-unknown-unknown", "release", "surety.wasm");

async function main() {
  const s = loadState();
  const sfx = process.env.SUFFIX || s.suffix || Date.now().toString(36);
  s.suffix = sfx;
  s.accounts = s.accounts || {};

  // 1. accounts
  for (const role of ["surety", "council", "ada", "bola", "chidi", "dayo"]) {
    if (s.accounts[role]) continue;
    if (fs.existsSync(path.join(KEYS_DIR, `${role}.json`))) {
      s.accounts[role] = JSON.parse(fs.readFileSync(path.join(KEYS_DIR, `${role}.json`))).account_id;
    } else {
      s.accounts[role] = await faucetAccount(`${role}-coop-${sfx}.testnet`, role);
      await sleep(1500);
    }
    console.log(role, s.accounts[role]);
    saveState(s);
  }
  const A = s.accounts;

  // 2. DAO
  if (!s.dao) {
    const name = `coop-${sfx}`;
    const policy = {
      roles: [
        { name: "all", kind: "Everyone", permissions: ["*:AddProposal"], vote_policy: {} },
        {
          name: "council",
          kind: { Group: [A.council] },
          permissions: ["*:AddProposal", "*:VoteApprove", "*:VoteReject", "*:VoteRemove", "*:Finalize"],
          vote_policy: {},
        },
      ],
      default_vote_policy: { weight_kind: "RoleWeight", quorum: "0", threshold: [1, 2] },
      proposal_bond: NEAR("0.1"),
      proposal_period: "604800000000000",
      bounty_bond: NEAR("0.1"),
      bounty_forgiveness_period: "86400000000000",
    };
    const args = {
      config: { name, purpose: "Savings group treasury. Member-guaranteed loans via Surety.", metadata: "" },
      policy,
    };
    const out = await call(A.council, FACTORY, "create", { name, args: Buffer.from(JSON.stringify(args)).toString("base64") }, NEAR("6"));
    s.dao = `${name}.${FACTORY}`;
    saveState(s);
    logTx("Create Sputnik DAO via sputnik-v2.testnet factory", out, { dao: s.dao, council: A.council, proposal_bond: "0.1 NEAR" });
  }

  // 3. surety contract
  if (!s.deployed) {
    const acc = await account(A.surety);
    const out = await acc.deployContract(fs.readFileSync(WASM));
    logTx("Deploy surety contract", out, { contract: A.surety });
    const init = await call(A.surety, A.surety, "new", {
      dao: s.dao,
      base_limit: NEAR("1"),
      limit_step: NEAR("0.5"),
      min_guarantors: 2,
    });
    logTx("Init surety (dao, base limit 1 NEAR, +0.5 NEAR per repaid loan/guarantee, 2 guarantors)", init, { dao: s.dao });
    s.deployed = true;
    saveState(s);
  }

  // 4. treasury
  if (!s.funded) {
    const c = await account(A.council);
    const out = await c.sendMoney(s.dao, BigInt(NEAR("2")));
    logTx("Council tops up DAO treasury with 2 NEAR", out, { dao: s.dao });
    s.funded = true;
    saveState(s);
  }

  console.log("DAO policy bond:", fmt((await view(s.dao, "get_policy")).proposal_bond), "NEAR");
  console.log("DAO balance:", fmt(await balance(s.dao)), "NEAR");
  console.log(JSON.stringify(await view(A.surety, "get_config"), null, 2));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
