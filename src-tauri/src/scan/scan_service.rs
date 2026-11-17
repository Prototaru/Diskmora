use super::{scan_model::*, archive_service, media_signature, base_signature};
use anyhow::{bail, Result};
use std::io::Read;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use walkdir::WalkDir;

pub fn file_types() -> Vec<FileTypeOption> {
    base_signature::FORMATS.iter().enumerate()
        .map(|(code, label)| FileTypeOption { code: code as i32, label: (*label).to_owned() })
        .chain(media_signature::EXTRA_FORMATS.iter().enumerate().map(|(index, label)| FileTypeOption { code: 17 + index as i32, label: (*label).to_owned() }))
        .collect()
}

pub fn scan(path: &str, types: &[i32], cancel: &AtomicBool, mut publish: impl FnMut(ScanProgress) -> bool) -> Result<ScanProgress> {
    let root = match Path::new(path).canonicalize() {
        Ok(root) => root,
        Err(error) if is_permission_error(&error) => {
            let mut result = ScanProgress::default();
            result.unread_directories.push(path.to_owned());
            record_error(&mut result, format!("{path}: {error}"), Some(&error));
            return Ok(result);
        }
        Err(error) => return Err(error.into()),
    };
    if !root.is_dir() {
        bail!("Select an existing folder");
    }
    if types.is_empty() || types.iter().any(|code| !file_types().iter().any(|option| option.code == *code)) {
        bail!("Select at least one supported file type");
    }
    let allowed: Vec<String> = file_types().into_iter().filter(|option| types.contains(&option.code)).map(|option| option.label).collect();
    let mut result = ScanProgress::default();
    let mut last = Instant::now();
    for entry in WalkDir::new(root).follow_links(false) {
        if cancel.load(Ordering::SeqCst) {
            result.cancelled = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                if let Some(path) = error.path() {
                    result.unread_directories.push(path.to_string_lossy().into_owned());
                    result.current_path = path.to_string_lossy().into_owned();
                }
                record_error(&mut result, error.to_string(), error.io_error());
                if last.elapsed() >= Duration::from_millis(150) {
                    if !publish(result.clone()) { result.cancelled = true; break; }
                    last = Instant::now();
                }
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        result.scanned += 1;
        result.current_path = entry.path().to_string_lossy().into_owned();
        match inspect(entry.path()) {
            Ok(file) => {
                let archive_format = file.format.clone();
                if allowed.contains(&file.format) { record_match(&mut result, file); }
                if matches!(archive_format.as_str(), "ZIP" | "RAR") {
                    let outcome = archive_service::scan_archive(entry.path(), &archive_format, |value| {
                        if cancel.load(Ordering::SeqCst) { result.cancelled = true; return false; }
                        result.scanned += 1;
                        match value {
                            Ok(file) => {
                                result.current_path = file.path.clone();
                                if allowed.contains(&file.format) { record_match(&mut result, file); }
                            }
                            Err(error) => record_error(&mut result, format!("{error:#}"), None),
                        }
                        if last.elapsed() >= Duration::from_millis(150) {
                            last = Instant::now();
                            if !publish(result.clone()) { result.cancelled = true; return false; }
                        }
                        true
                    });
                    if let Err(error) = outcome { record_error(&mut result, format!("{error:#}"), None); }
                    if result.cancelled { break; }
                }
            }
            Err(error) => record_error(&mut result, format!("{}: {error}", entry.path().display()), Some(&error)),
        }
        if last.elapsed() >= Duration::from_millis(150) {
            if !publish(result.clone()) {
                result.cancelled = true;
                break;
            }
            last = Instant::now();
        }
    }
    result.cancelled |= cancel.load(Ordering::SeqCst);
    Ok(result)
}

fn inspect(path: &Path) -> std::io::Result<ScanFile> {
    let mut head = Vec::with_capacity(media_signature::HEADER_BYTES);
    std::fs::File::open(path)?.take(media_signature::HEADER_BYTES as u64).read_to_end(&mut head)?;
    Ok(ScanFile {
        path: path.to_string_lossy().into_owned(),
        format: media_signature::detect(&head),
        size: path.metadata()?.len(),
        archive_path: None, entry_path: None,
    })
}

fn record_match(result: &mut ScanProgress, file: ScanFile) {
    result.matched += 1;
    result.bytes = result.bytes.saturating_add(file.size);
    if result.files.len() < 2_000 { result.files.push(file); }
}

fn record_error(result: &mut ScanProgress, error: String, cause: Option<&std::io::Error>) {
    result.skipped += 1;
    if cause.is_some_and(is_permission_error) {
        result.permission_denied += 1;
    }
    if result.errors.len() < 20 {
        result.errors.push(error);
    }
}

fn is_permission_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::PermissionDenied
        || (cfg!(target_os = "macos") && error.raw_os_error() == Some(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn counts_permission_failures_separately() {
        let mut result = ScanProgress::default();
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        record_error(&mut result, "denied".into(), Some(&denied));
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
        record_error(&mut result, "missing".into(), Some(&missing));
        assert_eq!((result.skipped, result.permission_denied), (2, 1));
        assert_eq!(result.errors.len(), 2);
        #[cfg(target_os = "macos")]
        assert!(is_permission_error(&std::io::Error::from_raw_os_error(1)));
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_directory_does_not_stop_other_files() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("diskmora-denied-{}", std::process::id()));
        let blocked = root.join("blocked");
        fs::create_dir_all(&blocked).unwrap();
        let expected = blocked.canonicalize().unwrap().to_string_lossy().into_owned();
        fs::write(root.join("readable.pdf"), b"%PDF-1.7").unwrap();
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).unwrap();
        let result = scan(root.to_str().unwrap(), &[4], &AtomicBool::new(false), |_| true);
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(&root).unwrap();
        let result = result.unwrap();
        assert_eq!(result.matched, 1);
        assert_eq!(result.permission_denied, 1);
        assert_eq!(result.unread_directories, vec![expected]);
        assert!(!result.cancelled);
    }

    #[test]
    fn scans_headers_recursively_without_extension_fallback() {
        let root = std::env::temp_dir().join(format!("diskmora-{}", std::process::id()));
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("nested/renamed.txt"), b"\x89PNG\r\n\x1a\n").unwrap();
        fs::write(root.join("fake.png"), b"plain text").unwrap();
        fs::create_dir_all(root.join(".hidden")).unwrap();
        fs::write(root.join(".hidden/.photo"), b"\x89PNG\r\n\x1a\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&root, root.join("nested/loop")).unwrap();
        let path = root.to_str().unwrap();
        let cancel = AtomicBool::new(false);
        let images = scan(path, &[1], &cancel, |_| true).unwrap();
        assert_eq!((images.scanned, images.matched, images.bytes), (3, 2, 16));
        assert!(images.files.iter().any(|file| file.path.ends_with("renamed.txt")));
        assert!(images.files.iter().any(|file| Path::new(&file.path).ends_with(Path::new(".hidden").join(".photo"))));
        let unknown = scan(path, &[0], &cancel, |_| true).unwrap();
        assert_eq!(unknown.matched, 1);
        assert!(unknown.files[0].path.ends_with("fake.png"));
        cancel.store(true, Ordering::SeqCst);
        assert!(scan(path, &[1], &cancel, |_| true).unwrap().cancelled);
        assert!(scan(path, &[], &cancel, |_| true).is_err());
        assert!(scan(path, &[99], &cancel, |_| true).is_err());
        assert!(scan(root.join("fake.png").to_str().unwrap(), &[1], &cancel, |_| true).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
