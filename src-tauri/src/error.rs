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

    #[error("Unsupported Windows version")]
    UnsupportedWindowsVersion,

    #[error("Requires administrator privileges")]
    RequiresAdmin,

    #[error("Service control failed: {0}")]
    ServiceControl(String),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Failed to acquire state lock")]
    StateLock,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
