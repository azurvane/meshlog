use std::path::Path;
use std::fs;
use chrono::{DateTime, Local};

use crate::config::LOG_PATH;

// get the file name and created_at 
pub fn get_filename_createdat(relative_file_path: &str, root_path: &str) -> Result<(String, String), String> {
    let root = Path::new(root_path);
    let relative = Path::new(relative_file_path);
    let absolute_path_buf = root.join(relative);
    let absolute_path: &Path = absolute_path_buf.as_path();
    let metadata_raw = fs::metadata(absolute_path).map_err(|e| e.to_string())?;
    
    let file_name = absolute_path
        .file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_string());
    
    let created_system_time = metadata_raw.created().map_err(|e| e.to_string())?;
    let created_chrono: DateTime<Local> = created_system_time.into();
    let created_str = created_chrono.format("%d-%m-%Y").to_string();
    
    Ok((file_name, created_str))
}

// get the log path for a specific file
pub fn get_log_path(relative_file_path: &str, root_path: &str) -> Result<String, String> {
    let file_name = get_filename_createdat(relative_file_path, root_path)?.0;
    let log_path = Path::new(root_path)
        .join(LOG_PATH)
        .join(format!("{}.md", file_name))
        .to_string_lossy()
        .into_owned();
    
    Ok(log_path)
}