use std::time::Duration;

use serde_json::{Value, json};

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
    pub terminal_context: Option<TerminalContextView>,
    pub timings: Vec<TimingEntry>,
    pub pipeline_duration: Duration,
    pub foundation_readiness: ReadinessView,
    pub baseline_readiness: ReadinessView,
    pub environment_readiness: ReadinessView,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessView {
    pub contract_version: String,
    pub ready: bool,
    pub checks: Vec<ReadinessCheckView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalContextView {
    pub name: String,
    pub term: Option<String>,
    pub color_term: Option<String>,
    pub capability: String,
    pub unicode: bool,
}

pub struct FetchRenderer;

impl Renderer for FetchRenderer {
    fn key(&self) -> &'static str {
        "fetch-text"
    }

    fn render(&self, view: &RenderView) -> String {
        let mut lines: Vec<String> = view
            .module_entries
            .iter()
            .map(|entry| format!("{}: {}", entry.label, entry.value))
            .collect();

        if lines.is_empty() {
            lines.push("No system data available".to_owned());
        }

        if !view.issues.is_empty() {
            lines.push(format!("Issues: {}", view.issues.len()));
        }

        lines.join("\n")
    }
}

pub struct MinimalRenderer;

impl Renderer for MinimalRenderer {
    fn key(&self) -> &'static str {
        "minimal-text"
    }

    fn render(&self, view: &RenderView) -> String {
        if view.module_entries.is_empty() {
            return "No system data available".to_owned();
        }

        view.module_entries
            .iter()
            .map(|entry| format!("{} {}", entry.label, entry.value))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn key(&self) -> &'static str {
        "json"
    }

    fn render(&self, view: &RenderView) -> String {
        let payload = json!({
            "program": "corefetch",
            "version": view.version,
            "binary": view.binary_name,
            "alias": view.alias,
            "primary_command": view.primary_command,
            "primary_entrypoint": view.is_primary_entrypoint,
            "args": view.raw_args,
            "config": view.config_state,
            "terminal": view.terminal_context.as_ref().map(|terminal| json!({
                "name": terminal.name.as_str(),
                "term": terminal.term.as_deref(),
                "color_term": terminal.color_term.as_deref(),
                "capability": terminal.capability.as_str(),
                "unicode": terminal.unicode,
            })),
            "modules": view.module_entries.iter().map(module_entry_to_json).collect::<Vec<_>>(),
            "issues": view.issues,
            "timings": {
                "pipeline_us": duration_to_micros(view.pipeline_duration),
                "entries": view.timings.iter().map(|timing| {
                    json!({
                        "label": timing.label,
                        "duration_us": duration_to_micros(timing.duration),
                    })
                }).collect::<Vec<_>>(),
            },
            "contracts": [
                readiness_to_json("foundation", &view.foundation_readiness),
                readiness_to_json("baseline", &view.baseline_readiness),
                readiness_to_json("environment", &view.environment_readiness),
            ],
        });

        serde_json::to_string_pretty(&payload).expect("json rendering should succeed")
    }
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

        let foundation_lines: Vec<String> = std::iter::once(format!(
            "contract: {}",
            view.foundation_readiness.contract_version
        ))
        .chain(std::iter::once(format!(
            "foundation readiness: {}",
            if view.foundation_readiness.ready {
                "ready"
            } else {
                "blocked"
            }
        )))
        .chain(view.foundation_readiness.checks.iter().map(|check| {
            format!(
                "foundation check: {}={} ({})",
                check.key,
                if check.passed { "pass" } else { "fail" },
                check.detail
            )
        }))
        .collect();

        let baseline_lines: Vec<String> = std::iter::once(format!(
            "contract: {}",
            view.baseline_readiness.contract_version
        ))
        .chain(std::iter::once(format!(
            "baseline readiness: {}",
            if view.baseline_readiness.ready {
                "ready"
            } else {
                "blocked"
            }
        )))
        .chain(view.baseline_readiness.checks.iter().map(|check| {
            format!(
                "baseline check: {}={} ({})",
                check.key,
                if check.passed { "pass" } else { "fail" },
                check.detail
            )
        }))
        .collect();

        let environment_lines: Vec<String> = std::iter::once(format!(
            "contract: {}",
            view.environment_readiness.contract_version
        ))
        .chain(std::iter::once(format!(
            "environment readiness: {}",
            if view.environment_readiness.ready {
                "ready"
            } else {
                "blocked"
            }
        )))
        .chain(view.environment_readiness.checks.iter().map(|check| {
            format!(
                "environment check: {}={} ({})",
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
            format!(
                "terminal context: {}",
                view.terminal_context
                    .as_ref()
                    .map(|terminal| {
                        format!(
                            "{} capability={} unicode={}",
                            terminal.name, terminal.capability, terminal.unicode
                        )
                    })
                    .unwrap_or_else(|| "<none>".to_owned())
            ),
        ];

        lines.extend(module_lines);
        lines.extend(issue_lines);
        lines.extend(timing_lines);
        lines.extend(foundation_lines);
        lines.extend(baseline_lines);
        lines.extend(environment_lines);
        lines.push(format!(
            "status: v{} capability groundwork active",
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

fn duration_to_micros(duration: Duration) -> u64 {
    let micros = duration.as_micros();
    if micros > u64::MAX as u128 {
        u64::MAX
    } else {
        micros as u64
    }
}

fn module_entry_to_json(entry: &ModuleEntry) -> Value {
    json!({
        "key": entry.key,
        "label": entry.label,
        "value": entry.value,
    })
}

fn readiness_to_json(kind: &str, readiness: &ReadinessView) -> Value {
    json!({
        "kind": kind,
        "contract_version": readiness.contract_version,
        "ready": readiness.ready,
        "checks": readiness.checks.iter().map(|check| {
            json!({
                "key": check.key,
                "passed": check.passed,
                "detail": check.detail,
            })
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::time::Duration;

    use super::{
        BootstrapRenderer, FetchRenderer, JsonRenderer, MinimalRenderer, ReadinessCheckView,
        ReadinessView, RenderView, Renderer, TerminalContextView, TimingEntry,
    };
    use crate::modules::ModuleEntry;

    #[test]
    fn renderer_includes_timing_lines() {
        let renderer = BootstrapRenderer;
        let view = RenderView {
            version: "0.3.0",
            binary_name: "corefetch".to_owned(),
            alias: "corefetch".to_owned(),
            primary_command: "corefetch".to_owned(),
            is_primary_entrypoint: true,
            raw_args: Vec::new(),
            config_state: "defaults (no config file found)".to_owned(),
            detectors: vec!["os".to_owned()],
            modules: vec!["os".to_owned()],
            renderers: vec!["bootstrap-text".to_owned()],
            terminal_context: Some(TerminalContextView {
                name: "Ghostty".to_owned(),
                term: Some("xterm-256color".to_owned()),
                color_term: Some("truecolor".to_owned()),
                capability: "truecolor".to_owned(),
                unicode: true,
            }),
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
                ModuleEntry {
                    key: "disk",
                    label: "Disk",
                    value: "31.2 GiB / 62.5 GiB (/)".to_owned(),
                },
                ModuleEntry {
                    key: "shell",
                    label: "Shell",
                    value: "fish".to_owned(),
                },
                ModuleEntry {
                    key: "terminal",
                    label: "Terminal",
                    value: "Ghostty (xterm-256color)".to_owned(),
                },
            ],
            timings: vec![TimingEntry {
                label: "detector.os".to_owned(),
                duration: Duration::from_micros(120),
            }],
            pipeline_duration: Duration::from_micros(250),
            foundation_readiness: ReadinessView {
                contract_version: "foundation-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            baseline_readiness: ReadinessView {
                contract_version: "baseline-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            environment_readiness: ReadinessView {
                contract_version: "environment-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            issues: Vec::new(),
        };

        let output = renderer.render(&view);

        assert!(output.contains("timing: pipeline=250 us"));
        assert!(output.contains("timing: detector.os=120 us"));
        assert!(output.contains("primary command: corefetch"));
        assert!(output.contains("primary entrypoint: true"));
        assert!(output.contains("terminal context: Ghostty capability=truecolor unicode=true"));
        assert!(output.contains("contract: foundation-v1"));
        assert!(output.contains("foundation readiness: ready"));
        assert!(output.contains("contract: baseline-v1"));
        assert!(output.contains("baseline readiness: ready"));
        assert!(output.contains("contract: environment-v1"));
        assert!(output.contains("environment readiness: ready"));
        assert!(
            output.contains("foundation check: snapshot-flow=pass (renderable module entries: 6)")
        );
        assert!(
            output.contains("baseline check: snapshot-flow=pass (renderable module entries: 6)")
        );
        assert!(
            output.contains("environment check: snapshot-flow=pass (renderable module entries: 6)")
        );
        assert!(output.contains("CPU: ExampleCore 9000 (4 cores)"));
        assert!(output.contains("Memory: 7.8 GiB / 31.2 GiB"));
        assert!(output.contains("Disk: 31.2 GiB / 62.5 GiB (/)"));
        assert!(output.contains("Shell: fish"));
        assert!(output.contains("Terminal: Ghostty (xterm-256color)"));
        assert!(output.contains("status: v0.3.0 capability groundwork active"));
    }

    #[test]
    fn fetch_renderer_emits_module_lines() {
        let renderer = FetchRenderer;
        let output = renderer.render(&sample_view());

        assert!(output.contains("OS: Fedora Linux 43"));
        assert!(output.contains("Disk: 31.2 GiB / 62.5 GiB (/)"));
    }

    #[test]
    fn minimal_renderer_collapses_to_single_line() {
        let renderer = MinimalRenderer;
        let output = renderer.render(&sample_view());

        assert!(output.contains("OS Fedora Linux 43 | CPU ExampleCore 9000 (4 cores)"));
        assert!(!output.contains('\n'));
    }

    #[test]
    fn json_renderer_includes_contracts_and_modules() {
        let renderer = JsonRenderer;
        let output = renderer.render(&sample_view());
        let parsed: Value = serde_json::from_str(&output).expect("json output should parse");

        assert_eq!(parsed["program"], "corefetch");
        assert_eq!(parsed["terminal"]["capability"], "truecolor");
        assert_eq!(parsed["terminal"]["unicode"], true);
        assert_eq!(parsed["contracts"][0]["contract_version"], "foundation-v1");
        assert_eq!(parsed["contracts"][1]["contract_version"], "baseline-v1");
        assert_eq!(parsed["contracts"][2]["contract_version"], "environment-v1");
        assert!(
            parsed["modules"]
                .as_array()
                .expect("modules should be an array")
                .iter()
                .any(|module| module["key"] == "terminal")
        );
    }

    fn sample_view() -> RenderView {
        RenderView {
            version: "0.3.0",
            binary_name: "corefetch".to_owned(),
            alias: "corefetch".to_owned(),
            primary_command: "corefetch".to_owned(),
            is_primary_entrypoint: true,
            raw_args: Vec::new(),
            config_state: "defaults (no config file found)".to_owned(),
            detectors: vec![
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
                "shell".to_owned(),
                "terminal".to_owned(),
            ],
            modules: vec![
                "os".to_owned(),
                "cpu".to_owned(),
                "memory".to_owned(),
                "disk".to_owned(),
                "shell".to_owned(),
                "terminal".to_owned(),
            ],
            renderers: vec![
                "bootstrap-text".to_owned(),
                "fetch-text".to_owned(),
                "minimal-text".to_owned(),
                "json".to_owned(),
            ],
            terminal_context: Some(TerminalContextView {
                name: "Ghostty".to_owned(),
                term: Some("xterm-256color".to_owned()),
                color_term: Some("truecolor".to_owned()),
                capability: "truecolor".to_owned(),
                unicode: true,
            }),
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
                ModuleEntry {
                    key: "disk",
                    label: "Disk",
                    value: "31.2 GiB / 62.5 GiB (/)".to_owned(),
                },
                ModuleEntry {
                    key: "shell",
                    label: "Shell",
                    value: "fish".to_owned(),
                },
                ModuleEntry {
                    key: "terminal",
                    label: "Terminal",
                    value: "Ghostty (xterm-256color)".to_owned(),
                },
            ],
            timings: vec![TimingEntry {
                label: "detector.os".to_owned(),
                duration: Duration::from_micros(120),
            }],
            pipeline_duration: Duration::from_micros(250),
            foundation_readiness: ReadinessView {
                contract_version: "foundation-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            baseline_readiness: ReadinessView {
                contract_version: "baseline-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            environment_readiness: ReadinessView {
                contract_version: "environment-v1".to_owned(),
                ready: true,
                checks: vec![ReadinessCheckView {
                    key: "snapshot-flow".to_owned(),
                    passed: true,
                    detail: "renderable module entries: 6".to_owned(),
                }],
            },
            issues: Vec::new(),
        }
    }
}

pub struct RendererRegistry {
    renderers: Vec<Box<dyn Renderer>>,
}

impl RendererRegistry {
    pub fn bootstrap() -> Self {
        Self {
            renderers: vec![
                Box::new(BootstrapRenderer),
                Box::new(FetchRenderer),
                Box::new(MinimalRenderer),
                Box::new(JsonRenderer),
            ],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.renderers
            .iter()
            .map(|renderer| renderer.key())
            .collect()
    }

    pub fn render(&self, key: &str, view: &RenderView) -> Option<String> {
        self.renderers
            .iter()
            .find(|renderer| renderer.key() == key)
            .map(|renderer| renderer.render(view))
    }
}
