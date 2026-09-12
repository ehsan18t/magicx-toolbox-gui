use serde::{Serialize, Serializer};

// Custom error enum for the application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[error("Registry key not found: {0}")]
    RegistryKeyNotFound(String),

    #[error("Registry access denied: {0}")]
    RegistryAccessDenied(String),

    #[error("Registry operation failed: {0}")]
    RegistryOperation(String),

    #[error("Windows API error: {0}")]
    WindowsApi(String),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Requires administrator privileges")]
    RequiresAdmin,

    #[error("Service control failed: {0}")]
    ServiceControl(String),

    #[error("Update error: {0}")]
    Update(String),

    #[error("A tweak is still being changed. Wait for it to finish, then {0}.")]
    ApplyInFlight(&'static str),

    #[error(transparent)]
    AppExiting(crate::tweaks::engine::lifecycle::AppExiting),

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// A Windows call that reported a code. The code is carried structurally so a failure can be
    /// classified without parsing the message, which quotes the key, the value or the path.
    #[error("{context} (Windows error {code:#x})")]
    Win32 { context: String, code: u32 },

    /// A tweak engine operation (`apply`/`restore`/snapshot I/O) failed. Carries the engine's own
    /// `Display` text -- the command layer maps `EngineError`/`SnapshotError`/`ClaimsError` here
    /// rather than exposing those internal types across the IPC boundary.
    #[error("{0}")]
    Tweak(String),

    /// `apply_tweak`/`restore_tweak` refused a tweak that is unavailable for the current app
    /// elevation level or SID state (spec §9) -- a typed refusal, never a silent no-op.
    #[error("tweak unavailable: {0}")]
    TweakUnavailable(String),
}

/// Win32 codes (WinError.h) that [`Error::Win32`] carries and the failure classification keys on.
pub mod win32 {
    pub const FILE_NOT_FOUND: u32 = 2;
    pub const PATH_NOT_FOUND: u32 = 3;
    pub const ACCESS_DENIED: u32 = 5;
    pub const INVALID_DATA: u32 = 13;
    pub const SHARING_VIOLATION: u32 = 32;
    pub const LOCK_VIOLATION: u32 = 33;
    pub const INVALID_PARAMETER: u32 = 87;
    pub const BUSY: u32 = 170;
    pub const DEPENDENT_SERVICES_RUNNING: u32 = 1051;
    pub const SERVICE_REQUEST_TIMEOUT: u32 = 1053;
    pub const SERVICE_DOES_NOT_EXIST: u32 = 1060;
    pub const SERVICE_CANNOT_ACCEPT_CTRL: u32 = 1061;
    pub const SERVICE_MARKED_FOR_DELETE: u32 = 1072;
    pub const SHUTDOWN_IN_PROGRESS: u32 = 1115;
    pub const NOT_FOUND: u32 = 1168;
    pub const PRIVILEGE_NOT_HELD: u32 = 1314;
}

impl Error {
    pub fn win32(context: impl Into<String>, code: u32) -> Self {
        Error::Win32 {
            context: context.into(),
            code,
        }
    }

    /// `raw_os_error` is the Win32 code for anything the OS reported; the two kinds below cover the
    /// io errors Rust synthesizes without one.
    pub fn from_io(context: impl std::fmt::Display, e: &std::io::Error) -> Self {
        let code = e.raw_os_error().map_or_else(
            || match e.kind() {
                std::io::ErrorKind::PermissionDenied => win32::ACCESS_DENIED,
                std::io::ErrorKind::NotFound => win32::FILE_NOT_FOUND,
                _ => 0,
            },
            |raw| raw as u32,
        );
        Error::win32(format!("{context}: {e}"), code)
    }

    /// Get a stable error code for programmatic error handling in the frontend.
    /// These codes can be used for conditional logic, i18n, or telemetry.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Tauri(_) => "TAURI_ERROR",
            Error::RegistryKeyNotFound(_) => "REGISTRY_KEY_NOT_FOUND",
            Error::RegistryAccessDenied(_) => "REGISTRY_ACCESS_DENIED",
            Error::RegistryOperation(_) => "REGISTRY_OPERATION_FAILED",
            Error::WindowsApi(_) => "WINDOWS_API_ERROR",
            Error::BackupFailed(_) => "BACKUP_FAILED",
            Error::RequiresAdmin => "REQUIRES_ADMIN",
            Error::ServiceControl(_) => "SERVICE_CONTROL_FAILED",
            Error::Update(_) => "UPDATE_ERROR",
            Error::ApplyInFlight(_) => "APPLY_IN_FLIGHT",
            Error::CommandExecution(_) => "COMMAND_EXECUTION_FAILED",
            Error::NotFound(_) => "NOT_FOUND",
            Error::ValidationError(_) => "VALIDATION_FAILED",
            Error::Win32 { .. } => "WINDOWS_API_ERROR",
            Error::Tweak(_) => "TWEAK_ENGINE_ERROR",
            Error::TweakUnavailable(_) => "TWEAK_UNAVAILABLE",
            Error::AppExiting(_) => "APP_EXITING",
        }
    }

    pub fn exit_refused(
        refused: crate::tweaks::engine::lifecycle::ExitRefused,
        action: &'static str,
    ) -> Self {
        use crate::tweaks::engine::lifecycle::ExitRefused;
        match refused {
            ExitRefused::ApplyInFlight => Error::ApplyInFlight(action),
            ExitRefused::Exiting(exiting) => Error::AppExiting(exiting),
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a struct with code and message for richer frontend handling
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
