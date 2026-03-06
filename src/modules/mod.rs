use crate::detectors::SystemSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEntry {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
}

pub trait Module {
    fn key(&self) -> &'static str;
    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsModule;

impl Module for OsModule {
    fn key(&self) -> &'static str {
        "os"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.os.as_ref().map(|os| ModuleEntry {
            key: "os",
            label: "OS",
            value: os.pretty_name.clone(),
        })
    }
}

pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn bootstrap() -> Self {
        Self {
            modules: vec![Box::new(OsModule)],
        }
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.modules.iter().map(|module| module.key()).collect()
    }

    pub fn collect(&self, snapshot: &SystemSnapshot) -> Vec<ModuleEntry> {
        self.modules
            .iter()
            .filter_map(|module| module.collect(snapshot))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Module, OsModule};
    use crate::detectors::{OsInfo, SystemSnapshot};

    #[test]
    fn os_module_uses_pretty_name() {
        let module = OsModule;
        let snapshot = SystemSnapshot {
            os: Some(OsInfo {
                name: "Fedora Linux".to_owned(),
                pretty_name: "Fedora Linux 43".to_owned(),
                id: Some("fedora".to_owned()),
                version_id: Some("43".to_owned()),
            }),
        };

        let entry = module.collect(&snapshot).expect("os module should emit");

        assert_eq!(entry.label, "OS");
        assert_eq!(entry.value, "Fedora Linux 43");
    }
}
