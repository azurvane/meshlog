use std::path::{Path, PathBuf};

use crate::config::CommitMetadata;

// format the commit metadata to be inserted in the log file
pub fn format_commit_metadata(metadata: CommitMetadata, version: &str) -> String {
    format!("## {}\n\
            - **Hash (full):** {}\n\
            - **Hash (short):** {}\n\
            - **Author:** {}\n\
            - **Created At:** {}\n\
            - **Summary:** {}\n\n\
            - **Message:**\n\
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

// extract asset id and version from tag
pub fn get_assetid_version_tag(tag: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    let asset_id = parts[1].to_string();
    
    Ok((asset_id, version))
}

// get the asset id and version through the path
pub fn get_assetid_version_path(relative_file_path: &str, root_path: &str) -> Result<(String, String), String> {
    let tag = crate::git::get_latest_tag_relative_path(root_path, relative_file_path)?;
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    let asset_id = parts[1].to_string();
    
    Ok((asset_id, version))
}

// find the relative path to the sub-directory 
pub fn get_relative_directory_path(full_path: PathBuf, root_path: &str) -> Result<String, String> {
    let root = Path::new(root_path);

    let relative = full_path
        .strip_prefix(root)
        .map_err(|e| format!("Path is not inside root: {e}"))?;
    let subdir = relative
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    let subdir_string = subdir.to_string_lossy().to_string();

    Ok(subdir_string)
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