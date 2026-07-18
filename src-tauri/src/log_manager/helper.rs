use crate::config::CommitMetadata;
use std::path::PathBuf;
use std::fs;


pub fn get_missing_version(log_file_path: &PathBuf, tags: Vec<String>) -> Result<Vec<String>, String> {
    let file_content = fs::read_to_string(log_file_path).map_err(|e| e.to_string())?;
    let mut missing_version: Vec<String> = Vec::new();
    
    for tag in tags {
        let version = get_version(&tag)?;
        let header = format!("## {}\n", version);
        if !file_content.contains(&header){
            missing_version.push(version);
        }
    }
    
    Ok(missing_version)
}

pub fn get_version(tag: &str) -> Result<String, String> {    
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    
    Ok(version)
}

pub fn insert_log_entry(log_file_path: &PathBuf, entry: &str, position: usize) -> Result<(), String> {
    let mut content = fs::read_to_string(log_file_path).map_err(|e| e.to_string())?;
    content.insert_str(position, entry);
    fs::write(log_file_path, content).map_err(|e| e.to_string())
}

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

    Ok(existing_content.len()) // nothing bigger found — goes at the end
}

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

pub fn format_commit_metadata(metadata: CommitMetadata, version: &str) -> String {
    format!("## {}\n\
            - **Hash (full):** {}\n\
            - **Hash (short):** {}\n\
            - **Author:** {}\n\
            - **Created At:** {}\n\
            - **Summary:** {}\n\n\
            **Message:**\n\
            {}\n\n\
            ---\n\n\n",
            version,
            metadata.commit_hash,
            metadata.abbreviated_hash,
            metadata.author_name,
            metadata.author_date,
            metadata.subject,
            metadata.body,
    )
}
