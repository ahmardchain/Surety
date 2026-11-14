// Bundles web/index.html + demo/txs.json into a single Cloudflare Worker module (web/dist/worker.js).
const fs = require("fs");
const path = require("path");
const root = path.join(__dirname, "..");
require("./build-web.js");
const html = fs.readFileSync(path.join(root, "web", "dist", "index.html"), "utf8");
const txs = fs.readFileSync(path.join(root, "demo", "txs.json"), "utf8");
const out = `const HTML = ${JSON.stringify(html)};
const TXS = ${JSON.stringify(txs)};
export default {
  async fetch(req) {
    const { pathname } = new URL(req.url);
    if (pathname === "/txs.json") return new Response(TXS, { headers: { "content-type": "application/json", "access-control-allow-origin": "*" } });
    if (pathname === "/" || pathname === "/index.html") return new Response(HTML, { headers: { "content-type": "text/html; charset=utf-8" } });
    return new Response("not found", { status: 404 });
  },
};
`;
fs.mkdirSync(path.join(root, "web", "dist"), { recursive: true });
fs.writeFileSync(path.join(root, "web", "dist", "worker.js"), out);
console.log("wrote web/dist/worker.js", out.length, "bytes");
