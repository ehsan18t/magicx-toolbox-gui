//! Authoring-surface parsers: turn authored text into the typed model (spec §5.1 registry
//! paths, §5.2 packed-value fields, §6.2 value literals, §6.6 windows-version grammar). Pure
//! logic, no Windows API calls. The YAML binding layer (`schema.rs`) lands in Task 3 and maps
//! `serde_yaml_bw` nodes onto [`LiteralInput`] before calling these functions — that is also why
//! this module depends on nothing but `model` and std (`build.rs` includes it directly by path).

use super::model::{BuildExpr, Hive, PackedFormat, RegType, TypedRegValue, Value, WindowsScope};

/// Every rejection a tweak author can hit while authoring paths, literals, the `windows:` grammar,
/// or a packed value. Each message names the fix, since later tasks surface it verbatim.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("registry path {path:?} {reason}")]
    InvalidPath { path: String, reason: &'static str },

    #[error("value is null or empty — write `absent` to delete it, or supply a literal")]
    MissingValue,

    #[error("{raw:?} is not a valid {ty:?} literal — use a decimal or 0x-prefixed hex integer")]
    InvalidNumber { ty: RegType, raw: String },

    #[error(
        "{raw:?} is not a valid REG_BINARY literal — use hex byte pairs separated by commas or spaces, e.g. \"de,ad,be,ef\""
    )]
    InvalidBinary { raw: String },

    #[error("a {shape} is not valid for {ty:?}")]
    WrongLiteralShape { shape: &'static str, ty: RegType },

    #[error("presence values are only ever `present` or `absent`")]
    InvalidPresenceLiteral,

    #[error("{raw:?} is not a valid windows build expression — use N, >=N, <=N, or A..B")]
    InvalidBuildExpr { raw: String },

    #[error("`revision` requires `build` to pin a single exact build — add an exact `build` or drop `revision`")]
    RevisionWithoutExactBuild,

    #[error("{0} is not a supported windows product — use 10 or 11")]
    UnknownProduct(u8),

    #[error("{raw:?} is not a valid kv_semicolon value — expected `Name=Value;` pairs")]
    MalformedPacked { raw: String },
}

/// YAML-agnostic input to [`parse_value_literal`] (spec §6.2). Task 3 maps a `serde_yaml_bw` node
/// onto one of these variants by inspecting its *shape* before any string content is interpreted —
/// that is what keeps a bare reserved word and its `{ literal: ... }` escape distinguishable all
/// the way to this function.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralInput {
    /// An ordinary scalar string.
    Str(String),
    /// An ordinary scalar integer.
    Int(i64),
    /// A YAML list of strings — the only shape `REG_MULTI_SZ` accepts; `[]` clears it.
    List(Vec<String>),
    /// A bare scalar matching the reserved word `absent` — never string content.
    Reserved,
    /// The `{ literal: <text> }` escape: `<text>` is literal, even if it reads like a keyword.
    Escape(String),
    /// A YAML `null`, or a value entry with nothing after it (spec §6.2, ADR-0004).
    Null,
}

/// What domain a literal is being parsed into (spec §5.1): the reserved `absent` keyword compiles
/// to `Value::Absent` for a registry value/field, or `Present(false)` for a presence kind — one
/// authoring spelling, two typed outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralTarget {
    Reg(RegType),
    Presence,
}

