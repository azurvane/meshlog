use std::path::Path;
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
                        match crate::string_formating::get_relative_directory_path(event.path, &root_for_closure) {
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
    
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    *guard = Some(debouncer);
    
    Ok(())
}


// stoping the watcher 
#[tauri::command]
pub fn stop_watching(state: State<WatcherState>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())

}
