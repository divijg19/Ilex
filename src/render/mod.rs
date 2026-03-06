use std::time::Duration;

use crate::modules::ModuleEntry;

pub trait Renderer {
    fn key(&self) -> &'static str;
    fn render(&self, view: &RenderView) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderView {
    pub version: &'static str,
    pub binary_name: String,
    pub alias: String,
    pub primary_command: String,
    pub is_primary_entrypoint: bool,
    pub raw_args: Vec<String>,
    pub config_state: String,
    pub detectors: Vec<String>,
    pub modules: Vec<String>,
    pub renderers: Vec<String>,
    pub module_entries: Vec<ModuleEntry>,
    pub timings: Vec<TimingEntry>,
    pub pipeline_duration: Duration,
    pub contract_version: String,
    pub ready_for_foundations: bool,
    pub readiness_checks: Vec<ReadinessCheckView>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimingEntry {
    pub label: String,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessCheckView {
    pub key: String,
    pub passed: bool,
    pub detail: String,
}

pub struct BootstrapRenderer;

impl Renderer for BootstrapRenderer {
    fn key(&self) -> &'static str {
        "bootstrap-text"
    }

    fn render(&self, view: &RenderView) -> String {
        let args = if view.raw_args.is_empty() {
            "<none>".to_owned()
        } else {
            view.raw_args.join(" ")
        };

        let module_lines: Vec<String> = if view.module_entries.is_empty() {
            vec!["module output: <none>".to_owned()]
        } else {
            view.module_entries
                .iter()
                .map(|entry| format!("{}: {}", entry.label, entry.value))
                .collect()
        };

        let issue_lines: Vec<String> = if view.issues.is_empty() {
            vec!["issues: <none>".to_owned()]
        } else {
            view.issues
                .iter()
                .map(|issue| format!("issue: {issue}"))
                .collect()
        };

        let timing_lines: Vec<String> = std::iter::once(format!(
            "timing: pipeline={}",
            format_duration(view.pipeline_duration)
        ))
        .chain(view.timings.iter().map(|timing| {
            format!(
                "timing: {}={}",
                timing.label,
                format_duration(timing.duration)
            )
        }))
        .collect();

        let readiness_lines: Vec<String> =
            std::iter::once(format!("contract: {}", view.contract_version))
                .chain(std::iter::once(format!(
                    "foundation readiness: {}",
                    if view.ready_for_foundations {
                        "ready"
                    } else {
                        "blocked"
                    }
                )))
                .chain(view.readiness_checks.iter().map(|check| {
                    format!(
                        "readiness: {}={} ({})",
                        check.key,
                        if check.passed { "pass" } else { "fail" },
                        check.detail
                    )
                }))
                .collect();

        let mut lines = vec![
            format!("corefetch {}", view.version),
            format!("binary: {}", view.binary_name),
            format!("alias: {}", view.alias),
            format!("primary command: {}", view.primary_command),
            format!("primary entrypoint: {}", view.is_primary_entrypoint),
            format!("args: {}", args),
            format!("config: {}", view.config_state),
            format!("detectors: {}", view.detectors.join(", ")),
            format!("modules: {}", view.modules.join(", ")),
            format!("renderers: {}", view.renderers.join(", ")),
        ];

        lines.extend(module_lines);
        lines.extend(issue_lines);
        lines.extend(timing_lines);
        lines.extend(readiness_lines);
        lines.push(format!(
            "status: v{} foundation stabilization active",
            view.version
        ));
        lines.join("\n")
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_micros() < 1_000 {
        format!("{} us", duration.as_micros())
    } else {
        format!("{:.3} ms", duration.as_secs_f64() * 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{BootstrapRenderer, ReadinessCheckView, RenderView, Renderer, TimingEntry};
    use crate::modules::ModuleEntry;

    #[test]
    fn renderer_includes_timing_lines() {
        let renderer = BootstrapRenderer;
        let view = RenderView {
            version: "0.1.1",
            binary_name: "corefetch".to_owned(),
            alias: "corefetch".to_owned(),
            primary_command: "corefetch".to_owned(),
            is_primary_entrypoint: true,
            raw_args: Vec::new(),
            config_state: "bootstrap".to_owned(),
            detectors: vec!["os".to_owned()],
            modules: vec!["os".to_owned()],
            renderers: vec!["bootstrap-text".to_owned()],
            module_entries: vec![
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
            timings: vec![TimingEntry {
                label: "detector.os".to_owned(),
                duration: Duration::from_micros(120),
            }],
            pipeline_duration: Duration::from_micros(250),
            contract_version: "foundation-v1".to_owned(),
            ready_for_foundations: true,
            readiness_checks: vec![ReadinessCheckView {
                key: "snapshot-flow".to_owned(),
                passed: true,
                detail: "renderable module entries: 3".to_owned(),
            }],
            issues: Vec::new(),
        };

        let output = renderer.render(&view);

        assert!(output.contains("timing: pipeline=250 us"));
        assert!(output.contains("timing: detector.os=120 us"));
        assert!(output.contains("primary command: corefetch"));
        assert!(output.contains("primary entrypoint: true"));
        assert!(output.contains("contract: foundation-v1"));
        assert!(output.contains("foundation readiness: ready"));
        assert!(output.contains("readiness: snapshot-flow=pass (renderable module entries: 3)"));
        assert!(output.contains("CPU: ExampleCore 9000 (4 cores)"));
        assert!(output.contains("Memory: 7.8 GiB / 31.2 GiB"));
        assert!(output.contains("status: v0.1.1 foundation stabilization active"));
    }
}

pub struct RendererRegistry {
    renderers: Vec<Box<dyn Renderer>>,
}

impl RendererRegistry {
    pub fn bootstrap() -> Self {
        Self {
            renderers: vec![Box::new(BootstrapRenderer)],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.renderers
            .iter()
            .map(|renderer| renderer.key())
            .collect()
    }
}
