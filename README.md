# Surety

A community that saves together should be able to lend to its own members, with members, not collateral, standing behind each loan.

Surety is a small NEAR contract that sits next to an existing **Sputnik DAO v2**. Custody and approvals stay in the DAO (visible in Trezu). Surety adds the loan logic: savings, guarantor pledges, filing the payout as a normal Sputnik `Transfer` proposal, routing repayments back to the treasury, and moving guarantors' locked savings to the treasury when a loan defaults. Each member's borrowing limit is derived from their real repayment record.

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
| Sputnik DAO (made with the official `sputnik-v2.testnet` factory) | [`coop-mwfopr1t.sputnik-v2.testnet`](https://testnet.nearblocks.io/address/coop-mwfopr1t.sputnik-v2.testnet) |
| Surety contract | [`surety-coop-mwfopr1t.testnet`](https://testnet.nearblocks.io/address/surety-coop-mwfopr1t.testnet) |
| Council (only DAO voter) | `council-coop-mwfopr1t.testnet` |
| Borrower, Loan A (repays) | `ada-coop-mwfopr1t.testnet` |
| Borrower, Loan B (defaults) | `dayo-coop-mwfopr1t.testnet` |
| Guarantors on both loans | `bola-coop-mwfopr1t.testnet`, `chidi-coop-mwfopr1t.testnet` |

DAO policy: `Everyone` may add proposals, the council group votes, the threshold is 1/2 and the proposal bond is 0.1 NEAR.

## Try it: do X → see Y

### 1. See the live board
```bash
git clone https://github.com/ahmardchain/surety && cd surety
```
```bash
python3 -m http.server 8765
```
Open <http://127.0.0.1:8765/web/>. **See:**
- **Treasury:** 8 NEAR in the DAO; 2 NEAR paid out through Sputnik; 2 NEAR returned to the DAO.
- **Loan #0 (ada): Repaid.** 1 NEAR came back from the borrower.
- **Loan #1 (dayo): Defaulted.** 0.2 NEAR was repaid by dayo and 0.8 NEAR claimed from the guarantors.
- **Members:** ada's limit is 1.5 NEAR, up from 1. bola and chidi are at 1.5 NEAR each, with 0.4 NEAR each lost to the claim. dayo's limit is 0.

The page reads the chain directly (`?contract=<id>` points it at another deployment).

### 2. Check the recorded run on-chain (same DAO, two loans)
Every step below was sent by `scripts/setup.js` and `scripts/demo.js`. The raw log, with balances before and after, is in [`demo/txs.json`](demo/txs.json).

| # | Step | Tx |
|---|---|---|
| 1 | Create Sputnik DAO via sputnik-v2.testnet factory | [GFu4K4VvUW…](https://testnet.nearblocks.io/txns/GFu4K4VvUW1aENGRT9HAWsYHuXwNpcKEo7KKLgFUGg6b) |
| 2 | Deploy surety contract | [BT3UmsSMgB…](https://testnet.nearblocks.io/txns/BT3UmsSMgBuNEwp6XxNKDXXm6TpcTqiBevJgdFrnjJZR) |
| 3 | Init surety (dao, base limit 1 NEAR, +0.5 NEAR per repaid loan/guarantee, 2 guarantors) | [5ncb16LSHg…](https://testnet.nearblocks.io/txns/5ncb16LSHg9nB64zaxdZ4Uq9ShiNzp3PTK2Y4WuPQTY7) |
| 4 | Council tops up DAO treasury with 2 NEAR | [8J5Lmc9FBu…](https://testnet.nearblocks.io/txns/8J5Lmc9FBuGATMk7SgisoS2wV68gpWB22oHSgUWd9sU4) |
| 5 | ada-coop-mwfopr1t.testnet deposits 0.5 NEAR savings | [A5AqRK9P8p…](https://testnet.nearblocks.io/txns/A5AqRK9P8pgRMP933x336F6uJdL3rCcn3DXQ5sEJ7Hg6) |
| 6 | dayo-coop-mwfopr1t.testnet deposits 0.5 NEAR savings | [5o7Y5nmW7F…](https://testnet.nearblocks.io/txns/5o7Y5nmW7Fxzt1vG3JGP4HuorwUoBeWoBEMaqJYYByS6) |
| 7 | bola-coop-mwfopr1t.testnet deposits 1.5 NEAR savings | [GWfFTh2QD8…](https://testnet.nearblocks.io/txns/GWfFTh2QD8m1Akg9WarHLTQG7tq9xFT1o53cKFNwTX7K) |
| 8 | chidi-coop-mwfopr1t.testnet deposits 1.5 NEAR savings | [FdFmZbVaWf…](https://testnet.nearblocks.io/txns/FdFmZbVaWfK9A5PsehuoxHThrTJzKezJsmNmfomA3Yx2) |
| 9 | Loan A: ada-coop-mwfopr1t.testnet requests loan #0 of 1 NEAR, 3600s term | [CTdqVybiMb…](https://testnet.nearblocks.io/txns/CTdqVybiMbXb2wXxydZFQVASvHonf86oTNhWqESr1pgC) |
| 10 | Loan A: bola-coop-mwfopr1t.testnet pledges 0.5 NEAR of savings behind loan #0 | [9TP7BsijGh…](https://testnet.nearblocks.io/txns/9TP7BsijGhiZDzzZTt3LJHBswqWkzSYJBuciRmzHhYJv) |
| 11 | Loan A: chidi-coop-mwfopr1t.testnet pledges 0.5 NEAR of savings behind loan #0 | [CW26JXrzfL…](https://testnet.nearblocks.io/txns/CW26JXrzfLRfhkDbnudkWb1theVTC57VmoN19rbDaYkb) |
| 12 | Loan A: surety files Sputnik Transfer proposal #0 on coop-mwfopr1t.sputnik-v2.testnet (bond 0.1 NEAR) | [Hofi7792vC…](https://testnet.nearblocks.io/txns/Hofi7792vC4sayQEh9kjkTh8wcrPkbj93ssk38VCzvUb) |
| 13 | Loan A: council approves proposal #0 once in Sputnik -> DAO pays 1 NEAR to ada-coop-mwfopr1t.testnet | [CVXz8L1eJN…](https://testnet.nearblocks.io/txns/CVXz8L1eJNWU4mxHx5tPeTmHwSJCU1L74Er18GLnTejA) |
| 14 | Loan A: sync reads proposal status -> loan #0 Active | [FtftLbQetf…](https://testnet.nearblocks.io/txns/FtftLbQetfz3yNtxExdQ5QvmtD5bLerJDSbjMfiSbed1) |
| 15 | Loan A: ada repays 1 NEAR -> forwarded to DAO treasury; pledges released | [ymXdFUqmGH…](https://testnet.nearblocks.io/txns/ymXdFUqmGHf5sTcQYNn5dKmsJiKdUi7Ur1NrjH2rAie) |
| 16 | Loan B: dayo-coop-mwfopr1t.testnet requests loan #1 of 1 NEAR, 90s term | [GuN4vQNJgc…](https://testnet.nearblocks.io/txns/GuN4vQNJgcpDyxYNx8ePsfvpD2tDHN2yaZgLjAn9MyJy) |
| 17 | Loan B: bola-coop-mwfopr1t.testnet pledges 0.5 NEAR of savings behind loan #1 | [BbJnp8RmhF…](https://testnet.nearblocks.io/txns/BbJnp8RmhFhhX8Y8aQSfN1Anwgf5A8RnSDFrXPj3WYX5) |
| 18 | Loan B: chidi-coop-mwfopr1t.testnet pledges 0.5 NEAR of savings behind loan #1 | [6VgSUtiBEg…](https://testnet.nearblocks.io/txns/6VgSUtiBEgDLvqA17p3DvgypLq9rHz3RvKYnXvPeH7zi) |
| 19 | Loan B: surety files Sputnik Transfer proposal #1 on coop-mwfopr1t.sputnik-v2.testnet (bond 0.1 NEAR) | [2A85EYKZqV…](https://testnet.nearblocks.io/txns/2A85EYKZqVRLtZCWDDKVyoaGT3QsnWLnVJFtpLbFx72M) |
| 20 | Loan B: council approves proposal #1 once in Sputnik -> DAO pays 1 NEAR to dayo-coop-mwfopr1t.testnet | [JDvoaT6YvZ…](https://testnet.nearblocks.io/txns/JDvoaT6YvZP2RcgsMuNnut9D8nXYGm7PjC95QJBeU8gH) |
| 21 | Loan B: sync reads proposal status -> loan #1 Active | [pEqRSDev4r…](https://testnet.nearblocks.io/txns/pEqRSDev4rUD7o8f4CyXnPtizGpWDSvXxNhoXwNEmYK) |
| 22 | Loan B: dayo repays only 0.2 NEAR, then stops | [7bU91TVG7r…](https://testnet.nearblocks.io/txns/7bU91TVG7rrcLikBSdHfHrpXu9kqRG6N288VCnhWDJ66) |
| 23 | Loan B: past due -> claim moves guarantors' locked savings (0.8 NEAR) to DAO treasury in one call | [4RWTSDRgdA…](https://testnet.nearblocks.io/txns/4RWTSDRgdABqkDbBN1FsJkNjj3otc49242xbmAFRhjSr) |

Key facts to check:
- **Step 15 (Loan A repaid):** the `loan_repaid` event shows ada's `new_limit` = 1.5 NEAR, up from 1 NEAR.
- **Step 23 (Loan B claim):** one transaction, two receipts. The `claimed` event shows 0.4 NEAR taken from bola and 0.4 NEAR from chidi, and a 0.8 NEAR transfer to `coop-mwfopr1t.sputnik-v2.testnet`. The DAO balance goes from 7.2 to 8 NEAR.
- **Steps 12/19:** the proposals filed on the DAO are ordinary Sputnik `Transfer` proposals:
  ```bash
  near view coop-mwfopr1t.sputnik-v2.testnet get_proposal '{"id":1}'
  ```

### 3. Read state yourself
```bash
near view surety-coop-mwfopr1t.testnet get_members
```
```bash
near view surety-coop-mwfopr1t.testnet get_loan '{"loan_id":1}'
```
**See:** ada `loans_repaid: 1`, `limit` 1.5 NEAR; dayo `loans_defaulted: 1`, `limit` "0"; loan 1 `status: "Defaulted"`, `claimed` 0.8 NEAR.

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
Open <http://localhost:3000/coop-mwfopr1t.sputnik-v2.testnet/requests>. **Expected:** the two Surety Transfer proposals as normal payment requests, approved by the council. *(Not yet verified in the recorded run; this README will be updated with a screenshot.)*

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

near-sdk-rs 5.24, near-api-js 5.1, Sputnik DAO v2 (`sputnik-v2.testnet` factory, unmodified), Trezu (unmodified, for viewing). All code was written during the hackathon window with Claude Code.

## Other NEAR Hackathon entry by the same author (rule Q-015)

This author's second, independent NEAR Hackathon project is **PreSafe**: <https://presafe.ahmardchain.workers.dev/>. Surety and PreSafe share no code and are submitted separately.

## License

MIT, see [LICENSE](LICENSE).