/// Parses one value literal (spec §6.2). A bare `absent` is *always* the reserved keyword —
/// for every value type, including `Sz`/`ExpandSz` — because a type class with no deletion
/// spelling would be a hole, not a safeguard (invariant 13; ADR-0004 amended). An author who
/// wants the literal four-letter string uses the `{ literal: absent }` escape instead.
pub fn parse_value_literal(
    input: LiteralInput,
    target: LiteralTarget,
) -> Result<Value, ParseError> {
    match input {
        LiteralInput::Null => Err(ParseError::MissingValue),

        LiteralInput::Reserved => match target {
            LiteralTarget::Reg(_) => Ok(Value::Absent),
            LiteralTarget::Presence => Ok(Value::Present(false)),
        },

        LiteralInput::List(items) => match target {
            LiteralTarget::Reg(RegType::MultiSz) => Ok(Value::Reg(TypedRegValue::MultiSz(items))),
            LiteralTarget::Reg(ty) => Err(ParseError::WrongLiteralShape { shape: "list", ty }),
            LiteralTarget::Presence => Err(ParseError::InvalidPresenceLiteral),
        },

        // Past the Reserved/List/Null gates above, Str and an unwrapped Escape are the same
        // thing: literal text to render as `ty` dictates. That equivalence is the entire point
        // of the escape — it exists only to reach this arm instead of the Reserved one.
        LiteralInput::Str(text) | LiteralInput::Escape(text) => match target {
            LiteralTarget::Reg(ty) => Ok(Value::Reg(reg_scalar_from_str(&text, ty)?)),
            LiteralTarget::Presence => Err(ParseError::InvalidPresenceLiteral),
        },

        LiteralInput::Int(n) => match target {
            LiteralTarget::Reg(ty) => Ok(Value::Reg(reg_scalar_from_str(&n.to_string(), ty)?)),
            LiteralTarget::Presence => Err(ParseError::InvalidPresenceLiteral),
        },
    }
}

/// Builds a typed registry scalar from text (decimal/hex for numbers, verbatim for strings).
/// Shared by [`LiteralInput::Str`], [`LiteralInput::Escape`], and [`LiteralInput::Int`] (the
/// latter via its decimal string form) — one conversion regardless of which YAML shape supplied
/// the text.
fn reg_scalar_from_str(text: &str, ty: RegType) -> Result<TypedRegValue, ParseError> {
    match ty {
        RegType::Dword => {
            let value = parse_reg_int(text, ty)?;
            u32::try_from(value)
                .map(TypedRegValue::Dword)
                .map_err(|_| ParseError::InvalidNumber {
                    ty,
                    raw: text.to_string(),
                })
        }
        RegType::Qword => parse_reg_int(text, ty).map(TypedRegValue::Qword),
        RegType::Sz => Ok(TypedRegValue::Sz(text.to_string())),
        RegType::ExpandSz => Ok(TypedRegValue::ExpandSz(text.to_string())),
        RegType::Binary => parse_binary(text).map(TypedRegValue::Binary),
        RegType::MultiSz => Err(ParseError::WrongLiteralShape {
            shape: "string",
            ty,
        }),
    }
}

/// Decimal or `0x`/`0X`-prefixed hex, widened to `u64` — callers narrow for `DWORD`.
fn parse_reg_int(text: &str, ty: RegType) -> Result<u64, ParseError> {
    let hex_digits = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"));
    let parsed = match hex_digits {
        Some(digits) => u64::from_str_radix(digits, 16),
        None => text.parse::<u64>(),
    };
    parsed.map_err(|_| ParseError::InvalidNumber {
        ty,
        raw: text.to_string(),
    })
}

/// `.reg` hex-pair form (spec §6.2): comma- or space-separated byte pairs, e.g. `"de,ad,be,ef"`
/// or `"de ad be ef"`. No regex (spec §5.2) — deterministic split + per-token hex parse.
fn parse_binary(text: &str) -> Result<Vec<u8>, ParseError> {
    let invalid = || ParseError::InvalidBinary {
        raw: text.to_string(),
    };
    let tokens: Vec<&str> = if text.contains(',') {
        text.split(',').map(str::trim).collect()
    } else {
        text.split_whitespace().collect()
    };
    tokens
        .into_iter()
        .map(|token| {
            if token.len() != 2 {
                return Err(invalid());
            }
            u8::from_str_radix(token, 16).map_err(|_| invalid())
        })
        .collect()
}

