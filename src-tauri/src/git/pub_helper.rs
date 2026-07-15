// get the asset id and version 
pub fn get_assetid_version(relative_file_path: &str, root_path: &str) -> Result<(String, String), String> {
    let tag = super::tag::get_latest_tag_relative_path(relative_file_path, root_path)?;
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = parts[0].to_string();
    let asset_id = parts[1].to_string();
    
    Ok((asset_id, version))
}
