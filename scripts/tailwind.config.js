module.exports = {
  content: [require("path").join(__dirname, "..", "web", "index.html")],
  darkMode: "media",
  theme: { extend: {
    fontFamily: { sans: ["Geist", "system-ui", "sans-serif"], mono: ["Geist Mono", "ui-monospace", "monospace"] },
    colors: { brand: { 50: "#ecfdf6", 100: "#d1fae9", 400: "#34d39e", 500: "#10b981", 600: "#059669", 700: "#047857" } },
    boxShadow: { card: "0 1px 2px rgba(0,0,0,.04), 0 4px 16px -4px rgba(0,0,0,.06)" },
  } },
};
