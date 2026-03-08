use std::fmt;
use std::time::{Duration, Instant};

mod cpu;
mod disk;
mod memory;
mod os;
mod parsers;
mod shell;
mod terminal;

pub use self::cpu::CpuInfoDetector;
pub use self::disk::DiskDetector;
pub use self::memory::MemoryInfoDetector;
pub use self::os::OsReleaseDetector;
pub use self::shell::ShellDetector;
pub use self::terminal::TerminalDetector;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub disk: Option<DiskInfo>,
    pub shell: Option<ShellInfo>,
    pub terminal: Option<TerminalInfo>,
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
pub struct DiskInfo {
    pub device: String,
    pub filesystem: String,
    pub mount_point: String,
    pub total_kib: u64,
    pub available_kib: Option<u64>,
}

impl DiskInfo {
    pub fn used_kib(&self) -> Option<u64> {
        self.available_kib
            .map(|available| self.total_kib.saturating_sub(available))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellInfo {
    pub executable_path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    pub name: String,
    pub term: Option<String>,
    pub color_term: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DiskMount {
    pub device: String,
    pub filesystem: String,
    pub mount_point: String,
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
    pub(super) fn io(detector_key: &'static str, message: String) -> Self {
        Self {
            detector_key,
            kind: DetectionErrorKind::Io,
            message,
        }
    }

    pub(super) fn parse(detector_key: &'static str, message: String) -> Self {
        Self {
            detector_key,
            kind: DetectionErrorKind::Parse,
            message,
        }
    }

    pub(super) fn missing_field(detector_key: &'static str, message: String) -> Self {
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
                Box::new(DiskDetector::default()),
                Box::new(ShellDetector::default()),
                Box::new(TerminalDetector::default()),
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

pub(super) fn map_parse_error(detector_key: &'static str, message: String) -> DetectionError {
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

    use super::parsers::{
        parse_cpu_info, parse_meminfo, parse_os_release, parse_passwd_shell, parse_primary_mount,
        parse_terminal_info,
    };
    use super::{
        CpuInfoDetector, DetectionErrorKind, DiskDetector, MemoryInfoDetector, OsReleaseDetector,
        ShellDetector, SystemSnapshot, TerminalDetector,
    };
    use crate::detectors::Detector;

    const FEDORA_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/fedora.txt");
    const MINIMAL_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/minimal.txt");
    const OS_RELEASE_MISSING_NAME: &str =
        include_str!("../../tests/fixtures/os-release/missing-name.txt");
    const CPUINFO: &str = include_str!("../../tests/fixtures/proc/cpuinfo/basic.txt");
    const CPUINFO_FALLBACK: &str = include_str!("../../tests/fixtures/proc/cpuinfo/fallback.txt");
    const CPUINFO_MISSING_MODEL: &str =
        include_str!("../../tests/fixtures/proc/cpuinfo/missing-model.txt");
    const CPUINFO_MISSING_CORES: &str =
        include_str!("../../tests/fixtures/proc/cpuinfo/missing-cores.txt");
    const MEMINFO: &str = include_str!("../../tests/fixtures/proc/meminfo/basic.txt");
    const MEMINFO_NO_AVAILABLE: &str =
        include_str!("../../tests/fixtures/proc/meminfo/no-available.txt");
    const MEMINFO_NO_AVAILABLE_NO_FREE: &str =
        include_str!("../../tests/fixtures/proc/meminfo/no-available-no-free.txt");
    const MEMINFO_INVALID_TOTAL: &str =
        include_str!("../../tests/fixtures/proc/meminfo/invalid-total.txt");
    const MEMINFO_MISSING_TOTAL: &str =
        include_str!("../../tests/fixtures/proc/meminfo/missing-total.txt");
    const MOUNTS_BASIC: &str = include_str!("../../tests/fixtures/proc/mounts/basic.txt");
    const MOUNTS_NO_ROOT: &str = include_str!("../../tests/fixtures/proc/mounts/no-root.txt");
    const PASSWD_BASIC: &str = include_str!("../../tests/fixtures/etc/passwd/basic.txt");
    const PASSWD_NO_MATCH: &str = include_str!("../../tests/fixtures/etc/passwd/no-match.txt");

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
    fn os_release_without_name_reports_missing_field() {
        let error =
            parse_os_release(OS_RELEASE_MISSING_NAME).expect_err("missing NAME should fail");

        assert!(error.contains("missing NAME"));
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
    fn cpuinfo_falls_back_to_hardware_and_cpu_cores() {
        let cpu = parse_cpu_info(CPUINFO_FALLBACK).expect("fallback cpuinfo should parse");

        assert_eq!(cpu.model_name, "ARM Example SoC");
        assert_eq!(cpu.logical_cores, 6);
    }

    #[test]
    fn cpuinfo_without_model_reports_missing_field() {
        let error = parse_cpu_info(CPUINFO_MISSING_MODEL).expect_err("missing model should fail");

        assert!(error.contains("missing model name"));
    }

    #[test]
    fn cpuinfo_without_core_data_reports_missing_field() {
        let error =
            parse_cpu_info(CPUINFO_MISSING_CORES).expect_err("missing core data should fail");

        assert!(error.contains("missing processor or cpu cores"));
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
    fn meminfo_falls_back_to_memfree_when_available_is_missing() {
        let memory = parse_meminfo(MEMINFO_NO_AVAILABLE).expect("meminfo should parse");

        assert_eq!(memory.total_kib, 16384000);
        assert_eq!(memory.available_kib, Some(8192000));
        assert_eq!(memory.used_kib(), Some(8192000));
    }

    #[test]
    fn meminfo_without_available_or_free_keeps_used_unknown() {
        let memory = parse_meminfo(MEMINFO_NO_AVAILABLE_NO_FREE).expect("meminfo should parse");

        assert_eq!(memory.total_kib, 16384000);
        assert_eq!(memory.available_kib, None);
        assert_eq!(memory.used_kib(), None);
    }

    #[test]
    fn meminfo_with_invalid_total_reports_parse_error() {
        let error = parse_meminfo(MEMINFO_INVALID_TOTAL).expect_err("invalid total should fail");

        assert!(error.contains("invalid numeric value for MemTotal"));
    }

    #[test]
    fn meminfo_without_total_reports_missing_field() {
        let error = parse_meminfo(MEMINFO_MISSING_TOTAL).expect_err("missing total should fail");

        assert!(error.contains("missing MemTotal"));
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
    fn parses_primary_disk_mount_from_fixture() {
        let mount = parse_primary_mount(MOUNTS_BASIC).expect("mounts should parse");

        assert_eq!(mount.device, "/dev/nvme0n1p2");
        assert_eq!(mount.filesystem, "ext4");
        assert_eq!(mount.mount_point, "/");
    }

    #[test]
    fn mounts_without_root_report_missing_field() {
        let error = parse_primary_mount(MOUNTS_NO_ROOT).expect_err("missing root should fail");

        assert!(error.contains("missing root filesystem entry"));
    }

    #[test]
    fn disk_detector_populates_snapshot_from_fixture() {
        let detector = DiskDetector::new(proc_fixture_path("mounts", "basic.txt"));
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("disk detector should succeed");

        let disk = snapshot.disk.as_ref().expect("disk snapshot should exist");
        assert_eq!(disk.mount_point, "/");
        assert!(disk.total_kib > 0);
        assert!(disk.available_kib.is_some());
    }

    #[test]
    fn parses_shell_from_passwd_fixture() {
        let shell = parse_passwd_shell(PASSWD_BASIC, 1000).expect("passwd should parse");

        assert_eq!(shell, "/usr/bin/fish");
    }

    #[test]
    fn passwd_without_matching_uid_reports_missing_field() {
        let error = parse_passwd_shell(PASSWD_NO_MATCH, 1000).expect_err("missing uid should fail");

        assert!(error.contains("missing shell entry for uid 1000"));
    }

    #[test]
    fn shell_detector_prefers_env_shell() {
        let detector = ShellDetector::new(
            Some("/usr/bin/zsh".to_owned()),
            etc_fixture_path("passwd", "basic.txt"),
            1000,
        );
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("shell detector should succeed");

        let shell = snapshot
            .shell
            .as_ref()
            .expect("shell snapshot should exist");
        assert_eq!(shell.name, "zsh");
        assert_eq!(shell.executable_path, "/usr/bin/zsh");
    }

    #[test]
    fn shell_detector_falls_back_to_passwd() {
        let detector = ShellDetector::new(None, etc_fixture_path("passwd", "basic.txt"), 1000);
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("shell detector should succeed");

        let shell = snapshot
            .shell
            .as_ref()
            .expect("shell snapshot should exist");
        assert_eq!(shell.name, "fish");
        assert_eq!(shell.executable_path, "/usr/bin/fish");
    }

    #[test]
    fn terminal_detector_prefers_term_program() {
        let detector = TerminalDetector::new(
            Some("Ghostty".to_owned()),
            Some("xterm-256color".to_owned()),
            Some("truecolor".to_owned()),
        );
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("terminal detector should succeed");

        let terminal = snapshot
            .terminal
            .as_ref()
            .expect("terminal snapshot should exist");
        assert_eq!(terminal.name, "Ghostty");
        assert_eq!(terminal.term.as_deref(), Some("xterm-256color"));
        assert_eq!(terminal.color_term.as_deref(), Some("truecolor"));
    }

    #[test]
    fn terminal_detector_falls_back_to_unknown() {
        let detector = TerminalDetector::new(None, None, None);
        let mut snapshot = SystemSnapshot::default();

        detector
            .detect(&mut snapshot)
            .expect("terminal detector should succeed");

        let terminal = snapshot
            .terminal
            .as_ref()
            .expect("terminal snapshot should exist");
        assert_eq!(terminal.name, "unknown");
        assert_eq!(terminal.term, None);
    }

    #[test]
    fn parses_terminal_info_from_known_values() {
        let terminal = parse_terminal_info(Some("Ghostty"), Some("xterm-256color"), None);

        assert_eq!(terminal.name, "Ghostty");
        assert_eq!(terminal.term.as_deref(), Some("xterm-256color"));
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

    #[test]
    fn detector_error_kind_is_missing_field_for_missing_os_name() {
        let detector = OsReleaseDetector::new(fixture_path("missing-name.txt"));
        let mut snapshot = SystemSnapshot::default();

        let error = detector
            .detect(&mut snapshot)
            .expect_err("missing name should fail");

        assert_eq!(error.kind, DetectionErrorKind::MissingField);
    }

    #[test]
    fn detector_error_kind_is_parse_for_invalid_memtotal() {
        let detector = MemoryInfoDetector::new(proc_fixture_path("meminfo", "invalid-total.txt"));
        let mut snapshot = SystemSnapshot::default();

        let error = detector
            .detect(&mut snapshot)
            .expect_err("invalid total should fail");

        assert_eq!(error.kind, DetectionErrorKind::Parse);
    }

    #[test]
    fn detector_error_kind_is_missing_field_for_missing_root_mount() {
        let detector = DiskDetector::new(proc_fixture_path("mounts", "no-root.txt"));
        let mut snapshot = SystemSnapshot::default();

        let error = detector
            .detect(&mut snapshot)
            .expect_err("missing root mount should fail");

        assert_eq!(error.kind, DetectionErrorKind::MissingField);
    }

    #[test]
    fn detector_error_kind_is_missing_field_for_missing_shell_uid() {
        let detector = ShellDetector::new(None, etc_fixture_path("passwd", "no-match.txt"), 1000);
        let mut snapshot = SystemSnapshot::default();

        let error = detector
            .detect(&mut snapshot)
            .expect_err("missing passwd shell should fail");

        assert_eq!(error.kind, DetectionErrorKind::MissingField);
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

    fn etc_fixture_path(kind: &str, name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("etc")
            .join(kind)
            .join(name)
    }

    #[test]
    fn fixture_files_exist() {
        assert!(fs::metadata(fixture_path("fedora.txt")).is_ok());
        assert!(fs::metadata(fixture_path("minimal.txt")).is_ok());
        assert!(fs::metadata(fixture_path("missing-name.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("cpuinfo", "basic.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("cpuinfo", "fallback.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("cpuinfo", "missing-model.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("cpuinfo", "missing-cores.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "basic.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "no-available.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "no-available-no-free.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "invalid-total.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("meminfo", "missing-total.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("mounts", "basic.txt")).is_ok());
        assert!(fs::metadata(proc_fixture_path("mounts", "no-root.txt")).is_ok());
        assert!(fs::metadata(etc_fixture_path("passwd", "basic.txt")).is_ok());
        assert!(fs::metadata(etc_fixture_path("passwd", "no-match.txt")).is_ok());
    }
}
