// web/index.html (dev: Tailwind + Lucide from CDN) -> web/dist/index.html (prod: compiled Tailwind CSS, inlined icons, no CDN scripts).
const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const lucide = require("lucide");
const root = path.join(__dirname, "..");
const src = fs.readFileSync(path.join(root, "web", "index.html"), "utf8");
fs.mkdirSync(path.join(root, "web", "dist"), { recursive: true });

const inCss = path.join(root, "web", "dist", "in.css");
fs.writeFileSync(inCss, "@tailwind base;@tailwind components;@tailwind utilities;");
const css = execFileSync(path.join(__dirname, "node_modules", ".bin", "tailwindcss"),
  ["-c", path.join(__dirname, "tailwind.config.js"), "-i", inCss, "--minify"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
fs.unlinkSync(inCss);

// icon names used: data-lucide="x" literals plus names inside JS string arrays
const names = new Set([...src.matchAll(/data-lucide="([a-z0-9-]+)"/g)].map((m) => m[1]));
for (const m of src.matchAll(/["']([a-z]+(?:-[a-z0-9]+)+|[a-z]{3,})["']/g)) names.add(m[1]);
const pascal = (n) => n.split("-").map((p) => p[0].toUpperCase() + p.slice(1)).join("");
const toSvg = ([tag, attrs, children = []]) =>
  `<${tag}${Object.entries(attrs).map(([k, v]) => ` ${k}="${v}"`).join("")}>${children.map(toSvg).join("")}</${tag}>`;
const map = {};
for (const n of names) {
  const node = lucide.icons[pascal(n)];
  if (node) map[n] = node[2].map(toSvg).join("");
}
const iconJs = `<script>window.lucide={createIcons(){const M=${JSON.stringify(map)};document.querySelectorAll("i[data-lucide]").forEach(el=>{const n=el.getAttribute("data-lucide");if(!M[n])return;const s=document.createElementNS("http://www.w3.org/2000/svg","svg");s.setAttribute("viewBox","0 0 24 24");s.setAttribute("fill","none");s.setAttribute("stroke","currentColor");s.setAttribute("stroke-width","2");s.setAttribute("stroke-linecap","round");s.setAttribute("stroke-linejoin","round");s.setAttribute("class",el.getAttribute("class")||"");s.setAttribute("aria-hidden","true");s.innerHTML=M[n];el.replaceWith(s)})}};</script>`;

let out = src
  .replace(/<script src="https:\/\/cdn\.tailwindcss\.com[^"]*"><\/script>\s*/, "")
  .replace(/<script src="https:\/\/unpkg\.com\/lucide[^"]*"><\/script>\s*/, iconJs)
  .replace(/<script>\s*tailwind\.config[\s\S]*?<\/script>\s*/, `<style>${css}</style>\n`);
if (/cdn\.tailwindcss|unpkg\.com/.test(out)) throw new Error("CDN reference left in output");
fs.writeFileSync(path.join(root, "web", "dist", "index.html"), out);
console.log(`wrote web/dist/index.html (${out.length} bytes, ${Object.keys(map).length} icons, ${css.length} bytes css)`);
