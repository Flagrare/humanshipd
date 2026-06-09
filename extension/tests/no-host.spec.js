// Guards the "drop the native host" rework: the extension must not regrow native
// messaging, and the host crate/installer must stay deleted.

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const ext = (...p) => path.join(__dirname, "..", ...p);

test("manifest has no native-messaging wiring", () => {
  const m = JSON.parse(fs.readFileSync(ext("manifest.json"), "utf8"));
  expect(m.permissions || []).not.toContain("nativeMessaging");
  expect(m.background).toBeUndefined();
});

test("native host crate, installer, and background relay are gone", () => {
  expect(fs.existsSync(ext("..", "native-host"))).toBe(false);
  expect(fs.existsSync(ext("host"))).toBe(false);
  expect(fs.existsSync(ext("background.js"))).toBe(false);
});
