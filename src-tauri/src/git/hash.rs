use std::process::Command;

// get all the hash for a specific asset id
#[tauri::command]
pub fn get_all_hash_assetid(asset_id: &str, root_path: &str) -> Result<Vec<String>, String> {
    let tags = super::tag::get_tag_assetid(asset_id, root_path)?;
    let mut hashes = Vec::new();
    
    for tag in &tags {
        let hash = super::hash::get_hash_assetid(tag, root_path)?;
        hashes.push(hash);
    };
    
    Ok(hashes)
}

// get the latest hash for a specific asset id
#[tauri::command]
pub fn get_latest_hash_assetid(asset_id: &str, root_path: &str) -> Result<String, String> {
    let tag = super::tag::get_latest_tag_assetid(asset_id, root_path)?;
    let hash = super::hash::get_hash_assetid(&tag, root_path)?;
    Ok(hash)
}

// get the hash for the commit for a specific tag
pub fn get_hash_assetid(tag: &str, root_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", tag])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tag = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();
        Ok(tag)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}
