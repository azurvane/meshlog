
// wrapper function for stamp adds the assetid
#[tauri::command]
pub fn stamp_version(assetid: &str, version: &str) -> Result<String, String>{
    let (_, _, _) = crate::log_manager::parse_version_tuple(version)?;
    
    Ok(format!("{}-v{}", assetid, version))
}

// wrapper function for terminal checks if valid assetid or 
/*
* need something if commiting for the first time either tell the user what is the 
* format used or accept the user format as final verdict
 */