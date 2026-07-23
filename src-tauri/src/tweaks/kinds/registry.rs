//! `EffectKind` for `Setting::Registry` (one value, optionally one packed field) and
//! `Setting::RegistryKey` (key existence) — spec §5.1/§5.2. Wraps the low-level
//! `services::registry_service` primitive, translating between the new typed model
//! (`Hive`/`RegType`/`TypedRegValue`) and the old (`RegistryHive`/`RegistryValueType`) that
//! primitive still speaks.

use std::sync::Mutex;

use crate::error::Error as BackendError;
use crate::models::{RegistryHive, RegistryValueType};
use crate::services::elevation::BrokerOp;
use crate::services::registry_service;
use crate::tweaks::model::{
    FieldAddr, Hive, KeyAddr, Level, RegAddr, RegType, Setting, TypedRegValue, Value,
};
use crate::tweaks::parse::{self, PackedFields};

use super::{EffectKind, Error, ExecCx};

/// Serializes packed-value read-modify-write cycles process-wide (spec §5.2). A per-tweak lock
/// would not be enough: two different tweaks can each own a different field of the *same* packed
/// value, so the critical section has to be keyed on nothing narrower than the whole process.
static FIELD_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// `EffectKind` for `Setting::Registry` and `Setting::RegistryKey`.
pub struct RegistryKind;

impl EffectKind for RegistryKind {
    fn read(&self, s: &Setting, _cx: &ExecCx) -> Result<Value, Error> {
        // Reads never escalate (spec invariant 24: "reads run at the current level"; the broker
        // protocol has no read op) -- `cx` is unused here on purpose; level only gates `drive`.
        match s {
            Setting::Registry(addr) => read_value(addr),
            Setting::RegistryKey(addr) => read_key(addr),
            Setting::Service(_) | Setting::Task(_) | Setting::Hosts(_) | Setting::Firewall(_) => {
                Err(Error::Invalid("RegistryKind cannot read this Setting"))
            }
        }
    }

    fn drive(&self, s: &Setting, target: &Value, cx: &ExecCx) -> Result<(), Error> {
        guard_level(cx)?;
        match s {
            Setting::Registry(addr) => drive_value(addr, target),
            Setting::RegistryKey(addr) => drive_key(addr, target),
            Setting::Service(_) | Setting::Task(_) | Setting::Hosts(_) | Setting::Firewall(_) => {
                Err(Error::Invalid("RegistryKind cannot drive this Setting"))
            }
        }
    }
}

/// `Level::User`/`Level::Admin` run in-process here; `System`/`Ti` are routed through the
/// elevation broker one layer up, by `engine::AllKinds::drive` (see `to_broker_op` below and the
/// `kinds` module docs for the placement) -- this kind's own `drive` (called directly, bypassing
/// that routing) still rejects them itself.
fn guard_level(cx: &ExecCx) -> Result<(), Error> {
    match cx.level() {
        Level::User | Level::Admin => Ok(()),
        other => Err(Error::UnsupportedLevel(other)),
    }
}

// --- hive / type conversions between the new typed model and the old primitive's types --------

fn old_hive(hive: Hive) -> RegistryHive {
    match hive {
        Hive::Hklm => RegistryHive::Hklm,
        Hive::Hkcu => RegistryHive::Hkcu,
    }
}

fn old_type(ty: RegType) -> RegistryValueType {
    match ty {
        RegType::Dword => RegistryValueType::Dword,
        RegType::Qword => RegistryValueType::Qword,
        RegType::Sz => RegistryValueType::String,
        RegType::ExpandSz => RegistryValueType::ExpandString,
        RegType::MultiSz => RegistryValueType::MultiString,
        RegType::Binary => RegistryValueType::Binary,
    }
}

fn new_type(ty: RegistryValueType) -> RegType {
    match ty {
        RegistryValueType::Dword => RegType::Dword,
        RegistryValueType::Qword => RegType::Qword,
        RegistryValueType::String => RegType::Sz,
        RegistryValueType::ExpandString => RegType::ExpandSz,
        RegistryValueType::MultiString => RegType::MultiSz,
        RegistryValueType::Binary => RegType::Binary,
    }
}

