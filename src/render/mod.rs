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

        [
            format!("corefetch {}", view.version),
            format!("binary: {}", view.binary_name),
            format!("alias: {}", view.alias),
            format!("args: {}", args),
            format!("config: {}", view.config_state),
            format!("detectors: {}", view.detectors.join(", ")),
            format!("modules: {}", view.modules.join(", ")),
            format!("renderers: {}", view.renderers.join(", ")),
            "status: v0.0.1 bootstrap scaffold ready".to_owned(),
        ]
        .join("\n")
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
