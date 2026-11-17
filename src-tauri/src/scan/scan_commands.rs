use super::{scan_model::*, scan_service};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{ipc::Channel, State};

#[derive(Default)]
pub struct ScanState(pub Arc<AtomicBool>, pub Arc<AtomicBool>);

#[tauri::command]
pub fn file_types() -> Vec<FileTypeOption> {
    scan_service::file_types()
}

#[tauri::command]
pub async fn scan_folder(
    path: String,
    types: Vec<i32>,
    progress: Channel<ScanProgress>,
    state: State<'_, ScanState>,
) -> Result<ScanProgress, String> {
    if state
        .0
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A scan is already running".into());
    }
    state.1.store(false, Ordering::SeqCst);
    let active = state.0.clone();
    let cancel = state.1.clone();
    let result = tauri::async_runtime::spawn_blocking(move || scan_service::scan(&path, &types, &cancel, |value| progress.send(value).is_ok())).await;
    active.store(false, Ordering::SeqCst);
    result
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, ScanState>) {
    state.1.store(true, Ordering::SeqCst);
}
