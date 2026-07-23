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

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

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

impl Error {
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
            Error::CommandExecution(_) => "COMMAND_EXECUTION_FAILED",
            Error::NotFound(_) => "NOT_FOUND",
            Error::ValidationError(_) => "VALIDATION_FAILED",
            Error::Tweak(_) => "TWEAK_ENGINE_ERROR",
            Error::TweakUnavailable(_) => "TWEAK_UNAVAILABLE",
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
