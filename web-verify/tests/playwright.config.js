const path = require("path");

// Builds the WASM bundle, then serves web-verify/ statically so the spec runs
// against the real, compiled verifier — the same artifact GitHub Pages ships.
module.exports = {
  testDir: ".",
  timeout: 60000,
  use: { browserName: "chromium", baseURL: "http://localhost:8753" },
  webServer: {
    command: "wasm-pack build --target web --out-dir pkg && python3 -m http.server 8753",
    cwd: path.resolve(__dirname, ".."),
    url: "http://localhost:8753/verify.html",
    timeout: 180000,
    reuseExistingServer: !process.env.CI,
  },
};
