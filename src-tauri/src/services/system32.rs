//! Launch Windows tools by absolute System32 path. std's `Command` searches the exe's own folder
//! before System32, and cmd.exe searches its working directory before PATH, so a bare name or an
//! inherited working directory can run a binary planted next to the app.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::Command;

use windows_sys::Win32::Foundation::{GetLastError, MAX_PATH};
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

use crate::error::Error;

/// Closed set: no caller names a path, so none can leave System32 via `..` or an absolute path.
#[derive(Debug, Clone, Copy)]
pub enum SystemTool {
    Cmd,
    PowerShell,
    Netsh,
    Msiexec,
}

impl SystemTool {
    fn relative_path(self) -> &'static str {
        match self {
            Self::Cmd => "cmd.exe",
            Self::PowerShell => r"WindowsPowerShell\v1.0\powershell.exe",
            Self::Netsh => "netsh.exe",
            Self::Msiexec => "msiexec.exe",
        }
    }

    /// Working directory is System32 too: cmd.exe resolves a script's bare names from it first.
    pub fn command(self) -> Result<Command, Error> {
        let dir = system_dir()?;
        let mut cmd = Command::new(dir.join(self.relative_path()));
        cmd.current_dir(dir);
        Ok(cmd)
    }
}

fn system_dir() -> Result<PathBuf, Error> {
    let mut buf = [0u16; MAX_PATH as usize];
    // SAFETY: `buf` is writable for exactly the length passed.
    let len = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    if len == 0 {
        return Err(Error::CommandExecution(format!(
            "failed to resolve the system directory: {}",
            unsafe { GetLastError() }
        )));
    }
    if len >= buf.len() {
        return Err(Error::CommandExecution(
            "the system directory path exceeds MAX_PATH".into(),
        ));
    }
    Ok(PathBuf::from(OsString::from_wide(&buf[..len])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn every_tool_launches_from_system32_with_it_as_cwd() {
        let system32 = Path::new(&std::env::var_os("SystemRoot").expect("SystemRoot is set"))
            .join("System32")
            .to_string_lossy()
            .to_lowercase();
        for tool in [
            SystemTool::Cmd,
            SystemTool::PowerShell,
            SystemTool::Netsh,
            SystemTool::Msiexec,
        ] {
            // Exhaustive: a new tool does not compile until its expected path is listed here.
            let expected = match tool {
                SystemTool::Cmd => "cmd.exe",
                SystemTool::PowerShell => r"windowspowershell\v1.0\powershell.exe",
                SystemTool::Netsh => "netsh.exe",
                SystemTool::Msiexec => "msiexec.exe",
            };
            let cmd = tool.command().expect("the system directory must resolve");
            let program = Path::new(cmd.get_program());
            assert_eq!(
                program.to_string_lossy().to_lowercase(),
                format!(r"{system32}\{expected}"),
                "{tool:?} must launch from System32, never by name resolution"
            );
            assert!(program.is_file(), "{} must exist", program.display());
            assert_eq!(
                cmd.get_current_dir()
                    .map(|d| d.to_string_lossy().to_lowercase()),
                Some(system32.clone()),
                "{tool:?} must run with System32 as its working directory"
            );
        }
    }

    #[test]
    fn no_module_launches_a_program_by_a_literal_name() {
        // Exempt: elevation/common.rs spawns cmd.exe only in its tests.
        let exempt = Path::new(r"services\elevation\common.rs");
        let mut offenders = Vec::new();
        let mut dirs = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "rs")
                    && !path.ends_with(exempt)
                    && std::fs::read_to_string(&path)
                        .unwrap()
                        .contains("Command::new(\"")
                {
                    offenders.push(path);
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "launch through SystemTool, not a literal name: {offenders:?}"
        );
    }
}
