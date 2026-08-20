use std::thread;
use std::time::Duration;

use crate::config::RETRY_DELAY;
use crate::config::MAX_RETRY_ATTEMPTS;

#[tauri::command]
pub fn commit_stamp(root_path: &str, relative_file_path: &str, tag: &str, summary: &str, detail: &str) -> Result<(), String>{
    let (asset_id, _) = crate::string_formating::get_assetid_version_tag(tag)?; 
    let mut last_error = String::from("Operation failed: loop exhausted without completing all steps.");
    for attempt in 0..MAX_RETRY_ATTEMPTS{
        if attempt > 0 {
            thread::sleep(Duration::from_millis(RETRY_DELAY));
        }
        let tag_list = crate::git::get_tag_assetid(root_path, &asset_id)?;
        if tag_list.contains(&tag.to_string()) {
            match crate::git::get_latest_tag_relative_path(root_path, relative_file_path) {
                Ok(latest_file_tag) if latest_file_tag == tag => { }
                _ => {
                    return Err(format!(
                        "FATAL: Tag '{}' already exists, but target path '{}' is not committed under it.",
                        tag, relative_file_path
                    ));
                }
            }
        } else {
            if let Err(error) = crate::git::stage_commit_tag(root_path, relative_file_path, tag, summary, detail) {
                if error.starts_with("FATAL") {
                    return Err(error);
                }
                last_error = error;
                continue;
            }
        }
        if let Err(error) = crate::database::update_db(root_path, relative_file_path) {
            last_error = format!("Database update failed: {}", error);
            continue;
        }
        if let Err(error) = crate::log_manager::update_log_md(root_path, &asset_id) {
            last_error = format!("Log markdown update failed: {}", error);
            continue;
        }
        if let Err(error) = crate::database::delete_link(root_path, relative_file_path) {
            last_error = format!("Database delete link failed: {}", error);
            continue;
        }
        return Ok(());
    }
    Err(format!("Commit stamp failed after {} attempts. Last error: {}", MAX_RETRY_ATTEMPTS, last_error))
}