use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub pretty_name: String,
    pub id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuInfo {
    pub model_name: String,
    pub logical_cores: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_kib: u64,
    pub available_kib: Option<u64>,
}

impl MemoryInfo {
    pub fn used_kib(&self) -> Option<u64> {
        self.available_kib
            .map(|available| self.total_kib.saturating_sub(available))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionErrorKind {
    Io,
    Parse,
    MissingField,
}

impl DetectionErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Parse => "parse",
            Self::MissingField => "missing-field",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionError {
    pub detector_key: &'static str,
    pub kind: DetectionErrorKind,
    pub message: String,
}

impl DetectionError {
    fn io(detector_key: &'static str, message: String) -> Self {
        Self {
            detector_key,
            kind: DetectionErrorKind::Io,
            message,
        }
    }

    fn parse(detector_key: &'static str, message: String) -> Self {
        Self {
            detector_key,
            kind: DetectionErrorKind::Parse,
            message,
        }
    }

    fn missing_field(detector_key: &'static str, message: String) -> Self {
        Self {
            detector_key,
            kind: DetectionErrorKind::MissingField,
            message,
        }
    }
}

impl fmt::Display for DetectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionIssue {
    pub detector_key: &'static str,
    pub kind: DetectionErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DetectionReport {
    pub snapshot: SystemSnapshot,
    pub issues: Vec<DetectionIssue>,
    pub timings: Vec<DetectorTiming>,
    pub total_detection_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectorTiming {
    pub detector_key: &'static str,
    pub elapsed: Duration,
}

pub trait Detector {
    fn key(&self) -> &'static str;
    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError>;
}

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
        let os_info =
            parse_os_release(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.os = Some(os_info);
        Ok(())
    }
}

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
        let cpu_info =
            parse_cpu_info(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.cpu = Some(cpu_info);
        Ok(())
    }
}

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
        let memory_info =
            parse_meminfo(&content).map_err(|message| map_parse_error(self.key(), message))?;
        snapshot.memory = Some(memory_info);
        Ok(())
    }
}

pub struct DetectorRegistry {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorRegistry {
    pub fn bootstrap() -> Self {
        Self {
            detectors: vec![
                Box::new(OsReleaseDetector::default()),
                Box::new(CpuInfoDetector::default()),
                Box::new(MemoryInfoDetector::default()),
            ],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.detectors
            .iter()
            .map(|detector| detector.key())
            .collect()
    }

    pub fn detect_all(&self) -> DetectionReport {
        let started_at = Instant::now();
        let mut snapshot = SystemSnapshot::default();
        let mut issues = Vec::new();
        let mut timings = Vec::new();

        for detector in &self.detectors {
            let detector_started_at = Instant::now();
            let result = detector.detect(&mut snapshot);
            let elapsed = detector_started_at.elapsed();

            timings.push(DetectorTiming {
                detector_key: detector.key(),
                elapsed,
            });

            if let Err(error) = result {
                issues.push(DetectionIssue {
                    detector_key: error.detector_key,
                    kind: error.kind,
                    message: error.message,
                });
            }
        }

        DetectionReport {
            snapshot,
            issues,
            timings,
            total_detection_time: started_at.elapsed(),
        }
    }
}

fn parse_os_release(content: &str) -> Result<OsInfo, String> {
    let values = parse_key_value_lines(content);
    let name = values
        .get("NAME")
        .cloned()
        .ok_or_else(|| "missing NAME in os-release".to_owned())?;
    let pretty_name = values
        .get("PRETTY_NAME")
        .cloned()
        .unwrap_or_else(|| name.clone());

    Ok(OsInfo {
        name,
        pretty_name,
        id: values.get("ID").cloned(),
        version_id: values.get("VERSION_ID").cloned(),
    })
}

fn parse_cpu_info(content: &str) -> Result<CpuInfo, String> {
    let logical_cores = content
        .lines()
        .filter(|line| line.trim_start().starts_with("processor"))
        .count();
    if logical_cores == 0 {
        return Err("missing processor entries in cpuinfo".to_owned());
    }

    let model_name = content
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.trim() == "model name").then(|| value.trim().to_owned())
        })
        .ok_or_else(|| "missing model name in cpuinfo".to_owned())?;

    Ok(CpuInfo {
        model_name,
        logical_cores,
    })
}

fn parse_meminfo(content: &str) -> Result<MemoryInfo, String> {
    let values = parse_proc_value_lines(content)?;
    let total_kib = values
        .get("MemTotal")
        .copied()
        .ok_or_else(|| "missing MemTotal in meminfo".to_owned())?;

    Ok(MemoryInfo {
        total_kib,
        available_kib: values.get("MemAvailable").copied(),
    })
}

fn parse_key_value_lines(content: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        values.insert(key.to_owned(), normalize_value(value));
    }

    values
}

fn parse_proc_value_lines(content: &str) -> Result<BTreeMap<String, u64>, String> {
    let mut values = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let numeric = value
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("missing numeric value for {key}"))?
            .parse::<u64>()
            .map_err(|error| format!("invalid numeric value for {key}: {error}"))?;

        values.insert(key.trim().to_owned(), numeric);
    }

    Ok(values)
}

