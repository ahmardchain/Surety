# Surety

A community that saves together should be able to lend to its own members, with members, not collateral, standing behind each loan.

Surety is a small NEAR contract that sits next to an existing **Sputnik DAO v2**. Custody and approvals stay in the DAO (visible in Trezu). Surety adds the loan logic: savings, guarantor pledges, filing the payout as a normal Sputnik `Transfer` proposal, routing repayments back to the treasury, and moving guarantors' locked savings to the treasury when a loan defaults. Each member's borrowing limit is derived from their real repayment record.

**Live board:** <https://surety.ahmardchain.workers.dev/> (reads the testnet contract and DAO live).

**Demo video (73 s, captions + voiceover):** [`demo/surety-demo.mp4`](demo/surety-demo.mp4). Voice: Microsoft Edge neural TTS (en-US-AndrewNeural); screen: Playwright recording of the live board and of the two key transactions read from NEAR RPC; built by `scripts/video/`.

Track: **Social Money** (shared budgets, reputation-based access on top of Sputnik DAO and Trezu, named accounts). Network: **NEAR testnet**.

## Problem

In a Sputnik DAO, paying a member is a one-way `Transfer` proposal. Nothing on-chain links that payout to a repayment, nothing lets other members vouch for the recipient, and if the money does not come back the group's only tools are chasing the person and holding another vote. So savings groups that run on a DAO can pool money but cannot safely lend it to their own members.

## How it works

1. A member calls `deposit` to put savings into Surety.
2. A member calls `request_loan(amount, duration_sec)`. The amount must be within their limit.
3. Two **named-account** guarantors call `pledge`, which locks part of their savings behind the loan. Pledges must cover 100% of it. The borrower cannot guarantee their own loan.
4. Once the loan is fully covered, `file_proposal` makes Surety call the DAO's `add_proposal` with a plain `Transfer` (treasury → borrower) and attaches the DAO's proposal bond. The loan id is in the description: `ref surety:<contract>:loan:<id>`.
5. The council approves once, in Sputnik or Trezu, and the DAO pays the borrower. Surety never holds or moves treasury funds.
6. `sync` reads the proposal status. On `Approved` the loan becomes **Active** and its due date is set. On `Rejected`/`Expired`/`Removed` the loan is cancelled and the pledges are unlocked. The bond goes back to whoever filed, except on `Removed`.
7. `repay` forwards the payment straight to the DAO treasury. Full repayment unlocks the pledges and raises the borrower's and guarantors' limits.
8. After the due date, **anyone** can call `claim`. It moves the outstanding balance from the guarantors' locked savings to the DAO treasury in one transaction, split pro rata to their pledges. The borrower's limit drops to 0. No chasing, no bad-debt vote.

Limit rule: `limit = 0` after any default, otherwise `base_limit + limit_step × (loans_repaid + guarantees_released)`. The deployment below uses base 1 NEAR and step 0.5 NEAR.

## Live testnet deployment

