use std::fs;
use std::path::PathBuf;

use super::parsers::parse_cpu_info;
use super::{CpuInfo, DetectionError, Detector, SystemSnapshot, map_parse_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuInfoDetector {
    source: PathBuf,
}

impl Default for CpuInfoDetector {
    fn default() -> Self {
        Self {
            source: PathBuf::from("/proc/cpuinfo"),
        }
    }
}

impl CpuInfoDetector {
    pub fn new(source: PathBuf) -> Self {
        Self { source }
    }
}

impl Detector for CpuInfoDetector {
    fn key(&self) -> &'static str {
        "cpu"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        let content = fs::read_to_string(&self.source).map_err(|error| {
            DetectionError::io(
                self.key(),
                format!("failed to read {}: {error}", self.source.display()),
            )
        })?;
        let cpu_info: CpuInfo =
            parse_cpu_info(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.cpu = Some(cpu_info);
        Ok(())
    }
}
