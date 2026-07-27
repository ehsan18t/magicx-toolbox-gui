//! Temp files this app hands to another process by path.
//!
//! `%TEMP%` is writable and enumerable by every process running as the same user, including
//! Medium-integrity ones that never passed a UAC prompt. So a file this app creates there and then
//! names on another process's command line is reachable by an attacker in the window between our
//! write and that process's read. Who reads it decides how bad that is: for an Action's `.cmd` it
//! is code execution at the app's own level; for the elevation broker's request it is code
//! execution as SYSTEM or TrustedInstaller, which turns same-user code into a full escalation.
//!
//! [`ExclusiveTempFile`] closes that window with three guards, all of which are needed:
//!
//! - a CSPRNG-derived filename, so there is no predictable path to pre-plant at
//! - `create_new`, so an already-existing path at that name fails loudly instead of being followed
//!   or truncated
//! - a write handle held open with only `FILE_SHARE_READ`, so the intended reader can still read
//!   it while no other process can open it for write, delete, or rename
//!
//! The handle is released and the file deleted on drop, so a caller keeps the value alive for
//! exactly as long as the reading process needs it.

use std::fs::File;
use std::io::{self, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use windows_sys::Win32::Security::Cryptography::{
    BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG,
};
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

/// 128 bits of CSPRNG output, hex-encoded.
///
/// `BCryptGenRandom` with `BCRYPT_USE_SYSTEM_PREFERRED_RNG` needs no algorithm-provider handle
/// (`hAlgorithm` is documented as ignored in this mode). `windows-sys` is already a direct
/// dependency; `rand`/`getrandom`/`uuid` exist only transitively in `Cargo.lock`, so reaching for
/// one would mean a brand-new direct dependency for sixteen bytes.
pub fn random_hex_token() -> io::Result<String> {
    let mut buf = [0u8; 16];
    // SAFETY: `buf` is a valid, correctly-sized stack buffer; the preferred-RNG flag makes the
    // algorithm-handle argument ignored, so null is its documented value here.
    let status = unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    if status != 0 {
        return Err(io::Error::other(format!(
            "BCryptGenRandom failed with NTSTATUS {status:#x}"
        )));
    }
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

/// A temp file only this process can write to, for the lifetime of this value. See the module docs.
pub struct ExclusiveTempFile {
    path: PathBuf,
    /// `Option` so `Drop` can close the handle before deleting: Windows refuses to delete a file
    /// while a `FILE_SHARE_READ`-only handle to it is open, including our own.
    handle: Option<File>,
}

impl ExclusiveTempFile {
    /// Create `<temp>/<prefix>-<pid>-<random>.<ext>` holding `contents`.
    pub fn create(prefix: &str, ext: &str, contents: &[u8]) -> io::Result<Self> {
        let path = unique_temp_path(prefix, ext)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(FILE_SHARE_READ)
            .open(&path)?;
        file.write_all(contents)?;
        file.flush()?;
        Ok(Self {
            path,
            handle: Some(file),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ExclusiveTempFile {
    fn drop(&mut self) {
        drop(self.handle.take()); // release the share-mode lock before attempting the delete
        let _ = std::fs::remove_file(&self.path);
    }
}

/// An unpredictable path in `%TEMP%`. Split out because a file another process *creates* (rather
/// than reads) cannot be opened exclusively here, but still benefits from being unguessable.
pub fn unique_temp_path(prefix: &str, ext: &str) -> io::Result<PathBuf> {
    let token = random_hex_token()?;
    Ok(std::env::temp_dir().join(format!(
        "{prefix}-{}-{token}.{ext}",
        std::process::id()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_tokens_are_distinct_and_hex() {
        let a = random_hex_token().unwrap();
        let b = random_hex_token().unwrap();
        assert_ne!(a, b);
        assert_eq!(a.len(), 32, "128 bits as hex");
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()), "got {a}");
    }

    #[test]
    fn create_new_refuses_an_already_existing_path() {
        // The pre-plant guard, pinned against the OS behaviour it relies on: a path that already
        // exists must fail the open outright, never be followed or truncated.
        let file = ExclusiveTempFile::create("magicx-test", "tmp", b"first").unwrap();
        let err = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(FILE_SHARE_READ)
            .open(file.path())
            .expect_err("create_new must refuse an already-existing path");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn the_held_handle_blocks_another_writer_but_still_allows_reads() {
        /// Windows reports a share-mode conflict as ERROR_SHARING_VIOLATION, which Rust does not
        /// map to a named `ErrorKind`, so the raw code is what pins this.
        const ERROR_SHARING_VIOLATION: i32 = 32;

        let file = ExclusiveTempFile::create("magicx-test", "tmp", b"payload").unwrap();

        // This is the guard that matters: the process we hand the path to can still read it...
        assert_eq!(std::fs::read(file.path()).unwrap(), b"payload");
        // ...while nothing else can swap the contents out from under that reader.
        let err = std::fs::OpenOptions::new()
            .write(true)
            .open(file.path())
            .expect_err("a second writer must be denied while the file is in use");
        assert_eq!(err.raw_os_error(), Some(ERROR_SHARING_VIOLATION));

        // Delete and rename are refused for the same reason, which is what stops a swap-by-rename.
        assert_eq!(
            std::fs::remove_file(file.path())
                .expect_err("delete must be refused while the file is in use")
                .raw_os_error(),
            Some(ERROR_SHARING_VIOLATION)
        );
    }

    #[test]
    fn drop_releases_the_lock_and_removes_the_file() {
        let path = {
            let file = ExclusiveTempFile::create("magicx-test", "tmp", b"x").unwrap();
            file.path().to_path_buf()
        };
        assert!(!path.exists(), "drop must delete the file, not leak it");
    }
}
