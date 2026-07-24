use std::path::PathBuf;
use std::fs;

// get the version of which logs are missing in the log md
pub fn get_missing_version(log_file_path: &PathBuf, tags: Vec<String>) -> Result<Vec<String>, String> {
    let file_content = fs::read_to_string(log_file_path).map_err(|e| e.to_string())?;
    let mut missing_version: Vec<String> = Vec::new();
    
    for tag in tags {
        let version = crate::string_formating::get_version(&tag)?;
        let header = format!("## {}\n", version);
        if !file_content.contains(&header){
            missing_version.push(version);
        }
    }
    
    Ok(missing_version)
}

// find the position where the missing log is suppose to go for the cronological order
pub fn find_insert_position(existing_content: &str, version: &str) -> Result<usize, String> {
    let new_version = parse_version_tuple(version)?;

    for (index, _) in existing_content.match_indices("## ") {
        let line_end = existing_content[index..]
            .find('\n')
            .map(|n| index + n)
            .unwrap_or(existing_content.len());

        let header_tag = existing_content[index + 3..line_end].trim();
        let existing_version = parse_version_tuple(header_tag)?;

        if new_version < existing_version {
            return Ok(index);
        }
    }

    Ok(existing_content.len()) 
}

// strip the v if present and convert in tupel of three unsign int 
pub fn parse_version_tuple(version: &str) -> Result<(u32, u32, u32), String> {
    let version = version.strip_prefix('v').unwrap_or(version);

    let nums: Vec<&str> = version.splitn(3, '.').collect();
    if nums.len() != 3 {
        return Err(format!("malformed version: {}", version));
    }

    let major: u32 = nums[0].parse().map_err(|_| format!("bad major: {}", nums[0]))?;
    let minor: u32 = nums[1].parse().map_err(|_| format!("bad minor: {}", nums[1]))?;
    let patch: u32 = nums[2].parse().map_err(|_| format!("bad patch: {}", nums[2]))?;
    Ok((major, minor, patch))
}

