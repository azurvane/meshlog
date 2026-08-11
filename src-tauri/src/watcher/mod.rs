use std::path::{Path, PathBuf};
use std::time::Duration;
use notify_debouncer_mini::{
    new_debouncer, 
    Debouncer, 
    DebounceEventResult, 
    notify::{
        RecursiveMode, 
        RecommendedWatcher
    }
};
use tauri::{Emitter, AppHandle, State};
use std::sync::Mutex;

pub type WatcherState = Mutex<Option<Debouncer<RecommendedWatcher>>>;

// watcher watches the changes in the root directory and sub-directories
#[tauri::command]
pub fn start_watching(
    app_handle: AppHandle, 
    state: State<WatcherState>,
    root_path: String
    ) -> Result<(), String> {
    let handle_for_closure = app_handle.clone();
    let root_for_closure = root_path.clone();
    
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |result: DebounceEventResult| {
            match result {
                Ok(events) =>{
                    for event in events {
                        match get_relative_directory_path(event.path, &root_for_closure) {
                            Ok(subdir_string) => { let _ = handle_for_closure.emit("fs-changed", subdir_string); }
                            Err(e) => { eprintln!("failed to compute subdir: {}", e); }
                        }
                    }
                }
                Err(error) => {
                    eprintln!("watch error: {:?}", error);
                }
            }
        }
    ).map_err(|e| e.to_string())?;
    
    debouncer
        .watcher()
        .watch(Path::new(&root_path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;
    
    let mut guard = state.lock().unwrap();
    *guard = Some(debouncer);
    
    Ok(())
}


// stoping the watcher 
#[tauri::command]
pub fn stop_watching(state: State<WatcherState>) -> Result<(), String> {
    let mut guard = state.lock().unwrap();
    *guard = None;
    Ok(())

}


// helper function 
// find the relative path to the sub-directory 
// move to string formating later
pub fn get_relative_directory_path(full_path: PathBuf, root_path: &str) -> Result<String, String> {
    let root = Path::new(root_path);

    let relative = full_path
        .strip_prefix(root)
        .map_err(|e| format!("Path is not inside root: {e}"))?;
    let subdir = relative
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    let subdir_string = subdir.to_string_lossy().to_string();

    Ok(subdir_string)
}