/// Maps the backend primitive's error onto ours, preserving the distinctions the "did-it-work"
/// contract requires (spec invariant 2) instead of flattening everything to a string.
fn backend(e: BackendError) -> Error {
    match e {
        BackendError::RegistryKeyNotFound(msg) => Error::KeyNotFound(msg),
        BackendError::RegistryAccessDenied(msg) => Error::AccessDenied(msg),
        BackendError::RequiresAdmin => {
            Error::AccessDenied("requires administrator privileges".to_string())
        }
        other => Error::Backend(other.to_string()),
    }
}

// --- Setting::Registry: reads -------------------------------------------------------------------

/// Reads the raw underlying value, typed against `addr.ty` (spec §5.1: nonexistent -> `None`; a
/// stored type that disagrees with `addr.ty` -> `Err(TypeMismatch)`, never a fake absence).
/// `detect_value_type` already collapses "key missing" and "value missing" into `None` for us.
fn read_raw(addr: &RegAddr) -> Result<Option<TypedRegValue>, Error> {
    let hive = old_hive(addr.hive);
    let Some(actual) =
        registry_service::detect_value_type(&hive, &addr.path, &addr.name).map_err(backend)?
    else {
        return Ok(None);
    };
    let expected = old_type(addr.ty);
    if actual != expected {
        return Err(Error::TypeMismatch {
            path: addr.path.clone(),
            name: addr.name.clone(),
            expected: addr.ty,
            actual: new_type(actual),
        });
    }
    let value = match addr.ty {
        RegType::Dword => registry_service::read_dword(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::Dword),
        RegType::Qword => registry_service::read_qword(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::Qword),
        RegType::Sz => registry_service::read_string(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::Sz),
        RegType::ExpandSz => registry_service::read_string(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::ExpandSz),
        RegType::MultiSz => registry_service::read_multi_string(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::MultiSz),
        RegType::Binary => registry_service::read_binary(&hive, &addr.path, &addr.name)
            .map_err(backend)?
            .map(TypedRegValue::Binary),
    };
    Ok(value)
}

/// The packed live-text behind a field address, or `""` when the whole value is absent (parses
/// cleanly to an empty [`PackedFields`] -- a fresh packed value starts life as no fields at all).
fn packed_text(addr: &RegAddr) -> Result<String, Error> {
    match read_raw(addr)? {
        None => Ok(String::new()),
        Some(TypedRegValue::Sz(s) | TypedRegValue::ExpandSz(s)) => Ok(s),
        Some(_) => Err(Error::Invalid(
            "a packed field address must declare REG_SZ or REG_EXPAND_SZ",
        )),
    }
}

fn parse_packed(addr: &RegAddr, field: &FieldAddr, live: &str) -> Result<PackedFields, Error> {
    parse::parse_packed(field.format, live).map_err(|source| Error::MalformedPacked {
        path: addr.path.clone(),
        name: addr.name.clone(),
        source,
    })
}

fn read_value(addr: &RegAddr) -> Result<Value, Error> {
    match &addr.field {
        None => Ok(read_raw(addr)?.map_or(Value::Absent, Value::Reg)),
        Some(field) => {
            let live = packed_text(addr)?;
            let fields = parse_packed(addr, field, &live)?;
            Ok(fields.get(&field.field).map_or(Value::Absent, |v| {
                Value::Reg(TypedRegValue::Sz(v.to_string()))
            }))
        }
    }
}

fn read_key(addr: &KeyAddr) -> Result<Value, Error> {
    let hive = old_hive(addr.hive);
    let exists = registry_service::key_exists(&hive, &addr.path).map_err(backend)?;
    Ok(Value::Present(exists))
}

// --- Setting::Registry: drives ------------------------------------------------------------------

fn drive_value(addr: &RegAddr, target: &Value) -> Result<(), Error> {
    match &addr.field {
        None => drive_whole_value(addr, target),
        Some(field) => drive_field(addr, field, target),
    }
}

fn drive_whole_value(addr: &RegAddr, target: &Value) -> Result<(), Error> {
    let hive = old_hive(addr.hive);
    match target {
        Value::Absent => delete_ok(registry_service::delete_value(
            &hive, &addr.path, &addr.name,
        ))
        .map_err(backend),
        Value::Reg(v) => write_typed(&hive, &addr.path, &addr.name, v),
        _ => Err(Error::Invalid(
            "a registry value can only be driven to a Reg literal or Absent",
        )),
    }
}

