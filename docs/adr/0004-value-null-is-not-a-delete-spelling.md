---
status: accepted
---

# `value: null` is not a way to delete a registry value

`action: delete_value` and `action: delete_key` are the only ways to remove something from the registry. A `set` action with no value is a build error, and stays one. The authoring guide's appendix previously claimed `value: null` deletes the value; no such code path has ever existed, and the claim is being removed rather than implemented.

## Why not implement it

YAML deserialization maps `value: null`, an omitted `value:`, and a bare `value:` with nothing after it to the **same** absent value. Distinguishing them requires a custom deserializer. That buys a second, redundant spelling of `delete_value` — and an actively dangerous one, because an author who forgets to fill in a value would get a silent delete instead of a build error.

## Consequences

The build error for a valueless `set` should name the alternatives (`action: delete_value` / `action: delete_key`) rather than just stating the requirement, since the same message is what an author sees when they leave a `value:` line unfilled. Note that quoted `value: ""` is a legitimate empty string and must keep working — only the *absent* case errors.

## Amended 2026-07-22 (tweak-system redesign, spec rev 2)

The **principle stands unchanged**: absence must be an explicit, deliberate spelling, and a forgotten
value must be a build error — never a silent delete. The **spelling evolves** with the effect-centric
schema:

- `action: delete_value` and `action: delete_key`/`create_key` **no longer exist**. Deletion is now a
  *driven state*: the reserved keyword **`absent`** is the only absence spelling, uniform at all three
  depths — registry **value** (`some_effect: absent` deletes the value), registry **key** (the
  `registry_key` presence Setting), and packed-value **field** (removes the field). This makes
  deletion reversible and detectable by construction instead of an opaque action. The one structural
  action that remains is delete-**tree**, explicitly one-way unless the author supplies `undo`.
- The YAML-null hazard this ADR closed stays closed: `null`, an omitted entry, and a bare value are
  still build errors (the coverage guard independently rejects omission — every option must value
  every Setting effect). The error message now names **`absent`** as the alternative.
- A bare `absent` is **always** the keyword, for every value type — including `REG_SZ` — because a
  type class with no deletion spelling would be a hole, not a safeguard. The escape
  `{ literal: absent }` exists for the rare string whose content really is that word. `value: ""`
  remains a legitimate empty string. The hazard this ADR closes is the *forgotten* value
  (`null`/omitted/bare), which stays a build error; deliberately typing `absent` was never that
  hazard.
