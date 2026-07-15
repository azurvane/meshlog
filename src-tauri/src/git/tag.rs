use std::process::Command;

use crate::config::NO_TAG_ERROR;

// get all the tags
#[tauri::command]
pub fn get_tag(root_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["tag", "--list", "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag_vec: Vec<String> = tags.lines().map(String::from).collect();
        Ok(tag_vec)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get tag for a specific asset id
#[tauri::command]
pub fn get_tag_assetid(asset_id: &str, root_path: &str) -> Result<Vec<String>, String> {
    let pattern = format!("{}-v*", asset_id);
    
    let output = Command::new("git")
        .args(["tag", "--list", &pattern, "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag_vec: Vec<String> = tags.lines().map(String::from).collect();
        Ok(tag_vec)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get the latest tag for a specific asset id
#[tauri::command]
pub fn get_latest_tag_assetid(asset_id: &str, root_path: &str) -> Result<String, String> {
    let pattern = format!("{}-v*", asset_id);
    
    let output = Command::new("git")
        .args(["tag", "--list", &pattern, "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag = tags.lines().next().map(String::from).ok_or(NO_TAG_ERROR.to_string())?;
        Ok(tag)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// generate new tag
#[tauri::command]
pub fn generate_tag(asset_id: &str, root_path: &str) -> Result<String, String> {
    match  get_latest_tag_assetid(asset_id, root_path) {
        Ok(tag) =>  {
            let len = tag.len();
            
            if len < 4 {
                return Err("Tag is too short to parse suffix".to_string());
            }
            
            let suffix = &tag[len - 4..];
            
            if let Ok(mut number) = suffix.parse::<u32>() {
                number += 1;
                
                let prefix = &tag[..len - 4];
                let new_tag = format!("{}{:04}", prefix, number);
                Ok(new_tag)
            } else {
                Err(format!("invalid last 4 digit: {}", suffix).to_string())
            }
        }
        // Match specifically against your constant
        Err(ref err) if err == NO_TAG_ERROR => {
            Ok(format!("{}-v00.00.0001", asset_id))
        }
        // Catch-all for actual Git failures or other errors
        Err(git_err) => {
            Err(format!("Git command failed: {}", git_err))
        }
    }
}

// get latest tag for a perticular file by relative path
pub fn get_latest_tag_relative_path(relative_file_path: &str, root_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["log", "-n", "1", "--oneline", "--decorate", "--", relative_file_path])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let git_history = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let line = git_history.lines().next().ok_or(NO_TAG_ERROR.to_string())?;
        if let Some(tag_start) = line.find("tag: ") {
            let tag_part = &line[tag_start + 5..];
            let tag_end = tag_part
                .find(|c| c == ')' || c == ',')
                .unwrap_or(tag_part.len());
                
            let tag_name = tag_part[..tag_end].trim().to_string();
            Ok(tag_name)
        } else {
            Err(NO_TAG_ERROR.to_string())
        }
    } else {    
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}
