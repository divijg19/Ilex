pub trait Module {
    fn key(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderModule {
    key: &'static str,
}

impl PlaceholderModule {
    pub fn new(key: &'static str) -> Self {
        Self { key }
    }
}

impl Module for PlaceholderModule {
    fn key(&self) -> &'static str {
        self.key
    }
}

pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn bootstrap() -> Self {
        Self {
            modules: vec![Box::new(PlaceholderModule::new("os"))],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.modules.iter().map(|module| module.key()).collect()
    }
}
