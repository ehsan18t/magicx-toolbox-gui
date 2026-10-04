import assert from "node:assert/strict";
import { test } from "node:test";
import { isAppCancelled, isAppExiting, isElevationDeclined, isUpdateFolderReadOnly } from "./error.ts";

const PREDICATES = {
  APP_CANCELLED: isAppCancelled,
  APP_EXITING: isAppExiting,
  ELEVATION_DECLINED: isElevationDeclined,
  UPDATE_FOLDER_READ_ONLY: isUpdateFolderReadOnly,
};

test("each error predicate matches its own backend code and nothing else", () => {
  for (const [code, predicate] of Object.entries(PREDICATES)) {
    for (const other of Object.keys(PREDICATES)) {
      assert.equal(predicate({ code: other, message: "m" }), other === code, `${predicate.name} on ${other}`);
    }
  }
});

test("a code without the backend's shape is not trusted", () => {
  for (const predicate of Object.values(PREDICATES)) {
    for (const value of [
      "UPDATE_FOLDER_READ_ONLY",
      null,
      { code: "UPDATE_FOLDER_READ_ONLY" },
      { code: "NOPE", message: "m" },
    ]) {
      assert.equal(predicate(value), false);
    }
  }
});
