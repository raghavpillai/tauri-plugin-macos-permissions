use tauri::{command, AppHandle, Runtime};

#[cfg(target_os = "macos")]
use {
    macos_accessibility_client::accessibility::{
        application_is_trusted, application_is_trusted_with_prompt,
    },
    objc2::{class, msg_send, runtime::Bool},
    objc2_foundation::NSString,
    std::{fs::read_dir, process::Command},
    tauri::Manager,
};

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDCheckAccess(request: u32) -> u32;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
    fn AudioObjectGetPropertyData(
        inObjectID: u32,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const std::ffi::c_void,
        ioDataSize: *mut u32,
        outData: *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct AudioObjectPropertyAddress {
    mSelector: u32,
    mScope: u32,
    mElement: u32,
}

#[cfg(target_os = "macos")]
const kAudioHardwarePropertyTranslatePIDToProcessObject: u32 = 0x70696432; // 'pid2'
#[cfg(target_os = "macos")]
const kAudioObjectSystemObject: u32 = 1;
#[cfg(target_os = "macos")]
const kAudioObjectPropertyScopeGlobal: u32 = 0x676c6f62; // 'glob'
#[cfg(target_os = "macos")]
const kAudioObjectPropertyElementMain: u32 = 0;

#[cfg(target_os = "macos")]
fn test_system_audio_permission() -> bool {
    unsafe {
        // Try to get the system process object (PID 0 represents system audio)
        let system_pid: u32 = 0;
        let mut process_object: u32 = 0;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyTranslatePIDToProcessObject,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        
        let result = AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            std::mem::size_of::<u32>() as u32,
            &system_pid as *const u32 as *const std::ffi::c_void,
            &mut data_size,
            &mut process_object as *mut u32 as *mut std::ffi::c_void,
        );
        
        // If we can successfully get the system process object, we likely have permission
        // This is a minimal test that doesn't actually create a tap, just checks if we can access system audio objects
        result == 0 && process_object != 0
    }
}



/// Check accessibility permission.
///
/// # Returns
/// - `bool`: `true` if accessibility permission are granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_accessibility_permission;
///
/// let authorized = check_accessibility_permission().await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    return application_is_trusted();

    #[cfg(not(target_os = "macos"))]
    return true;
}

/// Request accessibility permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_accessibility_permission;
///
/// request_accessibility_permission().await;
/// ```
#[command]
pub async fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    application_is_trusted_with_prompt();
}

/// Check full disk access permission.
///
/// # Returns
/// - `bool`: `true` if full disk access permission are granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_full_disk_access_permission;
///
/// let authorized = check_full_disk_access_permission(app_handle).await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_full_disk_access_permission<R: Runtime>(app_handle: AppHandle<R>) -> bool {
    #[cfg(target_os = "macos")]
    {
        // Reference: https://github.com/inket/FullDiskAccess/blob/846e04ea2b84fce843f47d7e7f3421189221829c/Sources/FullDiskAccess/FullDiskAccess.swift#L46
        let check_dirs = vec!["Library/Containers/com.apple.stocks", "Library/Safari"];

        if let Ok(home_dir) = app_handle.path().home_dir() {
            for check_dir in check_dirs.iter() {
                if read_dir(&home_dir.join(check_dir)).is_ok() {
                    return true;
                }
            }
        }

        false
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app_handle;

        true
    }
}

/// Request full disk access permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_full_disk_access_permission;
///
/// request_full_disk_access_permission().await;
/// ```
#[command]
pub async fn request_full_disk_access_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .output()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

/// Check screen recording permission.
///
/// # Returns
/// - `bool`: `true` if screen recording permission are granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_screen_recording_permission;
///
/// let authorized = check_screen_recording_permission().await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_screen_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        CGPreflightScreenCaptureAccess()
    }

    #[cfg(not(target_os = "macos"))]
    return true;
}

