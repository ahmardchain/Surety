// Shared helpers for Surety testnet scripts. Keys are read from $KEYS_DIR (never committed).
const fs = require("fs");
const path = require("path");
const { connect, keyStores, KeyPair, utils } = require("near-api-js");

const NETWORK = "testnet";
const RPC = process.env.NEAR_RPC || "https://rpc.testnet.near.org";
const KEYS_DIR = process.env.KEYS_DIR || path.join(__dirname, ".keys");
const STATE = path.join(__dirname, "..", "demo", "state.json");
const TXLOG = path.join(__dirname, "..", "demo", "txs.json");
const EXPLORER = "https://testnet.nearblocks.io/txns/";
const NEAR = (n) => utils.format.parseNearAmount(String(n));
const fmt = (y) => utils.format.formatNearAmount(String(y), 4);

function loadState() {
  return fs.existsSync(STATE) ? JSON.parse(fs.readFileSync(STATE, "utf8")) : {};
}
function saveState(s) {
  fs.mkdirSync(path.dirname(STATE), { recursive: true });
  fs.writeFileSync(STATE, JSON.stringify(s, null, 2) + "\n");
}

function keyStore() {
  const ks = new keyStores.InMemoryKeyStore();
  if (fs.existsSync(KEYS_DIR)) {
    for (const f of fs.readdirSync(KEYS_DIR)) {
      if (!f.endsWith(".json")) continue;
      const k = JSON.parse(fs.readFileSync(path.join(KEYS_DIR, f), "utf8"));
      ks.setKey(NETWORK, k.account_id, KeyPair.fromString(k.private_key));
    }
  }
  return ks;
}

async function near() {
  return connect({ networkId: NETWORK, nodeUrl: RPC, keyStore: keyStore() });
}

async function account(id) {
  return (await near()).account(id);
}

async function view(contractId, method, args = {}) {
  const a = await account(contractId);
  return a.viewFunction({ contractId, methodName: method, args });
}

async function balance(id) {
  const a = await account(id);
  return (await a.getAccountBalance()).total;
}

/** Create a top-level testnet named account funded by the faucet helper. */
async function faucetAccount(name, role) {
  fs.mkdirSync(KEYS_DIR, { recursive: true });
  const kp = KeyPair.fromRandom("ed25519");
  const r = await fetch("https://helper.testnet.near.org/account", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ newAccountId: name, newAccountPublicKey: kp.getPublicKey().toString() }),
  });
  if (!r.ok) throw new Error(`faucet ${name}: ${r.status} ${await r.text()}`);
  fs.writeFileSync(path.join(KEYS_DIR, `${role}.json`), JSON.stringify({ account_id: name, private_key: kp.toString() }));
  return name;
}

/** Record a step with its tx hash in demo/txs.json and print it. */
function logTx(step, outcome, extra = {}) {
  const hash = outcome?.transaction_outcome?.id || outcome?.transaction?.hash;
  const log = fs.existsSync(TXLOG) ? JSON.parse(fs.readFileSync(TXLOG, "utf8")) : [];
  const entry = { step, tx: hash, url: hash ? EXPLORER + hash : null, ...extra };
  log.push(entry);
  fs.mkdirSync(path.dirname(TXLOG), { recursive: true });
  fs.writeFileSync(TXLOG, JSON.stringify(log, null, 2) + "\n");
  console.log(`✓ ${step}\n  ${entry.url || ""}${Object.keys(extra).length ? "\n  " + JSON.stringify(extra) : ""}`);
  return entry;
}

/** Function call; throws with the contract's panic message if it failed. */
async function call(signerId, contractId, methodName, args = {}, deposit = "0", gas = "300000000000000") {
  const a = await account(signerId);
  const out = await a.functionCall({ contractId, methodName, args, attachedDeposit: BigInt(deposit), gas: BigInt(gas) });
  const fail = [out.status, ...out.receipts_outcome.map((r) => r.outcome.status)].find((s) => s && s.Failure);
  if (fail) throw new Error(`${methodName} failed: ${JSON.stringify(fail.Failure)}`);
  return out;
}

function returnValue(out) {
  const v = out.status?.SuccessValue;
  return v ? JSON.parse(Buffer.from(v, "base64").toString()) : null;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

module.exports = { NEAR, fmt, near, account, view, balance, call, returnValue, faucetAccount, logTx, loadState, saveState, sleep, KEYS_DIR };
