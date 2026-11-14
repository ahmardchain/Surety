const { chromium } = require('playwright');
const fs = require('fs');
const D = __dirname;
const SCENE = 15000;
(async () => {
  const b = await chromium.launch({ proxy: { server: process.env.HTTPS_PROXY, bypass: 'localhost,127.0.0.1' } });
  const ctx = await b.newContext({ viewport: { width: 1600, height: 900 }, recordVideo: { dir: D + '/raw', size: { width: 1600, height: 900 } }, ignoreHTTPSErrors: true });
  const p = await ctx.newPage();
  const t0 = Date.now(); const marks = [];
  const mark = (n) => { marks.push({ scene: n, t: (Date.now() - t0) / 1000 }); };
  const end = (n) => { marks.push({ end: n, t: (Date.now() - t0) / 1000 }); };
  const css = `.ring{outline:3px solid #10b981!important;outline-offset:4px;border-radius:14px;box-shadow:0 0 0 10px rgba(16,185,129,.15)!important;transition:all .3s}`;
  const scrollTo = (sel, off = 90) => p.evaluate(([s, o]) => { const e = document.querySelector(s); window.scrollTo({ top: e.getBoundingClientRect().top + scrollY - o, behavior: 'smooth' }); }, [sel, off]);
  const ring = (sel) => p.evaluate((s) => { document.querySelectorAll('.ring').forEach(e => e.classList.remove('ring')); if (s) document.querySelector(s).classList.add('ring'); }, sel);
  const hold = async (start) => { const left = SCENE - (Date.now() - start); if (left > 0) await p.waitForTimeout(left); };
  const board = async () => { await p.goto('http://127.0.0.1:8766/board.html'); await p.waitForFunction(() => document.querySelector('#members tr') && document.querySelector('#txs ol')); await p.addStyleTag({ content: css }); await p.waitForTimeout(800); };
  const tx = async (hash, sender, title) => { await p.goto(`http://127.0.0.1:8766/tx.html?tx=${hash}&sender=${sender}&title=${encodeURIComponent(title)}`); await p.waitForFunction(() => document.querySelector('#events .ev') && /#/.test(document.getElementById('block').textContent)); await p.waitForTimeout(500); };

  await board();
  // S1 hero
  let s = Date.now(); mark(1); await p.waitForTimeout(2500); await ring('main > section:nth-of-type(1)'); await hold(s); end(1);
  // S2 flow
  s = Date.now(); mark(2); await ring(null); await scrollTo('#flow', 140); await p.waitForTimeout(900);
  for (let i = 1; i <= 6; i++) { await ring(`#flow li:nth-child(${i})`); await p.waitForTimeout(1300); } await hold(s); end(2);
  // S3 loan #0 + proposal row
  s = Date.now(); mark(3); await ring(null); await scrollTo('#loans', 120); await p.waitForTimeout(1200); await ring('#loans article:nth-child(2)'); await p.waitForTimeout(3000); await ring('#loans article:nth-child(2) dl > div:nth-child(2)'); await hold(s); end(3);
  // S4 members limit, then repay tx
  s = Date.now(); mark(4); await ring(null); await scrollTo('#members', 260); await p.waitForTimeout(1200); await ring('#members tr:nth-child(1)'); await p.waitForTimeout(3500);
  marks.push({ gap4: 'nav', t: (Date.now() - t0) / 1000 }); await tx('81JeQZDs6Ep6pt2fuzeXc5JSBkW9d3JS938nLwsQMLUJ', 'borrower-a-mwfpr6a5.testnet', 'Loan A: borrower-a repays 1 NEAR'); marks.push({ gap4: 'ready', t: (Date.now() - t0) / 1000 + 0.15 }); s -= 0; await p.addStyleTag({ content: css }); await ring('#flows'); await p.waitForTimeout(12000); end(4);
  // S5 loan #1 defaulted
  await board(); s = Date.now(); mark(5); await scrollTo('#loans', 120); await p.waitForTimeout(1200); await ring('#loans article:nth-child(1)'); await p.waitForTimeout(3000); await ring('#loans article:nth-child(1) .mt-4'); await hold(s); end(5);
  // S6 claim tx
  await tx('4rJyNuqXD8JHr2jyWQnRWPadMAaH1p5btsLNJamgCZtq', 'council-mwfpr6a5.testnet', 'Loan B past due: anyone calls claim()'); s = Date.now(); mark(6); await p.addStyleTag({ content: css }); await p.waitForTimeout(1500); await ring('#flows'); await p.waitForTimeout(6000); await p.evaluate(() => window.scrollTo({ top: 400, behavior: 'smooth' })); await ring('#events'); await hold(s); end(6);
  // S7 proof + members
  await board(); s = Date.now(); mark(7); await scrollTo('#txs', 120); await p.waitForTimeout(1500); await ring('#txs > div:nth-child(2)'); await p.waitForTimeout(2500); await ring('#txs > div:nth-child(3)'); await p.waitForTimeout(3000); await ring(null); await p.evaluate(() => window.scrollTo({ top: 0, behavior: 'smooth' })); await hold(s); end(7);
  mark(8);
  await ctx.close(); await b.close();
  const vid = fs.readdirSync(D + '/raw').filter(f => f.endsWith('.webm')).map(f => D + '/raw/' + f)[0];
  fs.writeFileSync(D + '/marks.json', JSON.stringify({ vid, marks }, null, 1));
  console.log(JSON.stringify(marks));
})();
