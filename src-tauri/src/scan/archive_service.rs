use super::{scan_model::ScanFile, media_signature};
use anyhow::{bail, Context, Result};
use std::{fs::File, io::Read, path::Path};

const MAX_RAR_ENTRY: u64 = 64 * 1024 * 1024;
const MAX_RAR_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

pub fn scan_archive(path: &Path, format: &str, mut visit: impl FnMut(Result<ScanFile>) -> bool) -> Result<()> {
    match format {
        "ZIP" => scan_zip(path, &mut visit),
        "RAR" => scan_rar(path, &mut visit),
        _ => Ok(()),
    }.with_context(|| format!("Cannot read archive {}", path.display()))
}

fn scan_zip(path: &Path, visit: &mut impl FnMut(Result<ScanFile>) -> bool) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(path)?)?;
    for index in 0..archive.len().min(MAX_ENTRIES) {
        let result: Result<Option<ScanFile>> = (|| {
            let mut entry = archive.by_index(index)?;
            if entry.is_dir() || entry.is_symlink() { return Ok(None); }
            let name = entry.name().to_owned();
            let size = entry.size();
            let mut head = Vec::with_capacity(media_signature::HEADER_BYTES);
            entry.by_ref().take(media_signature::HEADER_BYTES as u64).read_to_end(&mut head)?;
            Ok(Some(archive_file(path, &name, size, &head)?))
        })();
        match result {
            Ok(None) => {},
            Ok(Some(file)) => if !visit(Ok(file)) { return Ok(()); },
            Err(error) => if !visit(Err(error).with_context(|| format!("{} entry #{index}", path.display()))) { return Ok(()); },
        }
    }
    if archive.len() > MAX_ENTRIES { bail!("Archive entry limit reached ({MAX_ENTRIES})"); }
    Ok(())
}

fn scan_rar(path: &Path, visit: &mut impl FnMut(Result<ScanFile>) -> bool) -> Result<()> {
    let mut archive = unrar::Archive::new(path).open_for_processing()?;
    let mut bytes = 0u64;
    for _ in 0..MAX_ENTRIES {
        let Some(entry) = archive.read_header()? else { return Ok(()); };
        if !entry.entry().is_file() { archive = entry.skip()?; continue; }
        let name = entry.entry().filename.to_string_lossy().into_owned();
        let size = entry.entry().unpacked_size;
        if entry.entry().is_encrypted() || size > MAX_RAR_ENTRY || size > MAX_RAR_BYTES.saturating_sub(bytes) {
            if !visit(Err(anyhow::anyhow!("{}!/{name}: encrypted or exceeds RAR memory limit", path.display()))) { return Ok(()); }
            archive = entry.skip()?;
            continue;
        }
        let (data, next) = entry.read().with_context(|| format!("{}!/{name}", path.display()))?;
        archive = next;
        bytes = bytes.saturating_add(data.len() as u64);
        if !visit(archive_file(path, &name, size, &data[..data.len().min(media_signature::HEADER_BYTES)])) { return Ok(()); }
    }
    bail!("Archive entry limit reached ({MAX_ENTRIES})")
}

fn archive_file(path: &Path, name: &str, size: u64, head: &[u8]) -> Result<ScanFile> {
    let name = name.replace('\\', "/");
    if name.starts_with('/') || name.split('/').any(|part| part == ".." || part.contains(':')) {
        bail!("Invalid archive entry path: {name}");
    }
    let name = name.split('/').filter(|part| !part.is_empty() && *part != ".").collect::<Vec<_>>().join("/");
    if name.is_empty() { bail!("Empty archive entry path"); }
    let archive_path = path.to_string_lossy().into_owned();
    Ok(ScanFile {
        path: format!("{archive_path}!/{name}"),
        format: media_signature::detect(head),
        size, archive_path: Some(archive_path), entry_path: Some(name),
    })
}
