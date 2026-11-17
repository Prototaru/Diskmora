use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTypeOption {
    pub code: i32,
    pub label: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanFile {
    pub path: String,
    pub format: String,
    pub size: u64,
    pub archive_path: Option<String>,
    pub entry_path: Option<String>,
}

#[derive(Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub scanned: u64,
    pub matched: u64,
    pub bytes: u64,
    pub skipped: u64,
    pub permission_denied: u64,
    pub current_path: String,
    pub files: Vec<ScanFile>,
    pub errors: Vec<String>,
    pub unread_directories: Vec<String>,
    pub cancelled: bool,
}
