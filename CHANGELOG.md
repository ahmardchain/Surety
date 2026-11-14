# Changelog

All work was done during NEAR HACKATHON (window opened 2026-10-19; deadline 2026-11-14 23:59 UTC).

## 2026-11-14
- **Fix:** a council-removed Sputnik proposal (deleted by Sputnik) no longer leaves a loan stuck with guarantors' savings locked; `sync` treats it as Removed. Deployed in place to `surety-mwfpr6a5.testnet` and demonstrated with Loan C. Tests: 19.
- README rebuilt to the judge-first format: verify-in-60-seconds quickstart, links table, architecture diagrams, live proof, real-vs-mock audit, criteria mapping. Added SUBMISSION, DEMO-SCRIPT, X-POST, FEEDBACK, CI workflow.
- Board: Loan C proof timeline; cancelled loans read "never paid out, pledges released".
- Demo video (1:13, captions + voiceover) and the scripts that rebuild it.
- Redeployed the demo on role-named accounts (`borrower-a/b`, `guarantor-1/2`, `council`) and a fresh DAO `coop-mwfpr6a5.sputnik-v2.testnet`.
- Polished board (Tailwind, Lucide, Geist; compiled at build time) hosted on Cloudflare Workers; custom logo.
- Contract: savings, guaranteed loan requests, pledges, Sputnik Transfer filing, sync, repay to treasury, pro-rata claim, reputation-based limits. Two-loan testnet demo with recorded transactions.
