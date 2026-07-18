use std::path::Path;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::config::LOG_PATH;
use crate::config::CommitMetadata;


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
            let version = get_version(&tag)?;
            let format_metadata = format_commit_metadata(commit_metadata, &version);
            append_log_md(&log_file_path, &format_metadata)?;
        }
    }
    
    // whole thing is wrong needs to check missing version and not tags or asset id since we are only 
    // doing one asset id at a time so no asset id and tag contains two things so cannto use it directly
    let missing_version = get_missing_version(&log_file_path, tags)?;
    
    for version in missing_version{
        let file_content = fs::read_to_string(&log_file_path).map_err(|e| e.to_string())?;
        let position = find_insert_position(&file_content, &version)?;
        let tag = format!("{}-{}", asset_id, version);
        let entry = crate::git::get_commit_metadata(root_path, &tag)?;  
        let format_metadata = format_commit_metadata(entry, &version);
        insert_log_entry(&log_file_path, &format_metadata, position)?;
    }
    
    Ok(())
}

fn get_missing_version(log_file_path: &PathBuf, tags: Vec<String>) -> Result<Vec<String>, String> {
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


fn get_version(tag: &str) -> Result<String, String> {    
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    
    Ok(version)
}

fn insert_log_entry(log_file_path: &PathBuf, entry: &str, position: usize) -> Result<(), String> {
    let mut content = fs::read_to_string(log_file_path).map_err(|e| e.to_string())?;
    content.insert_str(position, entry);
    fs::write(log_file_path, content).map_err(|e| e.to_string())
}

fn find_insert_position(existing_content: &str, version: &str) -> Result<usize, String> {
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

fn parse_version_tuple(version: &str) -> Result<(u32, u32, u32), String> {
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

fn append_log_md(log_file_path: &PathBuf, format_metadata: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(log_file_path)
        .map_err(|e| e.to_string())?;
    
    file.write_all(format_metadata.as_bytes()).map_err(|e| e.to_string())
}

fn format_commit_metadata(metadata: CommitMetadata, version: &str) -> String {
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