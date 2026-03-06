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
    pub raw_args: Vec<String>,
    pub config_state: String,
    pub detectors: Vec<String>,
    pub modules: Vec<String>,
    pub renderers: Vec<String>,
    pub module_entries: Vec<ModuleEntry>,
    pub issues: Vec<String>,
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

        let mut lines = vec![
            format!("corefetch {}", view.version),
            format!("binary: {}", view.binary_name),
            format!("alias: {}", view.alias),
            format!("args: {}", args),
            format!("config: {}", view.config_state),
            format!("detectors: {}", view.detectors.join(", ")),
            format!("modules: {}", view.modules.join(", ")),
            format!("renderers: {}", view.renderers.join(", ")),
        ];

        lines.extend(module_lines);
        lines.extend(issue_lines);
        lines.push("status: v0.0.2 os detection pipeline ready".to_owned());
        lines.join("\n")
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
