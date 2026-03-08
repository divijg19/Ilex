use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::parsers::parse_passwd_shell;
use super::{DetectionError, Detector, ShellInfo, SystemSnapshot, map_parse_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellDetector {
    env_shell: Option<String>,
    passwd_source: PathBuf,
    uid: u32,
}

impl Default for ShellDetector {
    fn default() -> Self {
        Self {
            env_shell: env::var("SHELL")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            passwd_source: PathBuf::from("/etc/passwd"),
            uid: current_uid(),
        }
    }
}

impl ShellDetector {
    pub fn new(env_shell: Option<String>, passwd_source: PathBuf, uid: u32) -> Self {
        Self {
            env_shell,
            passwd_source,
            uid,
        }
    }
}

impl Detector for ShellDetector {
    fn key(&self) -> &'static str {
        "shell"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        let shell_path = match self
            .env_shell
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(shell) => shell.to_owned(),
            None => {
                let content = fs::read_to_string(&self.passwd_source).map_err(|error| {
                    DetectionError::io(
                        self.key(),
                        format!("failed to read {}: {error}", self.passwd_source.display()),
                    )
                })?;
                parse_passwd_shell(&content, self.uid)
                    .map_err(|message| map_parse_error(self.key(), message))?
            }
        };
        let name = shell_name_from_path(&shell_path).ok_or_else(|| {
            DetectionError::missing_field(self.key(), "missing shell executable name".to_owned())
        })?;

        snapshot.shell = Some(ShellInfo {
            executable_path: shell_path,
            name,
        });
        Ok(())
    }
}

fn shell_name_from_path(value: &str) -> Option<String> {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn current_uid() -> u32 {
    unsafe { libc::geteuid() as u32 }
}
