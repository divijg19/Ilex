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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryModule;

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

impl Module for CpuModule {
    fn key(&self) -> &'static str {
        "cpu"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.cpu.as_ref().map(|cpu| ModuleEntry {
            key: "cpu",
            label: "CPU",
            value: format!("{} ({} cores)", cpu.model_name, cpu.logical_cores),
        })
    }
}

impl Module for MemoryModule {
    fn key(&self) -> &'static str {
        "memory"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.memory.as_ref().map(|memory| ModuleEntry {
            key: "memory",
            label: "Memory",
            value: format_memory(memory.total_kib, memory.used_kib()),
        })
    }
}

pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn bootstrap() -> Self {
        Self {
            modules: vec![
                Box::new(OsModule),
                Box::new(CpuModule),
                Box::new(MemoryModule),
            ],
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

fn format_memory(total_kib: u64, used_kib: Option<u64>) -> String {
    let total_gib = kib_to_gib(total_kib);
    match used_kib {
        Some(used) => format!("{:.1} GiB / {:.1} GiB", kib_to_gib(used), total_gib),
        None => format!("{:.1} GiB total", total_gib),
    }
}

fn kib_to_gib(kib: u64) -> f64 {
    kib as f64 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::{CpuModule, MemoryModule, Module, OsModule};
    use crate::detectors::{CpuInfo, MemoryInfo, OsInfo, SystemSnapshot};

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
            ..SystemSnapshot::default()
        };

        let entry = module.collect(&snapshot).expect("os module should emit");

        assert_eq!(entry.label, "OS");
        assert_eq!(entry.value, "Fedora Linux 43");
    }

    #[test]
    fn cpu_module_uses_model_and_core_count() {
        let module = CpuModule;
        let snapshot = SystemSnapshot {
            cpu: Some(CpuInfo {
                model_name: "ExampleCore 9000".to_owned(),
                logical_cores: 4,
            }),
            ..SystemSnapshot::default()
        };

        let entry = module.collect(&snapshot).expect("cpu module should emit");

        assert_eq!(entry.label, "CPU");
        assert_eq!(entry.value, "ExampleCore 9000 (4 cores)");
    }

    #[test]
    fn memory_module_formats_used_and_total() {
        let module = MemoryModule;
        let snapshot = SystemSnapshot {
            memory: Some(MemoryInfo {
                total_kib: 32768000,
                available_kib: Some(24576000),
            }),
            ..SystemSnapshot::default()
        };

        let entry = module
            .collect(&snapshot)
            .expect("memory module should emit");

        assert_eq!(entry.label, "Memory");
        assert_eq!(entry.value, "7.8 GiB / 31.2 GiB");
    }
}
