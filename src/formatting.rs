pub fn format_core_count(logical_cores: usize) -> String {
    let unit = if logical_cores == 1 { "core" } else { "cores" };
    format!("{logical_cores} {unit}")
}

pub fn format_gib_from_kib(kib: u64) -> String {
    format!("{:.1} GiB", kib_to_gib(kib))
}

pub fn format_memory_usage(total_kib: u64, used_kib: Option<u64>) -> String {
    match used_kib {
        Some(used) => format!(
            "{} / {}",
            format_gib_from_kib(used),
            format_gib_from_kib(total_kib)
        ),
        None => format!("{} total", format_gib_from_kib(total_kib)),
    }
}

fn kib_to_gib(kib: u64) -> f64 {
    kib as f64 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::{format_core_count, format_gib_from_kib, format_memory_usage};

    #[test]
    fn formats_core_count() {
        assert_eq!(format_core_count(1), "1 core");
        assert_eq!(format_core_count(8), "8 cores");
    }

    #[test]
    fn formats_gib_from_kib() {
        assert_eq!(format_gib_from_kib(32768000), "31.2 GiB");
    }

    #[test]
    fn formats_memory_usage_with_used_value() {
        assert_eq!(
            format_memory_usage(32768000, Some(8192000)),
            "7.8 GiB / 31.2 GiB"
        );
    }

    #[test]
    fn formats_memory_usage_without_used_value() {
        assert_eq!(format_memory_usage(16384000, None), "15.6 GiB total");
    }
}
