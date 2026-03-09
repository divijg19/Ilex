use crate::detectors::SystemSnapshot;
use crate::formatting::{
    format_core_count, format_disk_usage, format_memory_usage, format_terminal_identity,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalModule;

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
            value: format!(
                "{} ({})",
                cpu.model_name,
                format_core_count(cpu.logical_cores)
            ),
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
            value: format_memory_usage(memory.total_kib, memory.used_kib()),
        })
    }
}

impl Module for DiskModule {
    fn key(&self) -> &'static str {
        "disk"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.disk.as_ref().map(|disk| ModuleEntry {
            key: "disk",
            label: "Disk",
            value: format_disk_usage(disk.total_kib, disk.used_kib(), &disk.mount_point),
        })
    }
}

impl Module for ShellModule {
    fn key(&self) -> &'static str {
        "shell"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.shell.as_ref().map(|shell| ModuleEntry {
            key: "shell",
            label: "Shell",
            value: shell.name.clone(),
        })
    }
}

impl Module for TerminalModule {
    fn key(&self) -> &'static str {
        "terminal"
    }

    fn collect(&self, snapshot: &SystemSnapshot) -> Option<ModuleEntry> {
        snapshot.terminal.as_ref().map(|terminal| ModuleEntry {
            key: "terminal",
            label: "Terminal",
            value: format_terminal_identity(&terminal.name, terminal.term.as_deref()),
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
                Box::new(DiskModule),
                Box::new(ShellModule),
                Box::new(TerminalModule),
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

#[cfg(test)]
mod tests {
    use super::{
        CpuModule, DiskModule, MemoryModule, Module, OsModule, ShellModule, TerminalModule,
    };
    use crate::detectors::{
        CpuInfo, DiskInfo, MemoryInfo, OsInfo, ShellInfo, SystemSnapshot, TerminalCapability,
        TerminalInfo,
    };

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
                free_kib: Some(12288000),
                buffers_kib: Some(1024000),
                cached_kib: Some(5632000),
            }),
            ..SystemSnapshot::default()
        };

        let entry = module
            .collect(&snapshot)
            .expect("memory module should emit");

        assert_eq!(entry.label, "Memory");
        assert_eq!(entry.value, "7.8 GiB / 31.2 GiB");
    }

    #[test]
    fn disk_module_formats_used_total_and_mount_point() {
        let module = DiskModule;
        let snapshot = SystemSnapshot {
            disk: Some(DiskInfo {
                device: "/dev/nvme0n1p2".to_owned(),
                filesystem: "ext4".to_owned(),
                mount_point: "/".to_owned(),
                total_kib: 65536000,
                available_kib: Some(32768000),
            }),
            ..SystemSnapshot::default()
        };

        let entry = module.collect(&snapshot).expect("disk module should emit");

        assert_eq!(entry.label, "Disk");
        assert_eq!(entry.value, "31.2 GiB / 62.5 GiB (/)");
    }

    #[test]
    fn shell_module_uses_shell_name() {
        let module = ShellModule;
        let snapshot = SystemSnapshot {
            shell: Some(ShellInfo {
                executable_path: "/usr/bin/fish".to_owned(),
                name: "fish".to_owned(),
            }),
            ..SystemSnapshot::default()
        };

        let entry = module.collect(&snapshot).expect("shell module should emit");

        assert_eq!(entry.label, "Shell");
        assert_eq!(entry.value, "fish");
    }

    #[test]
    fn terminal_module_formats_program_and_term() {
        let module = TerminalModule;
        let snapshot = SystemSnapshot {
            terminal: Some(TerminalInfo {
                name: "Ghostty".to_owned(),
                term: Some("xterm-256color".to_owned()),
                color_term: Some("truecolor".to_owned()),
                capability: TerminalCapability::Truecolor,
                unicode: true,
            }),
            ..SystemSnapshot::default()
        };

        let entry = module
            .collect(&snapshot)
            .expect("terminal module should emit");

        assert_eq!(entry.label, "Terminal");
        assert_eq!(entry.value, "Ghostty (xterm-256color)");
    }
}
