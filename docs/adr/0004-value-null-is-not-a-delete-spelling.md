---
status: accepted
---

# `value: null` is not a way to delete a registry value

`action: delete_value` and `action: delete_key` are the only ways to remove something from the registry. A `set` action with no value is a build error, and stays one. The authoring guide's appendix previously claimed `value: null` deletes the value; no such code path has ever existed, and the claim is being removed rather than implemented.

## Why not implement it

YAML deserialization maps `value: null`, an omitted `value:`, and a bare `value:` with nothing after it to the **same** absent value. Distinguishing them requires a custom deserializer. That buys a second, redundant spelling of `delete_value` — and an actively dangerous one, because an author who forgets to fill in a value would get a silent delete instead of a build error.

## Consequences

The build error for a valueless `set` should name the alternatives (`action: delete_value` / `action: delete_key`) rather than just stating the requirement, since the same message is what an author sees when they leave a `value:` line unfilled. Note that quoted `value: ""` is a legitimate empty string and must keep working — only the *absent* case errors.
