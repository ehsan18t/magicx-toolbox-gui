use serde::{Serialize, Serializer};

// Custom error enum for the application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    // Add other custom error variants here
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
