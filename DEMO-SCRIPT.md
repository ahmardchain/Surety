# Demo video script

Video: [X post](https://x.com/Ahmardchain/status/2121431624369705119) (1:13, 1080p, burned-in captions). Rebuild it from this repo with `scripts/video/make.sh` (Playwright screen recording + Microsoft Edge neural TTS voice `en-US-AndrewNeural` + ffmpeg).

| # | Time | Screen | Narration |
|---|---|---|---|
| 1 | 0:00 | Board hero (caption: "Built at NEAR HACKATHON · #NEARHackathon") | Savings groups can pool money in a DAO. What they can't do safely, is lend it to their own members. |
| 2 | 0:06 | "How a loan moves", six steps highlighted in turn | Surety fixes that. A member asks for a loan. Two other members lock part of their savings behind it. That's the guarantee, not collateral. |
| 3 | 0:15 | Loan #0 card, then its "Sputnik proposal #0 · Transfer" row | When the loan is fully covered, Surety files a normal Sputnik DAO transfer. The council approves it once, in Sputnik or Trezu. The money never leaves the DAO's control. |
| 4 | 0:26 | Members table (borrower-a 1.5 NEAR ↑), then the repay transaction read from RPC | Loan A: the borrower repays. The money goes straight back to the treasury, the guarantors' savings unlock, and everyone's borrowing limit goes up, from one, to one point five NEAR. Access is earned by repaying. |
| 5 | 0:40 | Loan #1 card, red Defaulted badge and split bar | Loan B: the second borrower pays zero point two, and stops. Once it's past due, anyone can call claim. |
| 6 | 0:48 | The claim transaction: −0.4 NEAR from each guarantor, +0.8 NEAR to the DAO, `claimed` event | One transaction. Zero point four NEAR from each guarantor's locked savings moves to the treasury, and it's whole again. No chasing. No bad-debt vote. And the defaulter's limit drops to zero. |
| 7 | 1:01 | On-chain proof timelines (caption: repo + board links) | Same DAO, two loans, every step on NEAR testnet, with a transaction link. Built on Sputnik DAO and named accounts. The repo and the live board are linked below. |

The transaction scenes use `scripts/video/tx.html`, which reads the real transaction from NEAR RPC (`EXPERIMENTAL_tx_status`) because NearBlocks refuses automated browsers.