fn normalize_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace(r#"\""#, "\"")
        .replace(r#"\\"#, "\\")
}

fn map_parse_error(detector_key: &'static str, message: String) -> DetectionError {
    if message.starts_with("missing ") {
        DetectionError::missing_field(detector_key, message)
    } else {
        DetectionError::parse(detector_key, message)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        CpuInfoDetector, DetectionErrorKind, MemoryInfoDetector, OsReleaseDetector, SystemSnapshot,
        parse_cpu_info, parse_meminfo, parse_os_release,
    };
    use crate::detectors::Detector;

    const FEDORA_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/fedora.txt");
    const MINIMAL_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/minimal.txt");
    const CPUINFO: &str = include_str!("../../tests/fixtures/proc/cpuinfo/basic.txt");
    const MEMINFO: &str = include_str!("../../tests/fixtures/proc/meminfo/basic.txt");
    const MEMINFO_NO_AVAILABLE: &str =
        include_str!("../../tests/fixtures/proc/meminfo/no-available.txt");

    #[test]
    fn parses_pretty_name_from_os_release() {
        let os = parse_os_release(FEDORA_OS_RELEASE).expect("os-release should parse");

        assert_eq!(os.name, "Fedora Linux");
        assert_eq!(os.pretty_name, "Fedora Linux 43");
        assert_eq!(os.id.as_deref(), Some("fedora"));
        assert_eq!(os.version_id.as_deref(), Some("43"));
    }

    #[test]
    fn falls_back_to_name_when_pretty_name_missing() {
        let os = parse_os_release(MINIMAL_OS_RELEASE).expect("minimal os-release should parse");

        assert_eq!(os.name, "Tiny Linux");
        assert_eq!(os.pretty_name, "Tiny Linux");
        assert_eq!(os.id.as_deref(), Some("tiny"));
        assert_eq!(os.version_id.as_deref(), None);
    }

    #[test]
    fn detector_populates_snapshot_from_fixture() {
        let detector = OsReleaseDetector::new(fixture_path("fedora.txt"));
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("detector should succeed");

        assert_eq!(
            snapshot.os.as_ref().map(|os| os.pretty_name.as_str()),
            Some("Fedora Linux 43")
        );
    }

    #[test]
    fn parses_cpu_info_from_fixture() {
        let cpu = parse_cpu_info(CPUINFO).expect("cpuinfo should parse");

        assert_eq!(cpu.model_name, "ExampleCore 9000");
        assert_eq!(cpu.logical_cores, 4);
    }

    #[test]
    fn cpu_detector_populates_snapshot_from_fixture() {
        let detector = CpuInfoDetector::new(proc_fixture_path("cpuinfo", "basic.txt"));
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("cpu detector should succeed");

        assert_eq!(snapshot.cpu.as_ref().map(|cpu| cpu.logical_cores), Some(4));
    }

    #[test]
    fn parses_meminfo_from_fixture() {
        let memory = parse_meminfo(MEMINFO).expect("meminfo should parse");

        assert_eq!(memory.total_kib, 32768000);
        assert_eq!(memory.available_kib, Some(24576000));
        assert_eq!(memory.used_kib(), Some(8192000));
    }

    #[test]
    fn meminfo_allows_missing_available_value() {
        let memory = parse_meminfo(MEMINFO_NO_AVAILABLE).expect("meminfo should parse");

        assert_eq!(memory.total_kib, 16384000);
        assert_eq!(memory.available_kib, None);
        assert_eq!(memory.used_kib(), None);
    }

    #[test]
    fn memory_detector_populates_snapshot_from_fixture() {
        let detector = MemoryInfoDetector::new(proc_fixture_path("meminfo", "basic.txt"));
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("memory detector should succeed");

        assert_eq!(
            snapshot.memory.as_ref().map(|memory| memory.available_kib),
            Some(Some(24576000))
        );
    }

    #[test]
    fn detect_all_records_timings_and_issues() {
        struct FailingDetector;

        impl Detector for FailingDetector {
            fn key(&self) -> &'static str {
                "failing"
            }

            fn detect(&self, _snapshot: &mut SystemSnapshot) -> Result<(), super::DetectionError> {
                Err(super::DetectionError::parse(
                    self.key(),
                    "intentional failure".to_owned(),
                ))
            }
        }

        let registry = super::DetectorRegistry {
            detectors: vec![Box::new(FailingDetector)],
        };
        let report = registry.detect_all();

        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.timings.len(), 1);
        assert_eq!(report.issues[0].kind, DetectionErrorKind::Parse);
        assert_eq!(report.timings[0].detector_key, "failing");
        assert!(report.total_detection_time >= report.timings[0].elapsed);
    }

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("os-release")
            .join(name)
    }

    fn proc_fixture_path(kind: &str, name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("proc")
            .join(kind)
            .join(name)
    }

    #[test]
    fn fixture_files_exist() {
        assert!(fs::metadata(fixture_path("fedora.txt")).is_ok());
        assert!(fs::metadata(fixture_path("minimal.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("cpuinfo", "basic.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "basic.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "no-available.txt")).is_ok());
    }
}
