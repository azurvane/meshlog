use std::process::Command;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::config::RETRY_DELAY;
use crate::config::MAX_RETRY_ATTEMPTS;
const RETRY_CAP:u64 = (MAX_RETRY_ATTEMPTS + 9)/10;

// add, commit and tag the files
#[tauri::command]
pub fn stage_commit_tag(root_path: &str, relative_file_path: &str, tag: &str, summary: &str, detail: &str)  -> Result<String, String> { 
    let sub_path = Path::new(root_path).join(relative_file_path);
    
    // git add
    let add_output = Command::new("git")
        .args(["add"])
        .arg(&sub_path)
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    // If staging fails, exit early with the error
    if !add_output.status.success() {
        let error_text = String::from_utf8(add_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git add failed: {}", error_text));
    }

    // git rm ghost path if exist
    if let Some(old_path) = crate::database::get_old_path(root_path, relative_file_path)? {
        let rm_output = Command::new("git")
            .args(["rm", "--cached", "--ignore-unmatch", &old_path])
            .current_dir(root_path)
            .output()
            .map_err(|e| e.to_string())?;
        
        if !rm_output.status.success() {
            let output_error = String::from_utf8_lossy(&rm_output.stderr);
            return Err(format!("Git rm failed: {}", output_error.trim()));
        }
    }
    
    // git commit
    let mut commit_cmd = Command::new("git");
    commit_cmd.arg("commit").arg("-m").arg(summary);
    if !detail.trim().is_empty() {
        commit_cmd.arg("-m").arg(detail);
    }
    let commit_output = commit_cmd
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    // Check if commit succeeds
    if !commit_output.status.success() {
        let commit_error = String::from_utf8_lossy(&commit_output.stderr);
        let rollback_output = Command::new("git")
            .args(["restore", "--staged", "."])
            .current_dir(root_path)
            .output()
            .map_err(|e| e.to_string())?;
        
        if !rollback_output.status.success() {
            let rollback_error = String::from_utf8_lossy(&rollback_output.stderr);
            return Err(format!(
                "Git commit failed: {}\nRollback failed: {}",
                commit_error.trim(),
                rollback_error.trim()
            ));
        }
        return Err(format!("Git commit failed: {}", commit_error.trim()));
    }
    
    let mut last_tag_error = String::new();
    for attempt in 0..RETRY_CAP {
        // git tag
        let tag_output = Command::new("git")
            .args(["tag", "--"])
            .arg(tag)
            .current_dir(root_path)
            .output()
            .map_err(|e| e.to_string())?;
        
        // Check if commit succeeds
        if tag_output.status.success() {
            let text = String::from_utf8(tag_output.stdout).map_err(|e| e.to_string())?;
            return Ok(text);
        }
        last_tag_error = String::from_utf8_lossy(&tag_output.stderr).trim().to_string();
        if attempt + 1 < RETRY_CAP {
            thread::sleep(Duration::from_millis(RETRY_DELAY));
        }
    }

    // Roll back the commit if tagging failed.
    let rollback_output = Command::new("git")
        .args(["reset", "--soft", "HEAD~1"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !rollback_output.status.success() {
        let rollback_error = String::from_utf8_lossy(&rollback_output.stderr).into_owned();
        return Err(format!(
            "Git tag failed: {}\nRollback failed: {}",
            last_tag_error, rollback_error.trim()
        ));
    }

    Err(format!(
        "Git tag failed: {}. Commit was rolled back.",
        last_tag_error
    ))
}

// get new or uncommit modified files
#[tauri::command]
pub fn get_uncommited_files(root_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "--modified"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let git_uncommited = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let git_uncommited_vec: Vec<String> = git_uncommited
            .lines()
            .filter(|line| !line.starts_with('.'))
            .map(String::from)
            .collect();
        
        Ok(git_uncommited_vec)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

#[tauri::command]
pub fn get_existing_uncommited_files(root_path: &str) -> Result<Vec<String>, String> {
    let files = get_uncommited_files(root_path)?;
    
    let existing_files = files
        .into_iter()
        .filter(|rel_path| Path::new(root_path).join(rel_path).exists())
        .collect();

    Ok(existing_files)
}

// get first commit created at
pub fn get_first_commit_creation_date(root_path: &str, asset_id: &str) -> Result<String, String> {
    let tag_name = super::tag::get_first_tag_assetid(root_path, asset_id)?;

    let output = Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%cd",
            "--date=format:%d/%m/%Y",
            &tag_name,
        ])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let date_str = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();

        if date_str.is_empty() {
            Err("Failed to retrieve date for commit".to_string())
        } else {
            Ok(date_str)
        }
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}