use std::fs;
use std::path::PathBuf;

use super::parsers::parse_meminfo;
use super::{DetectionError, Detector, MemoryInfo, SystemSnapshot, map_parse_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInfoDetector {
    source: PathBuf,
}

impl Default for MemoryInfoDetector {
    fn default() -> Self {
        Self {
            source: PathBuf::from("/proc/meminfo"),
        }
    }
}

impl MemoryInfoDetector {
    pub fn new(source: PathBuf) -> Self {
        Self { source }
    }
}

impl Detector for MemoryInfoDetector {
    fn key(&self) -> &'static str {
        "memory"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        let content = fs::read_to_string(&self.source).map_err(|error| {
            DetectionError::io(
                self.key(),
                format!("failed to read {}: {error}", self.source.display()),
            )
        })?;
        let memory_info: MemoryInfo =
            parse_meminfo(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.memory = Some(memory_info);
        Ok(())
    }
}