/// Parses a merged `HIVE\key\...` registry path (spec §5.1). v1 hives: HKLM and HKCU, short or
/// long spelling, normalized.
pub fn parse_reg_path(raw: &str) -> Result<(Hive, String), ParseError> {
    if raw.contains('/') {
        return Err(ParseError::InvalidPath {
            path: raw.to_string(),
            reason: "uses a forward slash — registry paths use backslashes only",
        });
    }

    let segments: Vec<&str> = raw.split('\\').collect();
    if segments.len() < 2 {
        return Err(ParseError::InvalidPath {
            path: raw.to_string(),
            reason: "has no key path — expected HIVE\\Key\\..., e.g. HKLM\\Software\\...",
        });
    }
    if segments[0].is_empty() {
        return Err(ParseError::InvalidPath {
            path: raw.to_string(),
            reason: "starts with a backslash — remove the leading backslash",
        });
    }
    if segments.last().is_some_and(|s| s.is_empty()) {
        return Err(ParseError::InvalidPath {
            path: raw.to_string(),
            reason: "ends with a backslash — remove the trailing backslash",
        });
    }
    if segments[1..].iter().any(|s| s.is_empty()) {
        return Err(ParseError::InvalidPath {
            path: raw.to_string(),
            reason: "contains an empty segment — check for a doubled backslash",
        });
    }

    let hive = match segments[0] {
        "HKLM" | "HKEY_LOCAL_MACHINE" => Hive::Hklm,
        "HKCU" | "HKEY_CURRENT_USER" => Hive::Hkcu,
        _ => {
            return Err(ParseError::InvalidPath {
                path: raw.to_string(),
                reason: "does not start with a supported hive — use HKLM or HKCU (short or long spelling)",
            });
        }
    };
    Ok((hive, segments[1..].join("\\")))
}

/// Parses the `build`/`revision` grammar shared by both `windows:` fields (spec §6.6):
/// `N | >=N | <=N | A..B` (inclusive).
pub fn parse_build_expr(raw: &str) -> Result<BuildExpr, ParseError> {
    let invalid = || ParseError::InvalidBuildExpr {
        raw: raw.to_string(),
    };
    if let Some(rest) = raw.strip_prefix(">=") {
        return rest.parse().map(BuildExpr::Min).map_err(|_| invalid());
    }
    if let Some(rest) = raw.strip_prefix("<=") {
        return rest.parse().map(BuildExpr::Max).map_err(|_| invalid());
    }
    if let Some((lo, hi)) = raw.split_once("..") {
        let lo: u32 = lo.parse().map_err(|_| invalid())?;
        let hi: u32 = hi.parse().map_err(|_| invalid())?;
        return Ok(BuildExpr::Range(lo, hi));
    }
    raw.parse().map(BuildExpr::Exact).map_err(|_| invalid())
}

/// The one cross-field rule for `windows:` (spec §6.6): `revision` only makes sense pinned to a
/// single exact `build`, since the revision/UBR counter resets per build line. Assembling the rest
/// of the `windows:` block from YAML is Task 3's job.
pub fn validate_windows_scope(scope: &WindowsScope) -> Result<(), ParseError> {
    if scope.revision.is_some() && !matches!(scope.build, Some(BuildExpr::Exact(_))) {
        return Err(ParseError::RevisionWithoutExactBuild);
    }
    Ok(())
}

/// Expands the `products` sugar to the build range it stands for (spec §6.6: `10 = 10240..19045`,
/// `11 = >=22000`).
pub fn expand_product(product: u8) -> Result<BuildExpr, ParseError> {
    match product {
        10 => Ok(BuildExpr::Range(10_240, 19_045)),
        11 => Ok(BuildExpr::Min(22_000)),
        other => Err(ParseError::UnknownProduct(other)),
    }
}

/// The fields of a packed registry value, in the exact order they were read. Spec §5.2 requires
/// unknown fields and their order to survive a parse → upsert → serialize round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedFields(Vec<(String, String)>);

impl PackedFields {
    /// The current value of one field, if present.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(name, _)| name == field)
            .map(|(_, value)| value.as_str())
    }

    /// Sets a field's value in place, or appends it if not already present. Never reorders
    /// existing fields (spec §5.2).
    pub fn upsert(&mut self, field: &str, value: &str) {
        match self.0.iter_mut().find(|(name, _)| name == field) {
            Some(entry) => entry.1 = value.to_string(),
            None => self.0.push((field.to_string(), value.to_string())),
        }
    }

    /// Deletes one field (the `absent` field keyword), leaving the others untouched.
    pub fn remove(&mut self, field: &str) {
        self.0.retain(|(name, _)| name != field);
    }
}

/// Parses a packed value's live string into its fields (spec §5.2). A live string the parser
/// cannot understand is a typed error — never a partial or guessed result.
pub fn parse_packed(format: PackedFormat, live: &str) -> Result<PackedFields, ParseError> {
    match format {
        PackedFormat::KvSemicolon => parse_kv_semicolon(live),
    }
}

