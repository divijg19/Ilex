use std::fs;
use std::path::PathBuf;

use super::parsers::parse_os_release;
use super::{DetectionError, Detector, OsInfo, SystemSnapshot, map_parse_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsReleaseDetector {
    source: PathBuf,
}

impl Default for OsReleaseDetector {
    fn default() -> Self {
        Self {
            source: PathBuf::from("/etc/os-release"),
        }
    }
}

impl OsReleaseDetector {
    pub fn new(source: PathBuf) -> Self {
        Self { source }
    }
}

impl Detector for OsReleaseDetector {
    fn key(&self) -> &'static str {
        "os"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        let content = fs::read_to_string(&self.source).map_err(|error| {
            DetectionError::io(
                self.key(),
                format!("failed to read {}: {error}", self.source.display()),
            )
        })?;
        let os_info: OsInfo =
            parse_os_release(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.os = Some(os_info);
        Ok(())
    }
}
