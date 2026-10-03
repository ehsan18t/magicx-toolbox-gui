import assert from "node:assert/strict";
import { test } from "node:test";
import type { ConfigurationProfile, ProfileApplyResult, ProfileValidation } from "$lib/types";
import { wizardStep } from "./profileWizard.ts";

const profile = {} as ConfigurationProfile;
const validation = {} as ProfileValidation;
const result = {} as ProfileApplyResult;

test("wizardStep starts at select until both profile and validation are loaded", () => {
  assert.equal(wizardStep(false, null, null, null), "select");
  assert.equal(wizardStep(false, null, profile, null), "select");
  assert.equal(wizardStep(false, null, null, validation), "select");
  assert.equal(wizardStep(false, null, profile, validation), "review");
});

test("wizardStep: applying outranks a result, a result outranks review", () => {
  assert.equal(wizardStep(true, result, profile, validation), "applying");
  assert.equal(wizardStep(false, result, profile, validation), "result");
  assert.equal(wizardStep(false, result, null, null), "result");
});