/// `Name=Value;` pairs, one well-formed segment per `;`. A single trailing `;` is the normal
/// terminator; anything else that doesn't split into exactly one non-empty name and one value
/// (a stray `;;`, a missing `=`, a doubled `=`) is malformed — the whole parse fails rather than
/// guess at a partial reading (spec §5.2).
fn parse_kv_semicolon(live: &str) -> Result<PackedFields, ParseError> {
    let mut segments: Vec<&str> = live.split(';').collect();
    if segments.last().is_some_and(|s| s.is_empty()) {
        segments.pop();
    }

    let mut fields = Vec::with_capacity(segments.len());
    for segment in segments {
        let malformed = || ParseError::MalformedPacked {
            raw: live.to_string(),
        };
        let Some((name, value)) = segment.split_once('=') else {
            return Err(malformed());
        };
        if name.is_empty() || value.contains('=') {
            return Err(malformed());
        }
        fields.push((name.to_string(), value.to_string()));
    }
    Ok(PackedFields(fields))
}

/// Serializes fields back to a packed value's live string, in their current order.
pub fn serialize_packed(format: PackedFormat, fields: &PackedFields) -> String {
    match format {
        PackedFormat::KvSemicolon => fields
            .0
            .iter()
            .map(|(name, value)| format!("{name}={value};"))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- §5.1 registry paths ---------------------------------------------------------------

    #[test]
    fn accepts_hklm_short_and_long_spelling_normalized_equal() {
        let short = parse_reg_path(r"HKLM\A\B").expect("short HKLM path should parse");
        let long = parse_reg_path(r"HKEY_LOCAL_MACHINE\A\B").expect("long HKLM path should parse");
        assert_eq!(short, long);
        assert_eq!(short, (Hive::Hklm, "A\\B".to_string()));
    }

    #[test]
    fn accepts_hkcu_short_and_long_spelling_normalized_equal() {
        let short = parse_reg_path(r"HKCU\A\B").expect("short HKCU path should parse");
        let long = parse_reg_path(r"HKEY_CURRENT_USER\A\B").expect("long HKCU path should parse");
        assert_eq!(short, long);
        assert_eq!(short, (Hive::Hkcu, "A\\B".to_string()));
    }

    #[test]
    fn rejects_malformed_paths() {
        let cases: &[(&str, &str)] = &[
            (r"\HKLM\A\B", "starts with a backslash"),
            (r"HKLM\A\B\", "ends with a backslash"),
            (r"HKLM\A\\B", "empty segment"),
            (r"HKLM/A/B", "forward slash"),
            (r"HKLM", "no key path"),
            (r"HKCR\A\B", "HKLM or HKCU"),
        ];
        for (raw, expect_substring) in cases {
            let err = parse_reg_path(raw).expect_err(&format!("{raw:?} must be rejected"));
            let message = err.to_string();
            assert!(
                message.contains(expect_substring),
                "error for {raw:?} was {message:?}, expected it to mention {expect_substring:?}"
            );
        }
    }

    // --- §6.2 value literals -----------------------------------------------------------------

    #[test]
    fn dword_accepts_decimal_and_hex_equal() {
        let decimal = parse_value_literal(LiteralInput::Int(1), LiteralTarget::Reg(RegType::Dword))
            .expect("decimal DWORD should parse");
        let hex = parse_value_literal(
            LiteralInput::Str("0x1".to_string()),
            LiteralTarget::Reg(RegType::Dword),
        )
        .expect("hex DWORD should parse");
        assert_eq!(decimal, hex);
        assert_eq!(decimal, Value::Reg(TypedRegValue::Dword(1)));
    }

    #[test]
    fn dword_rejects_value_above_u32_max() {
        let result = parse_value_literal(
            LiteralInput::Int(5_000_000_000),
            LiteralTarget::Reg(RegType::Dword),
        );
        assert!(result.is_err(), "DWORD must reject values above u32::MAX");
    }

    #[test]
    fn qword_accepts_value_above_u32_max() {
        let value = parse_value_literal(
            LiteralInput::Int(5_000_000_000),
            LiteralTarget::Reg(RegType::Qword),
        )
        .expect("QWORD should accept values above u32::MAX");
        assert_eq!(value, Value::Reg(TypedRegValue::Qword(5_000_000_000)));
    }

    #[test]
    fn binary_accepts_comma_and_space_forms_equal() {
        let comma = parse_value_literal(
            LiteralInput::Str("de,ad,be,ef".to_string()),
            LiteralTarget::Reg(RegType::Binary),
        )
        .expect("comma-separated BINARY should parse");
        let space = parse_value_literal(
            LiteralInput::Str("de ad be ef".to_string()),
            LiteralTarget::Reg(RegType::Binary),
        )
        .expect("space-separated BINARY should parse");
        assert_eq!(comma, space);
        assert_eq!(
            comma,
            Value::Reg(TypedRegValue::Binary(vec![0xde, 0xad, 0xbe, 0xef]))
        );
    }

    #[test]
    fn binary_rejects_odd_length_pair() {
        let result = parse_value_literal(
            LiteralInput::Str("de,ad,be,f".to_string()),
            LiteralTarget::Reg(RegType::Binary),
        );
        assert!(result.is_err(), "an odd-length hex token must be rejected");
    }

    #[test]
    fn multi_sz_from_yaml_list() {
        let value = parse_value_literal(
            LiteralInput::List(vec!["a".to_string(), "b".to_string()]),
            LiteralTarget::Reg(RegType::MultiSz),
        )
        .expect("list should parse as MULTI_SZ");
        assert_eq!(
            value,
            Value::Reg(TypedRegValue::MultiSz(vec![
                "a".to_string(),
                "b".to_string()
            ]))
        );
    }

    #[test]
    fn multi_sz_empty_list_clears() {
        let value = parse_value_literal(
            LiteralInput::List(vec![]),
            LiteralTarget::Reg(RegType::MultiSz),
        )
        .expect("empty list should parse");
        assert_eq!(value, Value::Reg(TypedRegValue::MultiSz(vec![])));
    }

    #[test]
    fn reserved_absent_is_value_absent_for_every_reg_type() {
        for ty in [
            RegType::Dword,
            RegType::Qword,
            RegType::Sz,
            RegType::ExpandSz,
            RegType::Binary,
            RegType::MultiSz,
        ] {
            let value = parse_value_literal(LiteralInput::Reserved, LiteralTarget::Reg(ty))
                .unwrap_or_else(|e| panic!("absent should resolve for {ty:?}: {e}"));
            assert_eq!(
                value,
                Value::Absent,
                "absent should resolve to Value::Absent for {ty:?}"
            );
        }
    }

    #[test]
    fn reserved_absent_is_present_false_for_presence_target() {
        let value = parse_value_literal(LiteralInput::Reserved, LiteralTarget::Presence)
            .expect("absent should resolve for a presence kind");
        assert_eq!(value, Value::Present(false));
    }

    #[test]
    fn escape_literal_absent_produces_sz_absent_string() {
        let value = parse_value_literal(
            LiteralInput::Escape("absent".to_string()),
            LiteralTarget::Reg(RegType::Sz),
        )
        .expect("the literal escape should parse as ordinary string content");
        assert_eq!(value, Value::Reg(TypedRegValue::Sz("absent".to_string())));
    }

    /// Spec §6.2 / ADR-0004 (amended 2026-07-22, corrected): a bare `absent` is *always* the
    /// reserved keyword, for every value type including `REG_SZ`/`REG_EXPAND_SZ` — a type class
    /// with no deletion spelling would be a hole, not a safeguard (invariant 13). The
    /// `{ literal: absent }` escape is how an author gets the literal string instead; see
    /// `escape_literal_absent_produces_sz_absent_string`.
    #[test]
    fn bare_reserved_word_on_string_types_is_value_absent() {
        for ty in [RegType::Sz, RegType::ExpandSz] {
            let value = parse_value_literal(LiteralInput::Reserved, LiteralTarget::Reg(ty))
                .unwrap_or_else(|e| panic!("bare absent on {ty:?} should resolve: {e}"));
            assert_eq!(
                value,
                Value::Absent,
                "bare absent on {ty:?} should resolve to Value::Absent"
            );
        }
    }

    #[test]
    fn null_scalar_is_rejected_naming_absent() {
        for target in [LiteralTarget::Reg(RegType::Sz), LiteralTarget::Presence] {
            let err = parse_value_literal(LiteralInput::Null, target)
                .expect_err("a null/empty scalar must be rejected");
            assert!(
                err.to_string().contains("absent"),
                "error was {err}, expected it to name `absent`"
            );
        }
    }

    // --- §6.6 windows-version applicability ---------------------------------------------------

    #[test]
    fn build_expr_parses_exact_min_max_range() {
        assert_eq!(parse_build_expr("26100").unwrap(), BuildExpr::Exact(26100));
        assert_eq!(parse_build_expr(">=26100").unwrap(), BuildExpr::Min(26100));
        assert_eq!(parse_build_expr("<=26100").unwrap(), BuildExpr::Max(26100));
        assert_eq!(
            parse_build_expr("26100..27200").unwrap(),
            BuildExpr::Range(26100, 27200)
        );
    }

    #[test]
    fn build_expr_rejects_garbage() {
        assert!(parse_build_expr("not-a-number").is_err());
    }

    #[test]
    fn products_expand_to_build_ranges() {
        assert_eq!(expand_product(10).unwrap(), BuildExpr::Range(10240, 19045));
        assert_eq!(expand_product(11).unwrap(), BuildExpr::Min(22000));
    }

    #[test]
    fn unsupported_product_is_rejected() {
        assert!(expand_product(7).is_err());
    }

    #[test]
    fn revision_without_exact_build_is_rejected() {
        let no_build = WindowsScope {
            products: None,
            build: None,
            revision: Some(BuildExpr::Exact(5)),
        };
        assert!(validate_windows_scope(&no_build).is_err());

        let ranged_build = WindowsScope {
            products: None,
            build: Some(BuildExpr::Min(26100)),
            revision: Some(BuildExpr::Exact(5)),
        };
        assert!(validate_windows_scope(&ranged_build).is_err());
    }

    #[test]
    fn revision_with_exact_build_is_ok() {
        let scope = WindowsScope {
            products: None,
            build: Some(BuildExpr::Exact(26100)),
            revision: Some(BuildExpr::Min(2314)),
        };
        assert!(validate_windows_scope(&scope).is_ok());
    }

    #[test]
    fn empty_scope_is_unconstrained() {
        let scope = WindowsScope {
            products: None,
            build: None,
            revision: None,
        };
        assert!(validate_windows_scope(&scope).is_ok());
    }

    // --- §5.2 packed-value field addressing ---------------------------------------------------

    #[test]
    fn parses_fields_in_order() {
        let fields = parse_packed(PackedFormat::KvSemicolon, "A=1;B=2;")
            .expect("well-formed input should parse");
        assert_eq!(fields.get("A"), Some("1"));
        assert_eq!(fields.get("B"), Some("2"));
        assert_eq!(
            serialize_packed(PackedFormat::KvSemicolon, &fields),
            "A=1;B=2;"
        );
    }

    #[test]
    fn upsert_existing_field_preserves_others_and_order() {
        let mut fields = parse_packed(PackedFormat::KvSemicolon, "A=1;B=2;").unwrap();
        fields.upsert("B", "3");
        assert_eq!(fields.get("A"), Some("1"));
        assert_eq!(fields.get("B"), Some("3"));
        assert_eq!(
            serialize_packed(PackedFormat::KvSemicolon, &fields),
            "A=1;B=3;"
        );
    }

    #[test]
    fn upsert_new_field_appends() {
        let mut fields = parse_packed(PackedFormat::KvSemicolon, "A=1;B=2;").unwrap();
        fields.upsert("C", "9");
        assert_eq!(
            serialize_packed(PackedFormat::KvSemicolon, &fields),
            "A=1;B=2;C=9;"
        );
    }

    #[test]
    fn remove_deletes_only_that_field() {
        let mut fields = parse_packed(PackedFormat::KvSemicolon, "A=1;B=2;").unwrap();
        fields.remove("A");
        assert_eq!(fields.get("A"), None);
        assert_eq!(fields.get("B"), Some("2"));
        assert_eq!(serialize_packed(PackedFormat::KvSemicolon, &fields), "B=2;");
    }

    #[test]
    fn garbage_packed_input_is_a_typed_error_never_partial() {
        let result = parse_packed(PackedFormat::KvSemicolon, "no-separators==;;");
        assert!(
            result.is_err(),
            "malformed packed input must never produce a partial result"
        );
    }
}
