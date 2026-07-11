//! Helpers for spawning child processes without flashing console windows on Windows.

use std::ffi::OsStr;
use std::process::Command;

/// Create a `Command` that does not open a visible console window on Windows.
///
/// GUI builds use `windows_subsystem = "windows"`, so spawning `tasklist`, `netstat`,
/// `powershell`, and similar utilities without this flag causes focus-stealing flashes.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        return cmd;
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}
