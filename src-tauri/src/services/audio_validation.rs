use std::path::Path;

use anyhow::{bail, Result};

/// Supported audio file extensions (matching symphonia features in Cargo.toml).
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "ogg", "flac", "m4a", "aac", "mp4", "wma", "webm",
];

/// Maximum recommended duration in seconds (4 hours).
const MAX_RECOMMENDED_DURATION_SECS: f64 = 4.0 * 60.0 * 60.0;

/// Minimum free disk space in bytes required before processing (500 MB).
const MIN_FREE_DISK_BYTES: u64 = 500 * 1024 * 1024;

/// Result of audio file validation.
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
}

/// Validate that a file has a supported audio extension.
pub fn validate_format(file_path: &Path) -> Result<()> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext {
        Some(ref e) if SUPPORTED_EXTENSIONS.contains(&e.as_str()) => Ok(()),
        Some(e) => bail!(
            "InvalidFormat: unsupported audio format '.{}'. Supported formats: {}",
            e,
            SUPPORTED_EXTENSIONS.join(", ")
        ),
        None => bail!(
            "InvalidFormat: file has no extension. Supported formats: {}",
            SUPPORTED_EXTENSIONS.join(", ")
        ),
    }
}

/// Check if a file exists and is readable.
pub fn validate_file_exists(file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        bail!("FileNotFound: {}", file_path.display());
    }
    if !file_path.is_file() {
        bail!("FileNotFound: {} is not a file", file_path.display());
    }
    Ok(())
}

/// Check if duration exceeds the recommended maximum.
/// Returns a warning message if so, None otherwise.
pub fn check_duration_warning(duration_seconds: f64) -> Option<String> {
    if duration_seconds > MAX_RECOMMENDED_DURATION_SECS {
        let hours = duration_seconds / 3600.0;
        Some(format!(
            "This file is {:.1} hours long. Files over 4 hours may take a very long time to transcribe.",
            hours
        ))
    } else {
        None
    }
}

/// Check available disk space at the given path.
/// Returns an error if below the minimum threshold.
pub fn check_disk_space(path: &Path) -> Result<()> {
    let available = fs_available_space(path);
    match available {
        Some(bytes) if bytes < MIN_FREE_DISK_BYTES => {
            let available_mb = bytes / (1024 * 1024);
            bail!(
                "DiskSpaceLow: only {} MB available. At least {} MB is recommended for processing.",
                available_mb,
                MIN_FREE_DISK_BYTES / (1024 * 1024)
            );
        }
        _ => Ok(()),
    }
}

/// Get available space on the filesystem containing the given path.
/// Returns None if the check is not supported on the platform.
fn fs_available_space(path: &Path) -> Option<u64> {
    // Use std::fs::metadata approach -- get the parent dir that exists
    let check_path = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        let c_path = CString::new(check_path.to_str()?).ok()?;
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        let ret = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
        if ret == 0 {
            let stat = unsafe { stat.assume_init() };
            Some(stat.f_bavail as u64 * stat.f_frsize as u64)
        } else {
            None
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;

        let wide: Vec<u16> = OsStr::new(check_path.to_str()?)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let mut free_bytes: u64 = 0;
        let ret = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes as *mut u64,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if ret != 0 {
            Some(free_bytes)
        } else {
            None
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Run all validations on an audio file before upload/processing.
/// Returns warnings (non-fatal) or errors (fatal).
pub fn validate_audio_file(file_path: &Path, app_data_dir: &Path) -> Result<Vec<String>> {
    validate_file_exists(file_path)?;
    validate_format(file_path)?;
    check_disk_space(app_data_dir)?;

    Ok(vec![])
}

/// Run validations and include duration-based warnings.
pub fn validate_audio_with_duration(
    file_path: &Path,
    app_data_dir: &Path,
    duration_seconds: f64,
) -> Result<Vec<String>> {
    let mut warnings = validate_audio_file(file_path, app_data_dir)?;

    if let Some(w) = check_duration_warning(duration_seconds) {
        warnings.push(w);
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validate_format_supported() {
        for ext in SUPPORTED_EXTENSIONS {
            let path = PathBuf::from(format!("test.{}", ext));
            assert!(validate_format(&path).is_ok(), "should support .{}", ext);
        }
    }

    #[test]
    fn test_validate_format_case_insensitive() {
        let path = PathBuf::from("test.MP3");
        assert!(validate_format(&path).is_ok());
        let path = PathBuf::from("test.WaV");
        assert!(validate_format(&path).is_ok());
    }

    #[test]
    fn test_validate_format_unsupported() {
        let path = PathBuf::from("test.txt");
        let err = validate_format(&path).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("InvalidFormat"));
        assert!(msg.contains("unsupported"));
        assert!(msg.contains("mp3")); // should list supported formats
    }

    #[test]
    fn test_validate_format_no_extension() {
        let path = PathBuf::from("noextension");
        let err = validate_format(&path).unwrap_err();
        assert!(err.to_string().contains("no extension"));
    }

    #[test]
    fn test_validate_file_not_found() {
        let path = PathBuf::from("/nonexistent/audio.mp3");
        let err = validate_file_exists(&path).unwrap_err();
        assert!(err.to_string().contains("FileNotFound"));
    }

    #[test]
    fn test_duration_warning_short_file() {
        assert!(check_duration_warning(3600.0).is_none()); // 1 hour
        assert!(check_duration_warning(14399.0).is_none()); // just under 4 hours
    }

    #[test]
    fn test_duration_warning_long_file() {
        let warning = check_duration_warning(14401.0); // just over 4 hours
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("4 hours"));
    }

    #[test]
    fn test_duration_warning_very_long_file() {
        let warning = check_duration_warning(36000.0); // 10 hours
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("10.0 hours"));
    }

    #[test]
    fn test_supported_extensions_list() {
        // Verify we have the major formats
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"wav"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"ogg"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"m4a"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"flac"));
    }

    #[cfg(unix)]
    #[test]
    fn test_disk_space_check_passes_on_tmp() {
        // /tmp should have plenty of space in most environments
        assert!(check_disk_space(Path::new("/tmp")).is_ok());
    }
}
