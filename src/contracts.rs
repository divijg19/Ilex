use crate::modules::ModuleEntry;

pub const BOOTSTRAP_CONTRACT_VERSION: &str = "bootstrap-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessCheck {
    pub key: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessReport {
    pub contract_version: &'static str,
    pub ready_for_foundations: bool,
    pub checks: Vec<ReadinessCheck>,
}

pub fn evaluate_bootstrap_readiness(
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
            passed: detector_keys.iter().any(|key| key == "os"),
            detail: format!("registered detectors: {}", detector_keys.join(", ")),
        },
        ReadinessCheck {
            key: "module-registry",
            passed: module_keys.iter().any(|key| key == "os"),
            detail: format!("registered modules: {}", module_keys.join(", ")),
        },
        ReadinessCheck {
            key: "renderer-registry",
            passed: renderer_keys.iter().any(|key| key == "bootstrap-text"),
            detail: format!("registered renderers: {}", renderer_keys.join(", ")),
        },
        ReadinessCheck {
            key: "snapshot-flow",
            passed: module_entries.iter().any(|entry| entry.key == "os"),
            detail: format!("renderable module entries: {}", module_entries.len()),
        },
        ReadinessCheck {
            key: "issue-budget",
            passed: issue_count == 0,
            detail: format!("detection issues: {issue_count}"),
        },
    ];

    ReadinessReport {
        contract_version: BOOTSTRAP_CONTRACT_VERSION,
        ready_for_foundations: checks.iter().all(|check| check.passed),
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::{BOOTSTRAP_CONTRACT_VERSION, evaluate_bootstrap_readiness};
    use crate::modules::ModuleEntry;

    #[test]
    fn readiness_passes_for_bootstrap_happy_path() {
        let report = evaluate_bootstrap_readiness(
            "corefetch",
            &["os".to_owned()],
            &["os".to_owned()],
            &["bootstrap-text".to_owned()],
            &[ModuleEntry {
                key: "os",
                label: "OS",
                value: "Fedora Linux 43".to_owned(),
            }],
            0,
        );

        assert_eq!(report.contract_version, BOOTSTRAP_CONTRACT_VERSION);
        assert!(report.ready_for_foundations);
        assert!(report.checks.iter().all(|check| check.passed));
    }

    #[test]
    fn readiness_fails_when_os_flow_is_missing() {
        let report = evaluate_bootstrap_readiness(
            "corefetch",
            &["os".to_owned()],
            &["os".to_owned()],
            &["bootstrap-text".to_owned()],
            &[],
            0,
        );

        assert!(!report.ready_for_foundations);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.key == "snapshot-flow" && !check.passed)
        );
    }
}
