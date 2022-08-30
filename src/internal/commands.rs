use std::ffi::{OsStr, OsString};
use std::fs;
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::internal::config;
use crate::internal::error::{AppError, AppResult};
use crate::internal::is_tty;

pub struct StringOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

/// A wrapper around [`std::process::Command`] with predefined
/// commands used in this project as well as elevated access.
pub struct ShellCommand {
    command: String,
    args: Vec<OsString>,
    elevated: bool,
}

impl ShellCommand {
    pub fn pacman() -> Self {
        let config = config::read();
        let pacman_cmd = if config.base.powerpill && fs::metadata("/usr/bin/powerpill").is_ok() {
            Self::new("powerpill")
        } else {
            Self::new("pacman")
        };

        if is_tty() {
            pacman_cmd.arg("--color=always")
        } else {
            pacman_cmd
        }
    }

    pub fn pacdiff() -> Self {
        Self::new("pacdiff")
    }

    pub fn makepkg() -> Self {
        Self::new("makepkg")
    }

    pub fn git() -> Self {
        Self::new("git")
    }

    pub fn bash() -> Self {
        Self::new("bash")
    }

    pub fn sudo() -> Self {
        Self::new(&config::read().bin.sudo.unwrap_or_default())
    }

    fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            elevated: false,
        }
    }

    /// Adds one argument
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_os_string());

        self
    }

    /// Adds a list of arguments
    pub fn args<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(mut self, args: I) -> Self {
        self.args.append(
            &mut args
                .into_iter()
                .map(|a: S| a.as_ref().to_os_string())
                .collect(),
        );

        self
    }

    /// Runs the command with sudo
    pub const fn elevated(mut self) -> Self {
        self.elevated = true;

        self
    }

    /// Waits for the child to exit but returns an error when it exists with a non-zero status code
    pub fn wait_success(self) -> AppResult<()> {
        let status = self.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(AppError::NonZeroExit)
        }
    }

    /// Waits for the child to exit and returns the output status
    pub fn wait(self) -> AppResult<ExitStatus> {
        let mut child = self.spawn(false)?;

        child.wait().map_err(AppError::from)
    }

    /// Waits with output until the program completed and
    /// returns the string output object
    pub fn wait_with_output(self) -> AppResult<StringOutput> {
        let child = self.spawn(true)?;
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8(output.stdout).map_err(|e| AppError::from(e.to_string()))?;
        let stderr = String::from_utf8(output.stderr).map_err(|e| AppError::from(e.to_string()))?;

        Ok(StringOutput {
            status: output.status,
            stdout,
            stderr,
        })
    }

    fn spawn(self, piped: bool) -> AppResult<Child> {
        let (stdout, stderr) = if piped {
            (Stdio::piped(), Stdio::piped())
        } else {
            (Stdio::inherit(), Stdio::inherit())
        };
        let child = if self.elevated {
            Command::new(&config::read().bin.sudo.unwrap_or_default())
                .arg(self.command)
                .args(self.args)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()?
        } else {
            Command::new(self.command)
                .args(self.args)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()?
        };

        Ok(child)
    }
}
