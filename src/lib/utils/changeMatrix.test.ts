import assert from "node:assert/strict";
import { test } from "node:test";
import type { RegistryChange, TweakEffectOption } from "$lib/types";
import { buildMatrix, optionsMatchingNow, optionTone } from "./changeMatrix.ts";

const option = (label: string, parts: Partial<TweakEffectOption> = {}): TweakEffectOption => ({
  label,
  registry_changes: [],
  service_changes: [],
  scheduler_changes: [],
  hosts_changes: [],
  firewall_changes: [],
  power_changes: [],
  audit_changes: [],
  commands: [],
  ...parts,
});

const reg = (change: Partial<RegistryChange>): RegistryChange => ({
  hive: "HKLM",
  key: "Software\\Test",
  value_name: "Flag",
  action: "set",
  value_type: "REG_DWORD",
  value: 1,
  skip_validation: false,
  ...change,
});

test("optionTone marks the active option before the pending one", () => {
  assert.equal(optionTone("On", "On", "On"), "current");
  assert.equal(optionTone("Off", "On", "Off"), "pending");
  assert.equal(optionTone("Off", "On"), null);
  assert.equal(optionTone("Off", null, undefined), null);
});

test("one row per setting, matched case-insensitively across options", () => {
  const rows = buildMatrix(
    [
      option("On", { registry_changes: [reg({ value: 1 })] }),
      option("Off", { registry_changes: [reg({ key: "SOFTWARE\\TEST", value_name: "flag", value: 0 })] }),
    ],
    null,
  );
  assert.equal(rows.length, 1);
  assert.deepEqual(
    rows[0].cells.map((c) => c?.text),
    ["1 (0x1)", "0 (0x0)"],
  );
  assert.equal(rows[0].location, "HKLM\\Software\\Test");
  assert.equal(rows[0].type, "REG_DWORD");
  assert.equal(rows[0].now, null);
});

test("registry cells describe each action and value shape", () => {
  const cells = buildMatrix(
    [
      option("a", { registry_changes: [reg({ value: 255, windows_versions: [11] })] }),
      option("b", { registry_changes: [reg({ action: "delete_value", value_type: null, value: null })] }),
      option("c", { registry_changes: [reg({ value_type: "REG_SZ", value: "" })] }),
      option("d", { registry_changes: [reg({ value_type: "REG_MULTI_SZ", value: ["x", "y"] })] }),
      option("e", { registry_changes: [reg({ value: null })] }),
    ],
    null,
  )[0].cells;
  assert.deepEqual(cells, [
    { text: "255 (0xFF)", note: "Win 11" },
    { text: "Not set", removal: true },
    { text: '""' },
    { text: '["x","y"]' },
    { text: "(empty)" },
  ]);
});

test("a key action gets its own row, apart from values under that key", () => {
  const rows = buildMatrix(
    [option("a", { registry_changes: [reg({}), reg({ action: "delete_key", value_type: null, value: null })] })],
    null,
  );
  assert.equal(rows.length, 2);
  const key = rows.find((r) => r.title === "(key)");
  assert.ok(key);
  assert.equal(key.name, "Test");
  assert.equal(key.type, undefined);
  assert.deepEqual(key.cells[0], { text: "Key removed", removal: true });
});

