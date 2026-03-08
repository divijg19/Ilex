use crate::modules::ModuleEntry;

pub const FOUNDATION_CONTRACT_VERSION: &str = "foundation-v1";
pub const BASELINE_CONTRACT_VERSION: &str = "baseline-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessCheck {
    pub key: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessReport {
    pub contract_version: &'static str,
    pub ready: bool,
    pub checks: Vec<ReadinessCheck>,
}

pub fn evaluate_foundation_readiness(
    primary_command: &str,
    detector_keys: &[String],
    module_keys: &[String],
    renderer_keys: &[String],
    module_entries: &[ModuleEntry],
    issue_count: usize,
) -> ReadinessReport {
    let checks = vec![
        ReadinessCheck {
            key: "primary-command",
            passed: primary_command == "corefetch",
            detail: format!("canonical command is {primary_command}"),
        },
        ReadinessCheck {
            key: "detector-registry",
            passed: ["os", "cpu", "memory"]
                .iter()
                .all(|required| detector_keys.iter().any(|key| key == required)),
            detail: format!("registered detectors: {}", detector_keys.join(", ")),
        },
        ReadinessCheck {
            key: "module-registry",
            passed: ["os", "cpu", "memory"]
                .iter()
                .all(|required| module_keys.iter().any(|key| key == required)),
            detail: format!("registered modules: {}", module_keys.join(", ")),
        },
        ReadinessCheck {
            key: "renderer-registry",
            passed: renderer_keys.iter().any(|key| key == "bootstrap-text"),
            detail: format!("registered renderers: {}", renderer_keys.join(", ")),
        },
        ReadinessCheck {
            key: "snapshot-flow",
            passed: ["os", "cpu", "memory"]
                .iter()
                .all(|required| module_entries.iter().any(|entry| &entry.key == required)),
            detail: format!("renderable module entries: {}", module_entries.len()),
        },
        ReadinessCheck {
            key: "issue-budget",
            passed: issue_count == 0,
            detail: format!("detection issues: {issue_count}"),
        },
    ];

    ReadinessReport {
        contract_version: FOUNDATION_CONTRACT_VERSION,
        ready: checks.iter().all(|check| check.passed),
        checks,
    }
}