| What | Account |
|---|---|
| Sputnik DAO (made with the official `sputnik-v2.testnet` factory) | [`coop-mwfpr6a5.sputnik-v2.testnet`](https://testnet.nearblocks.io/address/coop-mwfpr6a5.sputnik-v2.testnet) |
| Surety contract | [`surety-mwfpr6a5.testnet`](https://testnet.nearblocks.io/address/surety-mwfpr6a5.testnet) |
| Council (only DAO voter) | `council-mwfpr6a5.testnet` |
| Borrower, Loan A (repays) | `borrower-a-mwfpr6a5.testnet` |
| Borrower, Loan B (defaults) | `borrower-b-mwfpr6a5.testnet` |
| Guarantors on both loans | `guarantor-1-mwfpr6a5.testnet`, `guarantor-2-mwfpr6a5.testnet` |

DAO policy: `Everyone` may add proposals, the council group votes, the threshold is 1/2 and the proposal bond is 0.1 NEAR.

## Try it: do X → see Y

### 1. See the live board
Open <https://surety.ahmardchain.workers.dev/>, or run it locally:
```bash
git clone https://github.com/ahmardchain/surety && cd surety
```
```bash
python3 -m http.server 8765
```
Open <http://127.0.0.1:8765/web/> (dev version, loads Tailwind/Lucide from CDN; `node scripts/build-worker.js` builds the compiled `web/dist/` version that is deployed). **See:**
- **Treasury:** 8 NEAR in the DAO; 2 NEAR paid out through Sputnik; 2 NEAR returned to the DAO.
- **Loan #0 (borrower-a): Repaid.** 1 NEAR came back from the borrower.
- **Loan #1 (borrower-b): Defaulted.** 0.2 NEAR was repaid by borrower-b and 0.8 NEAR claimed from the guarantors.
- **Members:** borrower-a's limit is 1.5 NEAR, up from 1. guarantor-1 and guarantor-2 are at 1.5 NEAR each, with 0.4 NEAR each lost to the claim. borrower-b's limit is 0.

The page reads the chain directly (`?contract=<id>` points it at another deployment).

### 2. Check the recorded run on-chain (same DAO, two loans)
Every step below was sent by `scripts/setup.js` and `scripts/demo.js`. The raw log, with balances before and after, is in [`demo/txs.json`](demo/txs.json).

| # | Step | Tx |
|---|---|---|
| 1 | Create Sputnik DAO via sputnik-v2.testnet factory | [6k6R2z23Rb…](https://testnet.nearblocks.io/txns/6k6R2z23RbjewibWoZg9ad8GZrUskTHmH8pFGUUJU3CU) |
| 2 | Deploy surety contract | [DGGYfXXqgF…](https://testnet.nearblocks.io/txns/DGGYfXXqgFVRmWDYr4JWmrT344SjYhM78TbgDcziqidw) |
| 3 | Init surety (dao, base limit 1 NEAR, +0.5 NEAR per repaid loan/guarantee, 2 guarantors) | [8bVrYGHWGH…](https://testnet.nearblocks.io/txns/8bVrYGHWGHd8X1pMvjdrALRvu26KVUYoDJgePxfWwtqc) |
| 4 | Council tops up DAO treasury with 2 NEAR | [Cjy8shichH…](https://testnet.nearblocks.io/txns/Cjy8shichHAxpeu48ZHARVaDTguL7FMGsyRUshvuyGns) |
| 5 | borrower-a-mwfpr6a5.testnet deposits 0.5 NEAR savings | [sHghM5vtFX…](https://testnet.nearblocks.io/txns/sHghM5vtFXFm3yRCH6GKxjXhPNc1eUucBknBAd2vesH) |
| 6 | borrower-b-mwfpr6a5.testnet deposits 0.5 NEAR savings | [52XsYRu24e…](https://testnet.nearblocks.io/txns/52XsYRu24eyYja3MZmUq5i899AKyYqiWqPm6tSzxcktC) |
| 7 | guarantor-1-mwfpr6a5.testnet deposits 1.5 NEAR savings | [9phGHZSRMM…](https://testnet.nearblocks.io/txns/9phGHZSRMMunqLrraZEWTmNpfQfmiLpGdZe28Uy7z252) |
| 8 | guarantor-2-mwfpr6a5.testnet deposits 1.5 NEAR savings | [4VJagrmht3…](https://testnet.nearblocks.io/txns/4VJagrmht3nqxhUbpPf2jF9dBj8VZ3iednEYUSjA5iHr) |
| 9 | Loan A: borrower-a-mwfpr6a5.testnet requests loan #0 of 1 NEAR, 3600s term | [8LdtxUkP85…](https://testnet.nearblocks.io/txns/8LdtxUkP85aD1mrfyceTX5sH8VEYLhxLCAN2uTnBJgh7) |
| 10 | Loan A: guarantor-1-mwfpr6a5.testnet pledges 0.5 NEAR of savings behind loan #0 | [C2NP4Yd3rP…](https://testnet.nearblocks.io/txns/C2NP4Yd3rPDWNS5XKDBzv55grHUYzfJ6eCTULhwMXWpf) |
| 11 | Loan A: guarantor-2-mwfpr6a5.testnet pledges 0.5 NEAR of savings behind loan #0 | [Bsrh2Ct7zi…](https://testnet.nearblocks.io/txns/Bsrh2Ct7zipVxhyutWGhzehnoEYMQRoxJGLSdNNt2uwg) |
| 12 | Loan A: surety files Sputnik Transfer proposal #0 on coop-mwfpr6a5.sputnik-v2.testnet (bond 0.1 NEAR) | [FEVy9gMf5w…](https://testnet.nearblocks.io/txns/FEVy9gMf5wbRTZhFk2XjBAxQf3SDPyEFzGq6ZzAcVBax) |
| 13 | Loan A: council approves proposal #0 once in Sputnik -> DAO pays 1 NEAR to borrower-a-mwfpr6a5.testnet | [26hYDRXjTr…](https://testnet.nearblocks.io/txns/26hYDRXjTrdxSGakPsv75Kx4rzhhr99nwtVHHKEh5qj6) |
| 14 | Loan A: sync reads proposal status -> loan #0 Active | [Aq8qUjdaMH…](https://testnet.nearblocks.io/txns/Aq8qUjdaMH5SXvAuA6XWMtDu1tMGvXG9UbQNzhMR8BDA) |
| 15 | Loan A: borrower-a-mwfpr6a5.testnet repays 1 NEAR -> forwarded to DAO treasury; pledges released | [81JeQZDs6E…](https://testnet.nearblocks.io/txns/81JeQZDs6Ep6pt2fuzeXc5JSBkW9d3JS938nLwsQMLUJ) |
| 16 | Loan B: borrower-b-mwfpr6a5.testnet requests loan #1 of 1 NEAR, 90s term | [GXVmcVwpHe…](https://testnet.nearblocks.io/txns/GXVmcVwpHe88TYfrum6hN9LJZ9HBSvab9RwdRfRnvwKZ) |
| 17 | Loan B: guarantor-1-mwfpr6a5.testnet pledges 0.5 NEAR of savings behind loan #1 | [FUT4aqxFJv…](https://testnet.nearblocks.io/txns/FUT4aqxFJv4WDiUR5ECo2RZ4ZcnyYR6jPJetNB7jZij) |
| 18 | Loan B: guarantor-2-mwfpr6a5.testnet pledges 0.5 NEAR of savings behind loan #1 | [FbnRRz5sW5…](https://testnet.nearblocks.io/txns/FbnRRz5sW5wjidgbBS8TX2aPr33ArsHtAd2dz5mtuv1f) |
| 19 | Loan B: surety files Sputnik Transfer proposal #1 on coop-mwfpr6a5.sputnik-v2.testnet (bond 0.1 NEAR) | [9SiGfr2YpC…](https://testnet.nearblocks.io/txns/9SiGfr2YpChuq5xXSKUJn7pveetrYbR3NouuZZDmhU66) |
| 20 | Loan B: council approves proposal #1 once in Sputnik -> DAO pays 1 NEAR to borrower-b-mwfpr6a5.testnet | [4diqL6dnVd…](https://testnet.nearblocks.io/txns/4diqL6dnVdKsXrSJbBU4J1BD5nqXRREvPdzPEHMYAQFP) |
| 21 | Loan B: sync reads proposal status -> loan #1 Active | [2Kmn47hoUx…](https://testnet.nearblocks.io/txns/2Kmn47hoUxCc8JZV12x2zD2GMpDC3GFKotWLsA6V2e2p) |
| 22 | Loan B: borrower-b-mwfpr6a5.testnet repays only 0.2 NEAR, then stops | [A2V2byrGYy…](https://testnet.nearblocks.io/txns/A2V2byrGYyvuwH7EW1SrDV6mBGhPtVujWhGFeBijPaVz) |
| 23 | Loan B: past due -> claim moves guarantors' locked savings (0.8 NEAR) to DAO treasury in one call | [4rJyNuqXD8…](https://testnet.nearblocks.io/txns/4rJyNuqXD8JHr2jyWQnRWPadMAaH1p5btsLNJamgCZtq) |

Key facts to check:
- **Step 15 (Loan A repaid):** the `loan_repaid` event shows borrower-a's `new_limit` = 1.5 NEAR, up from 1 NEAR.
- **Step 23 (Loan B claim):** one transaction, two receipts. The `claimed` event shows 0.4 NEAR taken from guarantor-1 and 0.4 NEAR from guarantor-2, and a 0.8 NEAR transfer to `coop-mwfpr6a5.sputnik-v2.testnet`. The DAO balance goes from 7.2 to 8 NEAR.
- **Steps 12/19:** the proposals filed on the DAO are ordinary Sputnik `Transfer` proposals:
  ```bash
  near view coop-mwfpr6a5.sputnik-v2.testnet get_proposal '{"id":1}'
  ```

### 3. Read state yourself
```bash
near view surety-mwfpr6a5.testnet get_members
```
```bash
near view surety-mwfpr6a5.testnet get_loan '{"loan_id":1}'
```
**See:** borrower-a `loans_repaid: 1`, `limit` 1.5 NEAR; borrower-b `loans_defaulted: 1`, `limit` "0"; loan 1 `status: "Defaulted"`, `claimed` 0.8 NEAR.

### 4. Run your own loan by hand
The board's **Do X → see Y** section lists each `near call` (deposit → request_loan → pledge ×2 → file_proposal → council `act_proposal` → sync → repay *or* wait and claim) next to what changes on the board after it. Expected failures you can trigger:
- requesting above your limit → `amount exceeds your limit`
- pledging on your own loan → `borrower cannot guarantee own loan`
- filing with one guarantor → `not enough guarantors`
- claiming early → `loan is not past due`

### 5. Rebuild, test and redeploy from scratch
Requires Rust (the toolchain is pinned to 1.86 in `contract/rust-toolchain.toml`) and Node 22.
```bash
cd contract && cargo test
```
**See:** `17 passed`. The tests cover limits, pledging rules, full cover and 2-guarantor checks, repay → limit rise, pro-rata claim with exact rounding, rejected proposals, failed filing, and locked savings that cannot be withdrawn.
```bash
cargo build --target wasm32-unknown-unknown --release
```
```bash
cd ../scripts && npm ci
```
Then move `demo/state.json` and `demo/txs.json` aside, so the scripts don't reuse the recorded accounts:
```bash
KEYS_DIR=$HOME/.surety-keys node setup.js
```
```bash
KEYS_DIR=$HOME/.surety-keys node demo.js
```
**See:** `setup.js` creates six named accounts from the testnet faucet, a new Sputnik DAO, deploys and initializes Surety, and funds the treasury. `demo.js` runs Loan A (repaid) and Loan B (defaulted and claimed after 90 s) and writes a fresh `demo/txs.json`. Keys are written only to `KEYS_DIR`. If your network needs a proxy, run Node with `NODE_USE_ENV_PROXY=1`.

### 6. See the DAO in Trezu
Hosted Trezu (trezu.app) reads mainnet DAOs only. The Trezu repo includes a dev mock backend for testnet DAOs (see [Trezu nt-fe README](https://github.com/NEAR-DevHub/trezu/blob/main/nt-fe/README.md)):
```bash
git clone https://github.com/NEAR-DevHub/trezu && cd trezu/nt-fe && bun install && bun run testnet:mock
```
In a second terminal:
```bash
NEXT_PUBLIC_NEAR_NETWORK=testnet NEXT_PUBLIC_NEAR_RPC_URL=https://rpc.testnet.fastnear.com NEXT_PUBLIC_BACKEND_API_BASE=http://127.0.0.1:8889 bun run dev
```
Open <http://localhost:3000/coop-mwfpr6a5.sputnik-v2.testnet/requests>. **Expected:** the two Surety Transfer proposals as normal payment requests, approved by the council. *(Not yet verified in the recorded run; this README will be updated with a screenshot.)*

## Contract API (`contract/src/lib.rs`)

| Method | Who | Effect |
|---|---|---|
| `new(dao, base_limit, limit_step, min_guarantors)` | deployer | bind to one Sputnik DAO |
| `deposit()` payable | member | add free savings |
| `withdraw(amount)` | member | withdraw free (unlocked) savings |
| `request_loan(amount, duration_sec, purpose?)` | named member | open a loan ≤ limit; one in-progress loan per borrower |
| `pledge(loan_id, amount)` / `unpledge(loan_id)` | named member ≠ borrower | lock or unlock savings behind an open loan |
| `cancel_loan(loan_id)` | borrower | cancel before filing; unlocks pledges |
| `file_proposal(loan_id)` payable (= DAO bond) | anyone | `add_proposal` Transfer on the DAO |
| `sync(loan_id)` | anyone | read `get_proposal`; activate or cancel |
| `repay(loan_id)` payable | anyone | forward to DAO treasury; excess refunded |
| `claim(loan_id)` | anyone, after due | guarantors' locked savings → DAO treasury, pro rata |
| `get_config`, `get_member`, `get_members`, `get_loan`, `get_loans`, `get_limit` | view | |

The contract emits NEP-297-style `EVENT_JSON` logs (`deposit`, `loan_requested`, `pledged`, `proposal_filed`, `loan_active`, `repaid`, `loan_repaid`, `claimed`, …).

## Scope and known limits

- Native NEAR only. FT/stablecoin loans would use the same `Transfer` with `token_id`; not built yet.
- No interest or fees.
- The due date starts when `sync` sees `Approved`, not at the DAO's payout block. Anyone can call `sync` right after the vote.
- A default takes the guarantors' pledges only. The borrower's own savings are not seized, but their limit goes to 0.
- Guarantor and borrower addresses must be named accounts. Implicit (64-hex / 0x) accounts are rejected.
- Surety does not change the DAO's policy and cannot move treasury funds. Removing it means the DAO simply stops approving its proposals.

## Built with

near-sdk-rs 5.24, near-api-js 5.1, Tailwind CSS 3.4 and Lucide icons (compiled into the page at build time), Geist font, Sputnik DAO v2 (`sputnik-v2.testnet` factory, unmodified), Trezu (unmodified, for viewing). All code was written during the hackathon window with Claude Code.

## Other NEAR Hackathon entry by the same author (rule Q-015)

This author's second, independent NEAR Hackathon project is **PreSafe**: <https://presafe.ahmardchain.workers.dev/>. Surety and PreSafe share no code and are submitted separately.

## License

MIT, see [LICENSE](LICENSE).
