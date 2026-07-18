use std::path::Path;
use std::fs;

use crate::config::LOG_PATH;


// populate log md for all of the asset id
#[tauri::command]
pub fn populate_log_md(root_path: &str) -> Result<(), String> {
    let commit_files_paths = crate::git::get_commited_files(root_path)?;
    
    for relative_file_path in commit_files_paths {
        let (asset_id, _) = crate::git::get_assetid_version(&relative_file_path, root_path)?;
        populate_log_md_assetid(root_path, &asset_id)?;
    }
    
    Ok(())
}


// populate log md with all or missing logs for a asset id
#[tauri::command]
pub fn populate_log_md_assetid(root_path: &str, asset_id: &str) -> Result<(), String> {
    let log_file_path = Path::new(root_path)
        .join(LOG_PATH)
        .join(format!("{}.md", asset_id));
    let tags = crate::get_tag_assetid(asset_id, root_path)?;
    
    if !Path::new(&log_file_path).exists() {
        crate::file_system::create_log_md(&log_file_path)?;
        for tag in &tags {
            let commit_metadata = crate::git::get_commit_metadata(root_path, &tag)?;
            let version = super::helper::get_version(&tag)?;
            let format_metadata = super::helper::format_commit_metadata(commit_metadata, &version);
            crate::file_system::append_log_md(&log_file_path, &format_metadata)?;
        }
    }
    
    // whole thing is wrong needs to check missing version and not tags or asset id since we are only 
    // doing one asset id at a time so no asset id and tag contains two things so cannto use it directly
    let missing_version = super::helper::get_missing_version(&log_file_path, tags)?;
    
    for version in missing_version{
        let file_content = fs::read_to_string(&log_file_path).map_err(|e| e.to_string())?;
        let position = super::helper::find_insert_position(&file_content, &version)?;
        let tag = format!("{}-{}", asset_id, version);
        let entry = crate::git::get_commit_metadata(root_path, &tag)?;  
        let format_metadata = super::helper::format_commit_metadata(entry, &version);
        super::helper::insert_log_entry(&log_file_path, &format_metadata, position)?;
    }
    
    Ok(())
}