test("an untouched setting leaves the option's cell null, and rows sort by kind", () => {
  const rows = buildMatrix(
    [
      option("a", {
        firewall_changes: [
          { name: "Rule", operation: "create", action: "block", direction: "outbound", skip_validation: false },
        ],
        hosts_changes: [{ ip: "0.0.0.0", domain: "ads.example", action: "add", skip_validation: false }],
        scheduler_changes: [{ task_path: "\\Microsoft\\Windows\\Task", action: "disable", skip_validation: false }],
        service_changes: [{ name: "Svc", startup: "automatic_delayed", skip_validation: false }],
      }),
      option("b", {
        registry_changes: [reg({})],
        firewall_changes: [
          { name: "rule", operation: "delete", direction: "inbound", action: "block", skip_validation: false },
        ],
      }),
    ],
    null,
  );
  assert.deepEqual(
    rows.map((r) => r.kind),
    ["registry", "service", "task", "hosts", "firewall"],
  );
  const [registry, service, task, hosts, firewall] = rows;
  assert.equal(registry.cells[0], null);
  assert.deepEqual(service.cells, [{ text: "Automatic delayed" }, null]);
  assert.equal(task.title, "Task");
  assert.equal(task.location, "\\Microsoft\\Windows");
  assert.deepEqual(task.cells[0], { text: "Disabled" });
  assert.deepEqual(hosts.cells[0], { text: "Mapped to 0.0.0.0" });
  assert.deepEqual(firewall.cells, [{ text: "Block outbound" }, { text: "Removed", removal: true }]);
});

test("power and audit rows sort last, one per setting or flag, named as the engine names them", () => {
  const wake = { name: "Allow wake timers", subgroup: "238c9fa8", setting: "bd3b718a" };
  const logon = (event: "success" | "failure", audited: boolean) => ({
    name: `Logon (${event})`,
    subcategory: "0cce9215",
    event,
    audited,
  });
  const rows = buildMatrix(
    [
      option("Off", {
        power_changes: [{ ...wake, ac: 0, dc: 0 }],
        audit_changes: [logon("success", true), logon("failure", true)],
      }),
      option("On", { power_changes: [{ ...wake, ac: 1, dc: 0 }], audit_changes: [logon("success", false)] }),
    ],
    option("now", { power_changes: [{ ...wake, ac: 2, dc: 0 }], registry_changes: [reg({})] }),
  );
  assert.deepEqual(
    rows.map((r) => [r.kind, r.name]),
    [
      ["registry", "Flag"],
      ["power", "Allow wake timers"],
      ["audit", "Logon (success)"],
      ["audit", "Logon (failure)"],
    ],
  );
  const [, power, success, failure] = rows;
  assert.deepEqual(power.cells, [{ text: "Plugged in 0, on battery 0" }, { text: "Plugged in 1, on battery 0" }]);
  assert.deepEqual(power.now, { text: "Plugged in 2, on battery 0" });
  assert.deepEqual(success.cells, [{ text: "Audited" }, { text: "Not audited" }]);
  assert.deepEqual(failure.cells, [{ text: "Audited" }, null]);
});

test("the observed state fills the now column and can add a row", () => {
  const rows = buildMatrix(
    [option("On", { registry_changes: [reg({ value: 1 })] })],
    option("now", {
      registry_changes: [reg({ value: 0 })],
      service_changes: [{ name: "Svc", startup: "manual", skip_validation: false }],
    }),
  );
  assert.deepEqual(rows[0].now, { text: "0 (0x0)" });
  assert.deepEqual(rows[1].cells, [null]);
  assert.deepEqual(rows[1].now, { text: "Manual" });
});

test("optionsMatchingNow prefers the engine's single agreement", () => {
  const [row] = buildMatrix(
    [option("On", { registry_changes: [reg({ value: 1 })] }), option("Off", { registry_changes: [reg({ value: 0 })] })],
    option("now", { registry_changes: [reg({ value: 1 })] }),
  );
  const labels = ["On", "Off"];
  assert.deepEqual(optionsMatchingNow(row, labels, [{ effect: "e", name: "Flag [bits]", wanted_by: ["Off"] }]), [
    "Off",
  ]);
  // Ambiguous or missing agreement falls back to text comparison.
  const two = [
    { effect: "a", name: "Flag", wanted_by: ["Off"] },
    { effect: "b", name: "Flag", wanted_by: ["On"] },
  ];
  assert.deepEqual(optionsMatchingNow(row, labels, two), ["On"]);
  assert.deepEqual(optionsMatchingNow(row, labels, []), ["On"]);
  assert.deepEqual(optionsMatchingNow({ ...row, now: null }, labels, []), []);
});
