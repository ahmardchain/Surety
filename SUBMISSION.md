# Submission (mirror)

Public fields of the NEAR HACKATHON submission, kept in sync with what was sent. Contact details are omitted.

| Field | Value |
|---|---|
| Project | Surety |
| Event | [NEAR HACKATHON: Build Beyond Boundaries](https://nearhackathon.com) |
| Track | Social Money (`social`) |
| Team size | 1 |
| Repository | https://github.com/ahmardchain/surety |
| Demo post | https://x.com/Ahmardchain/status/2121431624369705119 |
| Live board | https://surety.ahmardchain.workers.dev/ |
| Network | NEAR testnet |
| Community listing | https://nearhunt.xyz/p/40a2e147-c51e-4a86-b179-4b3b9f0233d2 |

## Description (as submitted)

Surety lets a community that saves together lend to its own members, with members, not collateral, standing behind each loan. Savings groups can pool money in a Sputnik DAO but cannot safely lend it to their own members: a payout is a one-way Transfer with nothing linking it to repayment, and no way for other members to stand behind it. Surety is a small NEAR contract next to an existing Sputnik DAO v2. Members deposit savings; a member requests a loan within their limit; two named-account members lock part of their savings as guarantors; once fully covered, Surety files a normal Sputnik Transfer proposal and the council approves it once (Sputnik/Trezu), so custody never leaves the DAO. Repayments go straight to the treasury, unlock guarantors and raise borrowing limits (reputation tied to real repayments). After the due date anyone calls claim(), which moves the guarantors' locked savings to the treasury in one transaction. Demo on NEAR testnet, one DAO, two loans: loan A repaid (limit 1→1.5 NEAR), loan B defaulted and claimed (treasury 7.2→8 NEAR). Other entry by the same author: PreSafe.

## Pre-submit checklist (from the organizers' SUBMITTING.md)

- [x] Repository public; README has "do X → see Y" steps an AI judge can follow
- [x] Demo post live on X, says it was built at NEAR HACKATHON, video attached
- [x] Mainnet NEAR account for the payout
- [x] Every fitting track selected
- [x] No keys or secrets anywhere in the repo history (scanned with `git log -p`)
- [x] Submitted before 2026-11-14 23:59 UTC (API answered 201)

Since submission, the repository gained Loan C (removed-proposal fix, on-chain), the judge quickstart and these supporting docs. The demo and claims in the description are unchanged.
