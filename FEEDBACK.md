# Builder feedback (Sputnik DAO, Trezu, NEAR tooling)

Things hit while building Surety, each with where it showed up. Written for the NEAR, Sputnik DAO and Trezu teams.

## Sputnik DAO v2

- **Removed proposals vanish.** A `VoteRemove` deletes the proposal, so `get_proposal(id)` panics with `ERR_NO_PROPOSAL` instead of returning `status: "Removed"`. Any contract that files proposals and later reads their outcome has to treat "not found" as Removed. Surety's first deployment left guarantors' savings locked because of this; fixed in `on_proposal_status` and proven on-chain with Loan C (README tx steps 24–30). A tombstone (status kept, kind dropped) would make integrations safer.
- **The bond must be exact.** `add_proposal` asserts `attached_deposit == proposal_bond`, so attaching more fails. A contract that files proposals must read the policy or pass the exact bond through.
- **`act_proposal` needs the proposal kind.** Current builds require `proposal: <kind>` (anti front-running); older docs and scripts show only `id` and `action`. Surety's demo script reads `get_proposal` first and passes the kind back.
- **Contracts as proposers work well.** The default policy's `Everyone: *:AddProposal` lets a contract file Transfers, and the bond is returned to it on Approve/Reject/Expire. That is what makes "build on the DAO, don't replace it" possible.

## Trezu

- **Hosted Trezu is mainnet-only.** Testnet DAOs need the dev-only mock backend from `nt-fe` (`bun run testnet:mock`), which is not mentioned in the public docs. A testnet toggle on trezu.app would help every hackathon team on testnet.
- **Install needs GitHub API access.** `@hot-labs/near-connect` is a GitHub tarball dependency; in a restricted network the tarball download returned 403 and blocked `bun install`. Cloning that repo at the pinned commit and using a `file:` dependency worked.

## Toolchain

- **near-sdk vs. nearcore wasm.** near-sdk 5.30 requires rustc ≥ 1.95, whose default wasm output uses features older nearcore runtimes reject. Pinning `near-sdk = "=5.24.1"`, Rust 1.86 (`rust-toolchain.toml`) and `resolver = "3"` (MSRV-aware) produced deployable wasm without cargo-near.
- **Empty view args.** Calling a view method with `args_base64: ""` fails to deserialize; `e30=` (`{}`) works. Worth a line in the RPC docs since curl examples often use `""`.
- **Testnet faucet helper** (`helper.testnet.near.org/account`) creates a funded named account in one POST; it made a 6-account demo reproducible from a script.
- **NearBlocks** refuses headless browsers, so demo recordings read transactions from RPC (`EXPERIMENTAL_tx_status`) instead.
