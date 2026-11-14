# X post

Published: https://x.com/Ahmardchain/status/2121431624369705119 (video attached)

```
Savings groups can pool money in a DAO. What they can't do safely is lend it to their own members.

Surety fixes that, built at #NEARHackathon on @NEARProtocol:

1. Members deposit savings.
2. A member requests a loan within their limit.
3. Two named-account members lock part of their savings behind it. That's the guarantee, not collateral.
4. Surety files a normal Sputnik DAO Transfer. The council approves once. Custody never leaves the DAO.
5. Repay: money goes straight to the treasury, guarantors unlock, everyone's limit rises.
6. Default: after the due date anyone calls claim(), and the guarantors' locked savings make the treasury whole in one transaction. No chasing, no bad-debt vote.

What happened on testnet, on one DAO:
• Loan A repaid. Borrower's limit went from 1 to 1.5 NEAR.
• Loan B paid 0.2 of 1 NEAR, then stopped. One claim moved 0.4 NEAR from each guarantor, and the treasury went from 7.2 back to 8 NEAR.
Claim tx: https://testnet.nearblocks.io/txns/4rJyNuqXD8JHr2jyWQnRWPadMAaH1p5btsLNJamgCZtq

Borrowing limits are earned only by real repayments, tracked per named account.

Live board: https://surety.ahmardchain.workers.dev/
Code: https://github.com/ahmardchain/surety

If you run a savings group or a community DAO: would your members guarantee each other like this?

@NearBuilders @NearLegion
```
