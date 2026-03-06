#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigState {
    description: &'static str,
}

impl ConfigState {
    pub fn bootstrap_defaults() -> Self {
        Self {
            description: "foundation defaults (config loading deferred to 0.2.0)",
        }
    }

    pub fn description(&self) -> &'static str {
        self.description
    }
}
