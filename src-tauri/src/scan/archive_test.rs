use super::{archive_service, scan_service};
use std::{fs, io::Write, sync::atomic::AtomicBool};

#[test]
fn zip_contents_are_scanned_by_header_even_when_container_is_filtered_out() {
    let root = std::env::temp_dir().join(format!("diskmora-zip-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let mut zip = zip::ZipWriter::new(fs::File::create(root.join("renamed.bin")).unwrap());
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(".hidden/photo.txt", options).unwrap();
    zip.write_all(b"\x89PNG\r\n\x1a\n").unwrap();
    zip.start_file("fake.png", options).unwrap();
    zip.write_all(b"plain text").unwrap();
    zip.start_file("audio/renamed.txt", options).unwrap();
    zip.write_all(b"RIFF\0\0\0\0WAVE").unwrap();
    zip.finish().unwrap();
    fs::write(root.join("broken.zip"), b"PK\x03\x04broken archive").unwrap();
    fs::write(root.join("sibling.png"), b"\x89PNG\r\n\x1a\n").unwrap();
    let result = scan_service::scan(root.to_str().unwrap(), &[1], &AtomicBool::new(false), |_| true).unwrap();
    assert_eq!(result.matched, 2);
    assert_eq!(result.skipped, 1);
    let entry = result.files.iter().find(|file| file.archive_path.is_some()).unwrap();
    assert_eq!(entry.format, "PNG");
    assert_eq!(entry.size, 8);
    assert_eq!(entry.entry_path.as_deref(), Some(".hidden/photo.txt"));
    assert!(entry.path.contains("renamed.bin!/.hidden/photo.txt"));
    let audio = scan_service::scan(root.to_str().unwrap(), &[23], &AtomicBool::new(false), |_| true).unwrap();
    assert_eq!(audio.matched, 1);
    assert_eq!(audio.files[0].format, "WAV");
    let mut visited = 0;
    archive_service::scan_archive(&root.join("renamed.bin"), "ZIP", |_| { visited += 1; false }).unwrap();
    assert_eq!(visited, 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rar_contents_are_scanned_without_extracting_to_disk() {
    let root = std::env::temp_dir().join(format!("diskmora-rar-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("renamed.bin"), include_bytes!("../../tests/fixtures/headers.rar")).unwrap();
    let result = scan_service::scan(root.to_str().unwrap(), &[1], &AtomicBool::new(false), |_| true).unwrap();
    assert_eq!(result.skipped, 0, "{:?}", result.errors);
    assert_eq!(result.matched, 1);
    assert_eq!(result.files[0].format, "PNG");
    assert_eq!(result.files[0].entry_path.as_deref(), Some(".hidden/photo.txt"));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}