fn write_typed(
    hive: &RegistryHive,
    path: &str,
    name: &str,
    v: &TypedRegValue,
) -> Result<(), Error> {
    match v {
        TypedRegValue::Dword(n) => registry_service::set_dword(hive, path, name, *n),
        TypedRegValue::Qword(n) => registry_service::set_qword(hive, path, name, *n),
        TypedRegValue::Sz(s) => registry_service::set_string(hive, path, name, s),
        TypedRegValue::ExpandSz(s) => registry_service::set_expand_string(hive, path, name, s),
        TypedRegValue::MultiSz(items) => {
            registry_service::set_multi_string(hive, path, name, items)
        }
        TypedRegValue::Binary(bytes) => registry_service::set_binary(hive, path, name, bytes),
    }
    .map_err(backend)
}

/// A missing key/value is already the target state for a delete — idempotent, not a failure
/// (mirrors `services::elevation::broker`'s own `delete_ok`).
fn delete_ok(result: Result<(), BackendError>) -> Result<(), BackendError> {
    match result {
        Err(BackendError::RegistryKeyNotFound(_)) => Ok(()),
        other => other,
    }
}

// --- System/TI routing: Setting + Value -> BrokerOp (spec §9; see kinds/mod.rs's module docs for
// WHERE this is called from -- `engine::AllKinds::drive`, never this file's own `drive`) ---------

/// Translates a System/TI-level registry drive into the broker's typed op (spec §9): the
/// mechanical `RegAddr`/`KeyAddr` + `Value` -> `BrokerOp` mapping. A field-addressed write is a
/// read-modify-write cycle (`drive_field`) this task does not route -- the read half would need to
/// run at the SAME elevated level to see the true live value, and the broker wire protocol has no
/// read op at all -- so it stays `Error::UnsupportedLevel`, a strict narrowing of what was
/// previously every System/Ti registry drive, not a new gap.
pub(crate) fn to_broker_op(s: &Setting, target: &Value, level: Level) -> Result<BrokerOp, Error> {
    match s {
        Setting::Registry(addr) if addr.field.is_some() => Err(Error::UnsupportedLevel(level)),
        Setting::Registry(addr) => {
            let hive = old_hive(addr.hive);
            match target {
                Value::Absent => Ok(BrokerOp::RegDeleteValue {
                    hive,
                    key: addr.path.clone(),
                    value_name: addr.name.clone(),
                }),
                Value::Reg(v) => Ok(BrokerOp::RegSet {
                    hive,
                    key: addr.path.clone(),
                    value_name: addr.name.clone(),
                    value_type: old_type(addr.ty),
                    value: broker_reg_value(v),
                }),
                _ => Err(Error::Invalid(
                    "a registry value can only be driven to a Reg literal or Absent",
                )),
            }
        }
        Setting::RegistryKey(addr) => {
            let hive = old_hive(addr.hive);
            match target {
                Value::Present(true) => Ok(BrokerOp::RegCreateKey {
                    hive,
                    key: addr.path.clone(),
                }),
                Value::Present(false) => Ok(BrokerOp::RegDeleteKey {
                    hive,
                    key: addr.path.clone(),
                }),
                _ => Err(Error::Invalid(
                    "a registry key can only be driven to Present(bool)",
                )),
            }
        }
        Setting::Service(_) | Setting::Task(_) | Setting::Hosts(_) | Setting::Firewall(_) => {
            Err(Error::Invalid("RegistryKind cannot drive this Setting"))
        }
    }
}

/// The broker's `RegSet` carries its value as untyped JSON (spec: the wire protocol is kind-
/// neutral); this is the same per-type encoding `write_typed`'s callee ultimately stores, just
/// expressed as JSON rather than a direct Win32 call.
fn broker_reg_value(v: &TypedRegValue) -> serde_json::Value {
    match v {
        TypedRegValue::Dword(n) => serde_json::json!(n),
        TypedRegValue::Qword(n) => serde_json::json!(n),
        TypedRegValue::Sz(s) | TypedRegValue::ExpandSz(s) => serde_json::json!(s),
        TypedRegValue::MultiSz(items) => serde_json::json!(items),
        TypedRegValue::Binary(bytes) => serde_json::json!(bytes),
    }
}

fn drive_key(addr: &KeyAddr, target: &Value) -> Result<(), Error> {
    let hive = old_hive(addr.hive);
    match target {
        Value::Present(true) => registry_service::create_key(&hive, &addr.path).map_err(backend),
        Value::Present(false) => {
            delete_ok(registry_service::delete_key(&hive, &addr.path)).map_err(backend)
        }
        _ => Err(Error::Invalid(
            "a registry key can only be driven to Present(bool)",
        )),
    }
}

