use std::time::Instant;

use crate::VERSION;
use crate::cli::Invocation;
use crate::config::ConfigState;
use crate::contracts::evaluate_foundation_readiness;
use crate::detectors::DetectorRegistry;
use crate::modules::ModuleRegistry;
use crate::render::{
    BootstrapRenderer, ReadinessCheckView, RenderView, Renderer, RendererRegistry, TimingEntry,
};

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
        let detector_keys: Vec<String> = self
            .detectors
            .keys()
            .iter()
            .map(ToString::to_string)
            .collect();
        let module_keys: Vec<String> = self
            .modules
            .keys()
            .iter()
            .map(ToString::to_string)
            .collect();
        let renderer_keys: Vec<String> = self
            .renderers
            .keys()
            .iter()
            .map(ToString::to_string)
            .collect();
        let readiness = evaluate_foundation_readiness(
            self.invocation.canonical_command(),
            &detector_keys,
            &module_keys,
            &renderer_keys,
            &module_entries,
            detection.issues.len(),
        );
        let view = RenderView {
            version: VERSION,
            binary_name: self.invocation.binary_name().to_owned(),
            alias: self.invocation.alias_name().to_owned(),
            primary_command: self.invocation.canonical_command().to_owned(),
            is_primary_entrypoint: self.invocation.is_primary_entrypoint(),
            raw_args: self.invocation.user_args().to_vec(),
            config_state: self.config.description().to_owned(),
            detectors: detector_keys,
            modules: module_keys,
            renderers: renderer_keys,
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
            contract_version: readiness.contract_version.to_owned(),
            ready_for_foundations: readiness.ready_for_foundations,
            readiness_checks: readiness
                .checks
                .iter()
                .map(|check| ReadinessCheckView {
                    key: check.key.to_owned(),
                    passed: check.passed,
                    detail: check.detail.clone(),
                })
                .collect(),
            issues: detection
                .issues
                .iter()
                .map(|issue| {
                    format!(
                        "{} [{}]: {}",
                        issue.detector_key,
                        issue.kind.as_str(),
                        issue.message
                    )
                })
                .collect(),
        };

        self.renderer.render(&view)
    }
}
