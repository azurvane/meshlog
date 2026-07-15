use std::process::Command;
use std::path::Path;

// add and commit the files
#[tauri::command]
pub fn stage_commit_tag(root_path: &str, relative_file_path: &str, summary: &str, tag: &str)  -> Result<String, String> { 
    let sub_path = Path::new(root_path).join(relative_file_path);
    
    // git add
    let add_output = Command::new("git")
    .arg("add")
    .arg(&sub_path)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // If staging fails, exit early with the error
    if !add_output.status.success() {
        let error_text = String::from_utf8(add_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git add failed: {}", error_text));
    }
    
    // git commit
    let commit_output = Command::new("git")
    .arg("commit")
    .arg("-m")
    .arg(summary)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // Check if commit succeeds
    if !commit_output.status.success() {
        let error_text = String::from_utf8(commit_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git commit failed: {}", error_text))
    }
    
    // git tag
    let tag_output = Command::new("git")
    .arg("tag")
    .arg(tag)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // Check if commit succeeds
    if tag_output.status.success() {
        let text = String::from_utf8(tag_output.stdout).map_err(|e| e.to_string())?;
        Ok(text)
    } else {    
        let error_text = String::from_utf8(tag_output.stderr).map_err(|e| e.to_string())?;
        Err(format!("Git commit failed: {}", error_text))
    }
}

// get all the commits
#[tauri::command]
pub fn get_commit(root_path: &str) -> Result<Vec<String>, String> { // USELESS FUNCTION FOR NOW AT LEAST
    let output = Command::new("git")
        .args(["log", "--oneline", "--graph", "--decorate"])
        .current_dir(root_path)
        .output() // Executes the command and captures stdout/stderr
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let git_history = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let git_history_vec: Vec<String> = git_history.lines().map(String::from).collect();
        // println!("{:?}", git_history_vec);
        Ok(git_history_vec)
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