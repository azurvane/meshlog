use std::path::PathBuf;
use std::fs;

// get the version of which logs are missing in the log md
pub fn get_missing_version(log_file_path: &PathBuf, tags: Vec<String>) -> Result<Vec<String>, String> {
    let file_content = fs::read_to_string(log_file_path).map_err(|e| e.to_string())?;
    let mut missing_version: Vec<String> = Vec::new();
    
    for tag in tags {
        let (_, version) = crate::string_formating::get_assetid_version_tag(&tag)?;
        let header = format!("## {}\n", version);
        if !file_content.contains(&header){
            missing_version.push(version);
        }
    }
    
    Ok(missing_version)
}

// find the position where the missing log is suppose to go for the cronological order
pub fn find_insert_position(existing_content: &str, version: &str) -> Result<usize, String> {
    let new_version = crate::string_formating::parse_version_tuple(version)?;

    for (index, _) in existing_content.match_indices("## ") {
        let line_end = existing_content[index..]
            .find('\n')
            .map(|n| index + n)
            .unwrap_or(existing_content.len());

        let header_tag = existing_content[index + 3..line_end].trim();
        let existing_version = crate::string_formating::parse_version_tuple(header_tag)?;

        if new_version < existing_version {
            return Ok(index);
        }
    }

    Ok(existing_content.len()) 
}


