use std::collections::BTreeMap;

use super::{CpuInfo, DiskMount, MemoryInfo, OsInfo};

pub(super) fn parse_os_release(content: &str) -> Result<OsInfo, String> {
    let values = parse_key_value_lines(content);
    let name = values
        .get("NAME")
        .cloned()
        .ok_or_else(|| "missing NAME in os-release".to_owned())?;
    let pretty_name = values
        .get("PRETTY_NAME")
        .cloned()
        .unwrap_or_else(|| name.clone());

    Ok(OsInfo {
        name,
        pretty_name,
        id: values.get("ID").cloned(),
        version_id: values.get("VERSION_ID").cloned(),
    })
}

pub(super) fn parse_cpu_info(content: &str) -> Result<CpuInfo, String> {
    let mut logical_cores = 0usize;
    let mut logical_cores_fallback: Option<usize> = None;
    let mut model_name: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        if key == "processor" {
            logical_cores += 1;
        }

        if model_name.is_none()
            && matches!(key, "model name" | "Hardware" | "Processor")
            && !value.is_empty()
        {
            model_name = Some(value.to_owned());
        }

        if logical_cores_fallback.is_none() && key == "cpu cores" {
            if let Ok(parsed) = value.parse::<usize>() {
                if parsed > 0 {
                    logical_cores_fallback = Some(parsed);
                }
            }
        }
    }

    let logical_cores = if logical_cores > 0 {
        logical_cores
    } else {
        logical_cores_fallback.unwrap_or(0)
    };
    if logical_cores == 0 {
        return Err("missing processor or cpu cores entries in cpuinfo".to_owned());
    }

    let model_name = model_name.ok_or_else(|| "missing model name in cpuinfo".to_owned())?;

    Ok(CpuInfo {
        model_name,
        logical_cores,
    })
}

pub(super) fn parse_meminfo(content: &str) -> Result<MemoryInfo, String> {
    let values = parse_proc_value_lines(content)?;
    let total_kib = values
        .get("MemTotal")
        .copied()
        .ok_or_else(|| "missing MemTotal in meminfo".to_owned())?;
    let available_kib = values
        .get("MemAvailable")
        .copied()
        .or_else(|| values.get("MemFree").copied());

    Ok(MemoryInfo {
        total_kib,
        available_kib,
    })
}

pub(super) fn parse_primary_mount(content: &str) -> Result<DiskMount, String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(device) = parts.next() else {
            continue;
        };
        let Some(mount_point) = parts.next() else {
            continue;
        };
        let Some(filesystem) = parts.next() else {
            continue;
        };

        let mount_point = normalize_mount_field(mount_point);
        if mount_point == "/" {
            return Ok(DiskMount {
                device: normalize_mount_field(device),
                mount_point,
                filesystem: normalize_mount_field(filesystem),
            });
        }
    }

    Err("missing root filesystem entry in mounts".to_owned())
}

fn parse_key_value_lines(content: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        values.insert(key.to_owned(), normalize_value(value));
    }

    values
}

fn parse_proc_value_lines(content: &str) -> Result<BTreeMap<String, u64>, String> {
    let mut values = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let numeric = value
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("missing numeric value for {key}"))?
            .parse::<u64>()
            .map_err(|error| format!("invalid numeric value for {key}: {error}"))?;

        values.insert(key.trim().to_owned(), numeric);
    }

    Ok(values)
}

fn normalize_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace(r#"\""#, "\"")
        .replace(r#"\\"#, "\\")
}

fn normalize_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}
