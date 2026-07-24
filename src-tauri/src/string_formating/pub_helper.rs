

use crate::config::CommitMetadata;

// format the commit metadata to be inserted in the log file
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

// extract version from the tag
pub fn get_version(tag: &str) -> Result<String, String> {    
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    
    Ok(version)
}

// get the asset id and version 
pub fn get_assetid_version(relative_file_path: &str, root_path: &str) -> Result<(String, String), String> {
    let tag = crate::git::get_latest_tag_relative_path(relative_file_path, root_path)?;
    let parts: Vec<&str> = tag.rsplitn(2, "-v").collect();
    
    if parts.len() != 2 {
        return Err(format!("Tag format is invalid: {}", tag));
    }
    
    let version = format!("v{}", parts[0]);
    let asset_id = parts[1].to_string();
    
    Ok((asset_id, version))
}