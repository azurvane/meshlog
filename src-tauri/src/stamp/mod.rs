

// links the renamed/relocate/both asset to the new name/path/both
#[tauri::command]
pub fn commit_stamp(root_path: &str, relative_file_path: &str, tag: &str, summary: &str, detail: &str) -> Result<(), String>{
    crate::git::stage_commit_tag(root_path, relative_file_path, tag, summary, detail)?;
    if let Some(old_path) = crate::database::get_old_path(root_path, relative_file_path)? {
        crate::git::delete_ghost_path(root_path, &old_path)?;
    }
    crate::database::update_db(root_path, relative_file_path)?;
    let (asset_id, _) = crate::string_formating::get_assetid_version_tag(tag)?;
    crate::log_manager::update_log_md(root_path, &asset_id)?;
    crate::database::delete_link(root_path, relative_file_path)?;
    Ok(())
}