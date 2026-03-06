use std::time::Instant;

use crate::VERSION;
use crate::cli::Invocation;
use crate::config::ConfigState;
use crate::detectors::DetectorRegistry;
use crate::modules::ModuleRegistry;
use crate::render::{BootstrapRenderer, RenderView, Renderer, RendererRegistry, TimingEntry};

pub struct App {
    invocation: Invocation,
    config: ConfigState,
    detectors: DetectorRegistry,
    modules: ModuleRegistry,
    renderers: RendererRegistry,
    renderer: BootstrapRenderer,
}

impl App {
    pub fn bootstrap(invocation: Invocation) -> Self {
        Self {
            invocation,
            config: ConfigState::bootstrap_defaults(),
            detectors: DetectorRegistry::bootstrap(),
            modules: ModuleRegistry::bootstrap(),
            renderers: RendererRegistry::bootstrap(),
            renderer: BootstrapRenderer,
        }
    }

    pub fn run(&self) -> String {
        let started_at = Instant::now();
        let detection = self.detectors.detect_all();
        let module_entries = self.modules.collect(&detection.snapshot);
        let view = RenderView {
            version: VERSION,
            binary_name: self.invocation.binary_name().to_owned(),
            alias: self.invocation.alias_name().to_owned(),
            primary_command: self.invocation.canonical_command().to_owned(),
            is_primary_entrypoint: self.invocation.is_primary_entrypoint(),
            raw_args: self.invocation.user_args().to_vec(),
            config_state: self.config.description().to_owned(),
            detectors: self
                .detectors
                .keys()
                .iter()
                .map(ToString::to_string)
                .collect(),
            modules: self
                .modules
                .keys()
                .iter()
                .map(ToString::to_string)
                .collect(),
            renderers: self
                .renderers
                .keys()
                .iter()
                .map(ToString::to_string)
                .collect(),
            module_entries,
            timings: {
                let mut timings = vec![TimingEntry {
                    label: "detection.total".to_owned(),
                    duration: detection.total_detection_time,
                }];
                timings.extend(detection.timings.iter().map(|timing| TimingEntry {
                    label: format!("detector.{}", timing.detector_key),
                    duration: timing.elapsed,
                }));
                timings
            },
            pipeline_duration: started_at.elapsed(),
            issues: detection
                .issues
                .iter()
                .map(|issue| format!("{}: {}", issue.detector_key, issue.message))
                .collect(),
        };

        self.renderer.render(&view)
    }
}
