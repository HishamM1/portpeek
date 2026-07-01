import assert from "node:assert/strict";
import test from "node:test";
import { brandSlug, groupPorts, isDatabase } from "./ports.js";

const port = (id, pid, number, processName = "node.exe") => ({
  id,
  pid,
  port: number,
  processName,
  displayName: null,
  command: null,
  executablePath: null,
  framework: null,
});

test("groups listeners by PID and recognizes desktop apps", () => {
  const groups = groupPorts([port("a", 42, 3000), port("b", 42, 3001), port("c", 77, 4000)]);
  assert.deepEqual(groups.map((group) => group.map((item) => item.port)), [[3000, 3001], [4000]]);
  assert.equal(brandSlug(port("cursor", 9, 3000, "Cursor.exe")), "cursor");
  assert.equal(brandSlug(port("tailscale", 10, 41112, "tailscaled.exe")), "tailscale");
  assert.equal(brandSlug(port("postgres", 11, 5432, "postgres.exe")), "postgresql");
  assert.equal(isDatabase(port("sqlserver", 12, 1433, "sqlservr.exe")), true);
});