/// Field writes are read-modify-write (spec §5.2): read the packed string, upsert/remove just the
/// addressed field, re-serialize, write back -- behind [`FIELD_WRITE_LOCK`] so two concurrent
/// field writes on the same value can never race each other's read.
fn drive_field(addr: &RegAddr, field: &FieldAddr, target: &Value) -> Result<(), Error> {
    let _guard = FIELD_WRITE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let live = packed_text(addr)?;
    let mut fields = parse_packed(addr, field, &live)?;

    match target {
        Value::Absent => fields.remove(&field.field),
        Value::Reg(TypedRegValue::Sz(text)) => fields.upsert(&field.field, text),
        _ => {
            return Err(Error::Invalid(
                "a packed field can only be driven to a plain string or Absent",
            ));
        }
    }

    let serialized = parse::serialize_packed(field.format, &fields);
    let wrapped = match addr.ty {
        RegType::Sz => TypedRegValue::Sz(serialized),
        RegType::ExpandSz => TypedRegValue::ExpandSz(serialized),
        _ => {
            return Err(Error::Invalid(
                "a packed field address must declare REG_SZ or REG_EXPAND_SZ",
            ));
        }
    };
    write_typed(&old_hive(addr.hive), &addr.path, &addr.name, &wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweaks::model::PackedFormat;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A unique HKCU scratch subtree that deletes itself on drop, even on panic, so parallel
    /// tests never collide and a failed assertion never leaks a key.
    struct Scratch {
        path: String,
    }
    impl Scratch {
        fn new(label: &str) -> Self {
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            Scratch {
                path: format!(
                    "Software\\MagicXToolboxTest\\kindreg_{label}_{}_{n}",
                    std::process::id()
                ),
            }
        }

        fn reg_addr(&self, name: &str, ty: RegType) -> RegAddr {
            RegAddr {
                hive: Hive::Hkcu,
                path: self.path.clone(),
                name: name.to_string(),
                ty,
                field: None,
            }
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = registry_service::delete_key(&RegistryHive::Hkcu, &self.path);
        }
    }

    fn user_cx() -> ExecCx {
        ExecCx::new(Level::User)
    }

    #[test]
    fn read_distinguishes_notfound_typemismatch() {
        let scratch = Scratch::new("notfound_typemismatch");
        let cx = user_cx();
        let addr = scratch.reg_addr("Flag", RegType::Dword);

        let absent = RegistryKind.read(&Setting::Registry(addr), &cx).unwrap();
        assert_eq!(absent, Value::Absent);

        registry_service::set_dword(&RegistryHive::Hkcu, &scratch.path, "Flag", 7).unwrap();
        let mismatched = scratch.reg_addr("Flag", RegType::Sz);
        let err = RegistryKind
            .read(&Setting::Registry(mismatched), &cx)
            .expect_err("a DWORD read as REG_SZ must be a typed error, never Absent");
        assert!(matches!(err, Error::TypeMismatch { .. }), "got {err:?}");
    }

    #[test]
    fn drive_roundtrip_every_reg_type() {
        let scratch = Scratch::new("roundtrip");
        let cx = user_cx();
        let cases = [
            (RegType::Dword, TypedRegValue::Dword(7)),
            (RegType::Qword, TypedRegValue::Qword(5_000_000_000)),
            (RegType::Sz, TypedRegValue::Sz("hello".to_string())),
            (
                RegType::ExpandSz,
                TypedRegValue::ExpandSz("%TEMP%\\x".to_string()),
            ),
            (
                RegType::MultiSz,
                TypedRegValue::MultiSz(vec!["a".to_string(), "b".to_string()]),
            ),
            (
                RegType::Binary,
                TypedRegValue::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            ),
        ];
        for (ty, value) in cases {
            let setting = Setting::Registry(scratch.reg_addr("Value", ty));
            RegistryKind
                .drive(&setting, &Value::Reg(value.clone()), &cx)
                .unwrap_or_else(|e| panic!("drive {ty:?} failed: {e}"));
            let read = RegistryKind
                .read(&setting, &cx)
                .unwrap_or_else(|e| panic!("read {ty:?} failed: {e}"));
            assert_eq!(read, Value::Reg(value));
        }
    }

    #[test]
    fn drive_absent_deletes_value() {
        let scratch = Scratch::new("absent_deletes");
        let cx = user_cx();
        let setting = Setting::Registry(scratch.reg_addr("Flag", RegType::Dword));

        RegistryKind
            .drive(&setting, &Value::Reg(TypedRegValue::Dword(1)), &cx)
            .unwrap();
        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Reg(TypedRegValue::Dword(1))
        );

        RegistryKind.drive(&setting, &Value::Absent, &cx).unwrap();
        assert_eq!(RegistryKind.read(&setting, &cx).unwrap(), Value::Absent);

        // Deleting an already-absent value is idempotent success, not an error.
        RegistryKind
            .drive(&setting, &Value::Absent, &cx)
            .expect("deleting an already-absent value must be a no-op success");
    }

    #[test]
    fn drive_autocreates_parent_path() {
        let scratch = Scratch::new("autocreate");
        let cx = user_cx();
        let addr = RegAddr {
            hive: Hive::Hkcu,
            path: format!("{}\\A\\B\\C", scratch.path),
            name: "Flag".to_string(),
            ty: RegType::Dword,
            field: None,
        };
        let setting = Setting::Registry(addr);

        RegistryKind
            .drive(&setting, &Value::Reg(TypedRegValue::Dword(42)), &cx)
            .expect("drive must auto-create the missing parent chain");
        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Reg(TypedRegValue::Dword(42))
        );
    }

    #[test]
    fn key_presence_roundtrip() {
        let scratch = Scratch::new("key_presence");
        let cx = user_cx();
        let setting = Setting::RegistryKey(KeyAddr {
            hive: Hive::Hkcu,
            path: format!("{}\\Sub", scratch.path),
        });

        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );

        RegistryKind
            .drive(&setting, &Value::Present(true), &cx)
            .unwrap();
        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Present(true)
        );

        RegistryKind
            .drive(&setting, &Value::Present(false), &cx)
            .unwrap();
        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Present(false)
        );
    }

    #[test]
    fn field_upsert_preserves_unknown_fields() {
        let scratch = Scratch::new("field_upsert");
        let cx = user_cx();
        registry_service::set_string(&RegistryHive::Hkcu, &scratch.path, "Packed", "A=1;X=9;")
            .unwrap();

        let addr = RegAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
            name: "Packed".to_string(),
            ty: RegType::Sz,
            field: Some(FieldAddr {
                field: "A".to_string(),
                format: PackedFormat::KvSemicolon,
            }),
        };
        let setting = Setting::Registry(addr);

        RegistryKind
            .drive(
                &setting,
                &Value::Reg(TypedRegValue::Sz("2".to_string())),
                &cx,
            )
            .unwrap();

        assert_eq!(
            RegistryKind.read(&setting, &cx).unwrap(),
            Value::Reg(TypedRegValue::Sz("2".to_string()))
        );
        let whole = registry_service::read_string(&RegistryHive::Hkcu, &scratch.path, "Packed")
            .unwrap()
            .unwrap();
        assert_eq!(whole, "A=2;X=9;");
    }

    #[test]
    fn field_on_malformed_value_is_typed_error() {
        let scratch = Scratch::new("field_malformed");
        let cx = user_cx();
        registry_service::set_string(
            &RegistryHive::Hkcu,
            &scratch.path,
            "Packed",
            "no-separators==;;",
        )
        .unwrap();

        let addr = RegAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
            name: "Packed".to_string(),
            ty: RegType::Sz,
            field: Some(FieldAddr {
                field: "A".to_string(),
                format: PackedFormat::KvSemicolon,
            }),
        };
        let setting = Setting::Registry(addr);

        let read_err = RegistryKind
            .read(&setting, &cx)
            .expect_err("malformed packed value must not parse");
        assert!(
            matches!(read_err, Error::MalformedPacked { .. }),
            "got {read_err:?}"
        );

        let drive_err = RegistryKind
            .drive(
                &setting,
                &Value::Reg(TypedRegValue::Sz("x".to_string())),
                &cx,
            )
            .expect_err("drive over a malformed packed value must not guess or rewrite");
        assert!(
            matches!(drive_err, Error::MalformedPacked { .. }),
            "got {drive_err:?}"
        );

        // No destructive rewrite: the raw string is exactly what it was before the failed drive.
        let unchanged = registry_service::read_string(&RegistryHive::Hkcu, &scratch.path, "Packed")
            .unwrap()
            .unwrap();
        assert_eq!(unchanged, "no-separators==;;");
    }

    /// `RegistryKind::drive` itself is unchanged (spec §9 -- see kinds/mod.rs's module docs on
    /// placement): the routing decision now lives one layer up, in `engine::AllKinds::drive`,
    /// which never reaches this in-process `drive` for System/Ti at all. Calling it directly here
    /// (bypassing that routing) still correctly rejects -- this in-process kind never silently
    /// escalates on its own.
    #[test]
    fn in_process_drive_still_rejects_system_and_ti_levels() {
        let scratch = Scratch::new("level_gate");
        let setting = Setting::Registry(scratch.reg_addr("Flag", RegType::Dword));

        for level in [Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            let err = RegistryKind
                .drive(&setting, &Value::Reg(TypedRegValue::Dword(1)), &cx)
                .expect_err("the in-process kind must still reject System/Ti directly");
            assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
        }
    }

    /// This FLIPS the old expectation: a System/Ti registry drive is no longer a dead end --
    /// `to_broker_op` (what `engine::AllKinds::drive` actually calls for System/Ti) translates it
    /// into the broker's typed op instead. Pure translation, no real elevation/broker spawn.
    #[test]
    fn system_and_ti_registry_drives_now_translate_to_broker_ops() {
        let scratch = Scratch::new("broker_translate");
        let value_addr = Setting::Registry(scratch.reg_addr("Flag", RegType::Dword));
        let key_addr = Setting::RegistryKey(KeyAddr {
            hive: Hive::Hkcu,
            path: format!("{}\\Sub", scratch.path),
        });

        for level in [Level::System, Level::Ti] {
            let set_op = to_broker_op(&value_addr, &Value::Reg(TypedRegValue::Dword(7)), level)
                .expect("a whole-value Reg drive must translate");
            assert!(
                matches!(set_op, crate::services::elevation::BrokerOp::RegSet { .. }),
                "got {set_op:?}"
            );

            let delete_op = to_broker_op(&value_addr, &Value::Absent, level)
                .expect("Absent must translate to a delete");
            assert!(
                matches!(
                    delete_op,
                    crate::services::elevation::BrokerOp::RegDeleteValue { .. }
                ),
                "got {delete_op:?}"
            );

            let create_key_op = to_broker_op(&key_addr, &Value::Present(true), level)
                .expect("Present(true) on a key must translate to a create");
            assert!(
                matches!(
                    create_key_op,
                    crate::services::elevation::BrokerOp::RegCreateKey { .. }
                ),
                "got {create_key_op:?}"
            );

            let delete_key_op = to_broker_op(&key_addr, &Value::Present(false), level)
                .expect("Present(false) on a key must translate to a delete");
            assert!(
                matches!(
                    delete_key_op,
                    crate::services::elevation::BrokerOp::RegDeleteKey { .. }
                ),
                "got {delete_key_op:?}"
            );
        }
    }

    /// The one narrowed gap the translation does not cover (spec §9's brief): a field-addressed
    /// packed write stays `UnsupportedLevel`, since its read-modify-write cycle needs a read at the
    /// SAME elevated level the broker protocol has no op for.
    #[test]
    fn field_addressed_registry_drive_is_not_routed() {
        let scratch = Scratch::new("broker_field_gap");
        let addr = RegAddr {
            hive: Hive::Hkcu,
            path: scratch.path.clone(),
            name: "Packed".to_string(),
            ty: RegType::Sz,
            field: Some(FieldAddr {
                field: "A".to_string(),
                format: PackedFormat::KvSemicolon,
            }),
        };
        let err = to_broker_op(
            &Setting::Registry(addr),
            &Value::Reg(TypedRegValue::Sz("x".to_string())),
            Level::System,
        )
        .expect_err("a field-addressed write must not be routed yet");
        assert!(matches!(err, Error::UnsupportedLevel(_)), "got {err:?}");
    }

    #[test]
    fn read_runs_in_process_regardless_of_declared_level() {
        // Spec invariant 24: reads run at the current level; there is no elevated read op to
        // route, so `read` must not reject System/Ti the way `drive` does.
        let scratch = Scratch::new("level_read");
        registry_service::set_dword(&RegistryHive::Hkcu, &scratch.path, "Flag", 5).unwrap();
        let setting = Setting::Registry(scratch.reg_addr("Flag", RegType::Dword));

        for level in [Level::User, Level::Admin, Level::System, Level::Ti] {
            let cx = ExecCx::new(level);
            assert_eq!(
                RegistryKind.read(&setting, &cx).unwrap(),
                Value::Reg(TypedRegValue::Dword(5)),
                "read must not depend on level {level:?}"
            );
        }
    }
}
