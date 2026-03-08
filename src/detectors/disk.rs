use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::parsers::parse_primary_mount;
use super::{DetectionError, Detector, DiskInfo, SystemSnapshot, map_parse_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskDetector {
    mounts_source: PathBuf,
}

impl Default for DiskDetector {
    fn default() -> Self {
        Self {
            mounts_source: PathBuf::from("/proc/mounts"),
        }
    }
}

impl DiskDetector {
    pub fn new(mounts_source: PathBuf) -> Self {
        Self { mounts_source }
    }
}

impl Detector for DiskDetector {
    fn key(&self) -> &'static str {
        "disk"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        let content = fs::read_to_string(&self.mounts_source).map_err(|error| {
            DetectionError::io(
                self.key(),
                format!("failed to read {}: {error}", self.mounts_source.display()),
            )
        })?;
        let mount = parse_primary_mount(&content)
            .map_err(|message| map_parse_error(self.key(), message))?;
        let (total_kib, available_kib) = filesystem_usage(Path::new(&mount.mount_point))
            .map_err(|message| DetectionError::io(self.key(), message))?;

        snapshot.disk = Some(DiskInfo {
            device: mount.device,
            filesystem: mount.filesystem,
            mount_point: mount.mount_point,
            total_kib,
            available_kib: Some(available_kib),
        });
        Ok(())
    }
}

fn filesystem_usage(path: &Path) -> Result<(u64, u64), String> {
    let raw_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| format!("invalid filesystem path: {}", path.display()))?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let result = unsafe { libc::statvfs(raw_path.as_ptr(), stats.as_mut_ptr()) };
    if result != 0 {
        return Err(format!(
            "failed to inspect filesystem {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }

    let stats = unsafe { stats.assume_init() };
    let block_size = if stats.f_frsize > 0 {
        stats.f_frsize as u128
    } else {
        stats.f_bsize as u128
    };
    let total_bytes = (stats.f_blocks as u128).saturating_mul(block_size);
    let available_bytes = (stats.f_bavail as u128).saturating_mul(block_size);

    Ok((bytes_to_kib(total_bytes), bytes_to_kib(available_bytes)))
}

fn bytes_to_kib(value: u128) -> u64 {
    let kib = value / 1024;
    if kib > u64::MAX as u128 {
        u64::MAX
    } else {
        kib as u64
    }
}
