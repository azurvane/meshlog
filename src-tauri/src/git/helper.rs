use std::process::Command;
use std::path::Path;

use crate::config::NO_COMMIT_METADATA;
use crate::config::CommitMetadata;

// get the commit meta data
pub fn get_commit_metadata(root_path: &str, tag: &str) -> Result<CommitMetadata, String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%H%x1f%h%x1f%an%x1f%aI%x1f%s%x1f%b", tag])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let commit_metadata = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let line = commit_metadata.lines().next().ok_or(NO_COMMIT_METADATA.to_string())?;
        let parts: Vec<&str> = line.split('\x1f').collect();
        
        if parts.len() < 6 {
            return Err("Malformed git log output".to_string());
        }
        
        Ok(CommitMetadata {
            commit_hash: parts[0].to_string(),
            abbreviated_hash: parts[1].to_string(),
            author_name: parts[2].to_string(),
            author_date: parts[3].to_string(),
            subject: parts[4].to_string(),
            body: parts[5].to_string(),
        })    
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get all the commit files (remove the hidden file)
pub fn get_commited_files(root_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let files = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let file_vec: Vec<String> = files
        .lines()
        .filter(|line| {
            !Path::new(line)
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        })
        .map(String::from)
        .collect();        
    Ok(file_vec)
    } else {    
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}