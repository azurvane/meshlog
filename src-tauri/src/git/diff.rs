use std::process::Command;

use crate::config::RenameCandidate;
use crate::config::RETRY_DELAY;
use crate::config::MAX_RETRY_ATTEMPTS;

// get the pairs of missing asset and closet match
#[tauri::command]
pub fn detect_renamed_files(root_path: &str, threshold: u8) -> Result<Vec<RenameCandidate>, String> {
    let add_output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !add_output.status.success() {
        let error_text = String::from_utf8(add_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git add failed: {}", error_text));
    }
    
    let threshold_str = format!("--find-renames={}%", threshold);

    let diff_output = Command::new("git")
        .args(["diff", "--cached", &threshold_str, "--name-status", "HEAD"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string());

    loop_git_restore(root_path)?;

    let diff_output = diff_output?;
    if !diff_output.status.success() {
        let error_text = String::from_utf8(diff_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git diff commit failed: {}", error_text))
    }

    let diff_text = String::from_utf8(diff_output.stdout).map_err(|e| e.to_string())?;
    let lines: Vec<String> = diff_text
        .lines()                       
        .filter(|line| !line.is_empty()) 
        .map(|s| s.to_string())
        .collect();

    let mut output_vec: Vec<RenameCandidate> = Vec::new();

    for line in lines {
        let split: Vec<&str> = line.split('\t').collect();
        if split.len() == 3 && split[0].starts_with('R') {
            let status = split[0];
            let score_str = &status[1..];
            let score = score_str.parse::<u8>().map_err(|e| format!("Failed to parse score: {e}"))?;
            let old_path = split[1].to_string();
            let new_path = split[2].to_string();
            crate::database::update_link(root_path, &old_path, &new_path)?;
            output_vec.push(RenameCandidate {
                old_path,
                new_path,
                score,
            });
        }
    }
    
    Ok(output_vec)
}

fn loop_git_restore(root_path: &str) -> Result<String, String> {
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        match git_restore(root_path) {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt == MAX_RETRY_ATTEMPTS {
                    return Err(format!("cannot undo git add due to: {}", err));
                }
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY));
            }
        }
    }
    unreachable!()
}

fn git_restore(root_path: &str) -> Result<String, String> {
    let restore_output = Command::new("git")
        .args(["restore", "--staged", "."])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;

    if restore_output.status.success() {
        let text = String::from_utf8(restore_output.stdout).map_err(|e| e.to_string())?;
        Ok(text)
    } else {
        let error_text = String::from_utf8(restore_output.stderr).map_err(|e| e.to_string())?;
        Err(format!("Git restore failed: {}", error_text))
    }
}