/// Request screen recording permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_screen_recording_permission;
///
/// request_screen_recording_permission().await;
/// ```
#[command]
pub async fn request_screen_recording_permission() {
    #[cfg(target_os = "macos")]
    unsafe {
        CGRequestScreenCaptureAccess();
    }
}

/// Check microphone permission.
///
/// # Returns
/// - `bool`: `true` if microphone permission is granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_microphone_permission;
///
/// let authorized = check_microphone_permission().await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let av_media_type = NSString::from_str("soun");
        let status: i32 = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: &*av_media_type
        ];

        status == 3
    }

    #[cfg(not(target_os = "macos"))]
    return true;
}

/// Request microphone permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_microphone_permission;
///
/// request_microphone_permission().await;
/// ```
#[command]
pub async fn request_microphone_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    unsafe {
        let av_media_type = NSString::from_str("soun");
        type CompletionBlock = Option<extern "C" fn(Bool)>;
        let completion_block: CompletionBlock = None;
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: &*av_media_type,
            completionHandler: completion_block
        ];
    }

    Ok(())
}

/// Check camera permission.
///
/// # Returns
/// - `bool`: `true` if camera permission is granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_camera_permission;
///
/// let authorized = check_camera_permission().await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_camera_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let av_media_type = NSString::from_str("vide");
        let status: i32 = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: &*av_media_type
        ];

        status == 3
    }

    #[cfg(not(target_os = "macos"))]
    return true;
}

/// Request camera permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_camera_permission;
///
/// request_camera_permission().await;
/// ```
#[command]
pub async fn request_camera_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    unsafe {
        let av_media_type = NSString::from_str("vide");
        type CompletionBlock = Option<extern "C" fn(Bool)>;
        let completion_block: CompletionBlock = None;
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: &*av_media_type,
            completionHandler: completion_block
        ];
    }

    Ok(())
}

/// Check input monitoring permission.
///
/// # Returns
/// - `bool`: `true` if input monitoring permission is granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_input_monitoring_permission;
///
/// let authorized = check_input_monitoring_permission().await;
/// println!("Authorized: {}", authorized); // false
/// ```
#[command]
pub async fn check_input_monitoring_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let status = IOHIDCheckAccess(1);

        status == 0
    }

    #[cfg(not(target_os = "macos"))]
    return true;
}

/// Request input monitoring permission.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_input_monitoring_permission;
///
/// request_input_monitoring_permission().await;
/// ```
#[command]
pub async fn request_input_monitoring_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .output()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

/// Check system audio recording permission.
///
/// This function attempts to access system audio objects to determine if permission has been granted.
/// Unlike other permissions, system audio recording permission is typically granted the first time
/// the app attempts to capture system audio.
///
/// # Returns
/// - `bool`: `true` if system audio recording permission appears to be granted, `false` otherwise.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::check_system_audio_recording_permission;
///
/// let authorized = check_system_audio_recording_permission().await;
/// println!("Authorized: {}", authorized);
/// ```
#[command]
pub async fn check_system_audio_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        test_system_audio_permission()
    }
    
    #[cfg(not(target_os = "macos"))]
    true
}

/// Request system audio recording permission.
///
/// Note: System audio recording permission follows a different model than other permissions.
/// There is no direct API to request permission. Instead, the system automatically prompts
/// the user the first time the app attempts to capture system audio.
///
/// This function will open System Settings to the Privacy & Security > Microphone section
/// where users can manually grant the permission, or inform them about the automatic process.
///
/// # Example
/// ```
/// use tauri_plugin_macos_permissions::request_system_audio_recording_permission;
///
/// request_system_audio_recording_permission().await;
/// ```
#[command]
pub async fn request_system_audio_recording_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // For system audio recording, we can't directly request the permission.
        // The best we can do is direct the user to System Settings where they
        // can see the permission controls, or inform them that permission will
        // be requested when they first use the audio capture feature.
        
        // Open System Settings to Privacy & Security > Screen Recording
        // Note: System audio recording permissions are managed in the same section as screen recording
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .output()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}
