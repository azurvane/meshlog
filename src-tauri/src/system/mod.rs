// return OS username and hostname to be displayed on the terminal 
#[tauri::command]
pub fn get_user_info() -> Result<(String, String), String> {
    let username = whoami::username().map_err(|e| e.to_string())?;
    let hostname = whoami::hostname().map_err(|e| e.to_string())?;
    Ok((username, hostname))
}