use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemSnapshot {
    pub os: Option<OsInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub pretty_name: String,
    pub id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionIssue {
    pub detector_key: &'static str,
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
    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), String>;
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

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), String> {
        let content = fs::read_to_string(&self.source)
            .map_err(|error| format!("failed to read {}: {error}", self.source.display()))?;
        let os_info = parse_os_release(&content)?;
        snapshot.os = Some(os_info);
        Ok(())
    }
}

pub struct DetectorRegistry {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorRegistry {
    pub fn bootstrap() -> Self {
        Self {
            detectors: vec![Box::new(OsReleaseDetector::default())],
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

            if let Err(message) = result {
                issues.push(DetectionIssue {
                    detector_key: detector.key(),
                    message,
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

fn normalize_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace(r#"\""#, "\"")
        .replace(r#"\\"#, "\\")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{OsReleaseDetector, SystemSnapshot, parse_os_release};
    use crate::detectors::Detector;

    const FEDORA_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/fedora.txt");
    const MINIMAL_OS_RELEASE: &str = include_str!("../../tests/fixtures/os-release/minimal.txt");

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
    fn detect_all_records_timings_and_issues() {
        struct FailingDetector;

        impl Detector for FailingDetector {
            fn key(&self) -> &'static str {
                "failing"
            }

            fn detect(&self, _snapshot: &mut SystemSnapshot) -> Result<(), String> {
                Err("intentional failure".to_owned())
            }
        }

        let registry = super::DetectorRegistry {
            detectors: vec![Box::new(FailingDetector)],
        };
        let report = registry.detect_all();

        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.timings.len(), 1);
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

    #[test]
    fn fixture_files_exist() {
        assert!(fs::metadata(fixture_path("fedora.txt")).is_ok());
        assert!(fs::metadata(fixture_path("minimal.txt")).is_ok());
    }
}
