use tauri_plugin_macos_permissions::{check_system_audio_recording_permission, request_system_audio_recording_permission};

#[tokio::main]
async fn main() {
    println!("Testing system audio recording permission...");
    
    // Test check permission
    let has_permission = check_system_audio_recording_permission().await;
    println!("Has system audio recording permission: {}", has_permission);
    
    // Test request permission
    match request_system_audio_recording_permission().await {
        Ok(_) => println!("Successfully requested permission"),
        Err(e) => println!("Error requesting permission: {}", e),
    }
    
    // Check again
    let has_permission_after = check_system_audio_recording_permission().await;
    println!("Has system audio recording permission after request: {}", has_permission_after);
}