pub fn evaluate_baseline_readiness(
    primary_command: &str,
    detector_keys: &[String],
    module_keys: &[String],
    renderer_keys: &[String],
    module_entries: &[ModuleEntry],
    issue_count: usize,
) -> ReadinessReport {
    let checks = vec![
        ReadinessCheck {
            key: "primary-command",
            passed: primary_command == "corefetch",
            detail: format!("canonical command is {primary_command}"),
        },
        ReadinessCheck {
            key: "detector-registry",
            passed: ["os", "cpu", "memory", "disk"]
                .iter()
                .all(|required| detector_keys.iter().any(|key| key == required)),
            detail: format!("registered detectors: {}", detector_keys.join(", ")),
        },
        ReadinessCheck {
            key: "module-registry",
            passed: ["os", "cpu", "memory", "disk"]
                .iter()
                .all(|required| module_keys.iter().any(|key| key == required)),
            detail: format!("registered modules: {}", module_keys.join(", ")),
        },
        ReadinessCheck {
            key: "renderer-registry",
            passed: ["fetch-text", "minimal-text", "json"]
                .iter()
                .all(|required| renderer_keys.iter().any(|key| key == required)),
            detail: format!("registered renderers: {}", renderer_keys.join(", ")),
        },
        ReadinessCheck {
            key: "snapshot-flow",
            passed: ["os", "cpu", "memory", "disk"]
                .iter()
                .all(|required| module_entries.iter().any(|entry| &entry.key == required)),
            detail: format!("renderable module entries: {}", module_entries.len()),
        },
        ReadinessCheck {
            key: "issue-budget",
            passed: issue_count == 0,
            detail: format!("detection issues: {issue_count}"),
        },
    ];

    ReadinessReport {
        contract_version: BASELINE_CONTRACT_VERSION,
        ready: checks.iter().all(|check| check.passed),
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BASELINE_CONTRACT_VERSION, FOUNDATION_CONTRACT_VERSION, evaluate_baseline_readiness,
        evaluate_foundation_readiness,
    };
    use crate::modules::ModuleEntry;

    #[test]
    fn readiness_passes_for_foundation_happy_path() {
        let report = evaluate_foundation_readiness(
            "corefetch",
            &["os".to_owned(), "cpu".to_owned(), "memory".to_owned()],
            &["os".to_owned(), "cpu".to_owned(), "memory".to_owned()],
            &["bootstrap-text".to_owned()],
            &[
                ModuleEntry {
                    key: "os",
                    label: "OS",
                    value: "Fedora Linux 43".to_owned(),
                },
                ModuleEntry {
                    key: "cpu",
                    label: "CPU",
                    value: "ExampleCore 9000 (4 cores)".to_owned(),
                },
                ModuleEntry {
                    key: "memory",
                    label: "Memory",
                    value: "7.8 GiB / 31.2 GiB".to_owned(),
                },
            ],
            0,
        );

        assert_eq!(report.contract_version, FOUNDATION_CONTRACT_VERSION);
        assert!(report.ready);
        assert!(report.checks.iter().all(|check| check.passed));
    }

    #[test]
    fn readiness_fails_when_memory_flow_is_missing() {
        let report = evaluate_foundation_readiness(
            "corefetch",
            &["os".to_owned(), "cpu".to_owned(), "memory".to_owned()],
            &["os".to_owned(), "cpu".to_owned(), "memory".to_owned()],
            &["bootstrap-text".to_owned()],
            &[
                ModuleEntry {
                    key: "os",
                    label: "OS",
                    value: "Fedora Linux 43".to_owned(),
                },
                ModuleEntry {
                    key: "cpu",
                    label: "CPU",
                    value: "ExampleCore 9000 (4 cores)".to_owned(),
                },
            ],
            0,
        );

        assert!(!report.ready);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.key == "snapshot-flow" && !check.passed)
        );
    }

    #[test]
    fn readiness_passes_for_baseline_happy_path() {
        let report = evaluate_baseline_readiness(
            "corefetch",
            &[
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
            ],
            &[
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
            ],
            &[
                "bootstrap-text".to_owned(),
                "fetch-text".to_owned(),
                "minimal-text".to_owned(),
                "json".to_owned(),
            ],
            &[
                ModuleEntry {
                    key: "os",
                    label: "OS",
                    value: "Fedora Linux 43".to_owned(),
                },
                ModuleEntry {
                    key: "cpu",
                    label: "CPU",
                    value: "ExampleCore 9000 (4 cores)".to_owned(),
                },
                ModuleEntry {
                    key: "memory",
                    label: "Memory",
                    value: "7.8 GiB / 31.2 GiB".to_owned(),
                },
                ModuleEntry {
                    key: "disk",
                    label: "Disk",
                    value: "31.2 GiB / 62.5 GiB (/)".to_owned(),
                },
            ],
            0,
        );

        assert_eq!(report.contract_version, BASELINE_CONTRACT_VERSION);
        assert!(report.ready);
        assert!(report.checks.iter().all(|check| check.passed));
    }

    #[test]
    fn readiness_fails_for_baseline_when_disk_flow_is_missing() {
        let report = evaluate_baseline_readiness(
            "corefetch",
            &[
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
            ],
            &[
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
            ],
            &[
                "fetch-text".to_owned(),
                "minimal-text".to_owned(),
                "json".to_owned(),
            ],
            &[
                ModuleEntry {
                    key: "os",
                    label: "OS",
                    value: "Fedora Linux 43".to_owned(),
                },
                ModuleEntry {
                    key: "cpu",
                    label: "CPU",
                    value: "ExampleCore 9000 (4 cores)".to_owned(),
                },
                ModuleEntry {
                    key: "memory",
                    label: "Memory",
                    value: "7.8 GiB / 31.2 GiB".to_owned(),
                },
            ],
            0,
        );

        assert!(!report.ready);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.key == "snapshot-flow" && !check.passed)
        );
    }
}
