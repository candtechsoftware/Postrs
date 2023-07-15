use crate::rest;


#[tauri::command]
pub async fn send_request(full_url: &str, method: &str) -> std::result::Result<String, ()> {
  match rest::make_request(full_url, method).await {
    Ok(i) => Ok(String::from(i)),
    Err(_) => Ok(String::from("This is an error")),
  }
}