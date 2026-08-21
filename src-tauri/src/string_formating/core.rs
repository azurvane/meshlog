
// wrapper function for stamp adds the assetid
#[tauri::command]
pub fn stamp_version(asset_id: &str, version: &str) -> Result<String, String>{
    let (_, _, _) = crate::string_formating::parse_version_tuple(version)?;
    
    Ok(format!("{}-v{}", asset_id, version))
}

// wrapper function for terminal checks if valid assetid or 
/*
* need something if commiting for the first time either tell the user what is the 
* format used or accept the user format as final verdict
 */