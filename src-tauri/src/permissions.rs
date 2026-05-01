//! macOS permission checking and request flows.
//! This is the ONLY module that directly calls AXIsProcessTrusted() and AVCaptureDevice APIs.
//! Revocation monitoring is added in Story 2.6.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Returns true if the current process is trusted for Accessibility access (no side effects).
    fn AXIsProcessTrusted() -> bool;
}

/// Returns true if Accessibility permission has been granted.
/// On non-macOS platforms, always returns false.
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Safety: AXIsProcessTrusted takes no arguments and returns a simple bool.
        // It queries the macOS TCC database without memory allocation or side effects.
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// Opens System Settings > Privacy & Security > Accessibility so the user can grant access.
/// Fire-and-forget: spawns `open` and returns immediately. Logs a warning on failure.
pub fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let result = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
        if let Err(e) = result {
            tracing::warn!("Failed to open Accessibility settings: {}", e);
        }
    }
}

/// Tauri command: check if Accessibility permission is granted.
#[tauri::command]
pub async fn check_accessibility_permission() -> Result<bool, String> {
    Ok(check_accessibility())
}

/// Tauri command: open System Settings so the user can grant Accessibility permission.
#[tauri::command]
pub async fn request_accessibility_permission() -> Result<(), String> {
    open_accessibility_settings();
    Ok(())
}

// ── Microphone Permission ────────────────────────────────────────────────────

// AVFoundation exports AVMediaTypeAudio as a C-linkage NSString* constant.
#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    /// NSString constant "soun" — the media type for audio capture.
    static AVMediaTypeAudio: *mut std::ffi::c_void;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;

    /// objc_msgSend variant: class method returning NSInteger (isize).
    /// Used for: [AVCaptureDevice authorizationStatusForMediaType:]
    #[link_name = "objc_msgSend"]
    fn msg_send_authorization_status(
        receiver: *mut std::ffi::c_void,
        sel: *mut std::ffi::c_void,
        media_type: *mut std::ffi::c_void,
    ) -> isize;

    /// Global block class — used as the `isa` pointer for blocks with no captured variables.
    static _NSConcreteGlobalBlock: *const std::ffi::c_void;
}

#[cfg(target_os = "macos")]
#[allow(clashing_extern_declarations)]
extern "C" {
    /// objc_msgSend variant: class method taking object + block pointer, returning void.
    /// Used for: [AVCaptureDevice requestAccessForMediaType:completionHandler:]
    /// Separate extern block required because Rust's clashing_extern_declarations lint
    /// fires when two typed aliases for objc_msgSend appear in the same block.
    #[link_name = "objc_msgSend"]
    fn msg_send_request_access(
        receiver: *mut std::ffi::c_void,
        sel: *mut std::ffi::c_void,
        media_type: *mut std::ffi::c_void,
        block: *const BlockLayout,
    );
}

/// ObjC block layout for a completion handler block with no captured variables.
/// This is the ABI-defined struct layout for `^(BOOL granted) {}` blocks.
/// See: https://clang.llvm.org/docs/Block-ABI-Apple.html
#[cfg(target_os = "macos")]
#[repr(C)]
struct BlockLayout {
    isa: *const std::ffi::c_void,
    flags: i32,
    reserved: i32,
    invoke: unsafe extern "C" fn(*const BlockLayout, bool),
    descriptor: *const BlockDescriptor,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct BlockDescriptor {
    reserved: usize,
    size: usize,
}

/// Safety: BlockLayout contains raw pointers, but we only ever use it as a
/// static (BLOCK_IS_GLOBAL = 1 << 28). No mutable state or aliasing.
/// Send is safe because all contained pointers point to static data only.
#[cfg(target_os = "macos")]
unsafe impl Sync for BlockLayout {}
#[cfg(target_os = "macos")]
unsafe impl Send for BlockLayout {}

#[cfg(target_os = "macos")]
unsafe extern "C" fn microphone_block_invoke(_block: *const BlockLayout, _granted: bool) {
    // Intentionally empty — the frontend polls check_microphone_permission after calling this.
}

#[cfg(target_os = "macos")]
static MICROPHONE_BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: std::mem::size_of::<BlockLayout>(),
};

#[cfg(target_os = "macos")]
static MICROPHONE_REQUEST_BLOCK: std::sync::OnceLock<BlockLayout> = std::sync::OnceLock::new();

// AVAuthorizationStatus values (from AVFoundation headers):
// NotDetermined = 0, Restricted = 1, Denied = 2, Authorized = 3
const AV_AUTHORIZATION_STATUS_AUTHORIZED: isize = 3;
const AV_AUTHORIZATION_STATUS_NOT_DETERMINED: isize = 0;

/// Returns true if Microphone permission has been granted.
/// On non-macOS platforms, always returns false.
pub fn check_microphone() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            let cls = objc_getClass(c"AVCaptureDevice".as_ptr());
            let sel = sel_registerName(c"authorizationStatusForMediaType:".as_ptr());
            let status = msg_send_authorization_status(cls, sel, AVMediaTypeAudio);
            status == AV_AUTHORIZATION_STATUS_AUTHORIZED
        }
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// Requests microphone access from the OS.
///
/// - Status `NotDetermined`: calls `requestAccessForMediaType:completionHandler:`
///   which shows the macOS system permission dialog.
/// - Status `Denied` or `Restricted`: opens System Settings > Privacy & Security > Microphone
///   (the OS does not allow re-prompting once denied).
///
/// The Tauri command returns immediately; the frontend polls `check_microphone_permission`
/// to detect when the user has responded.
pub fn request_microphone_access() {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            let cls = objc_getClass(c"AVCaptureDevice".as_ptr());
            let status_sel = sel_registerName(c"authorizationStatusForMediaType:".as_ptr());
            let status = msg_send_authorization_status(cls, status_sel, AVMediaTypeAudio);

            if status == AV_AUTHORIZATION_STATUS_NOT_DETERMINED {
                // Use a static block so the pointer remains valid for the async callback.
                let block = MICROPHONE_REQUEST_BLOCK.get_or_init(|| BlockLayout {
                    isa: _NSConcreteGlobalBlock,
                    flags: 1 << 28, // BLOCK_IS_GLOBAL
                    reserved: 0,
                    invoke: microphone_block_invoke,
                    descriptor: &MICROPHONE_BLOCK_DESCRIPTOR,
                });
                let request_sel =
                    sel_registerName(c"requestAccessForMediaType:completionHandler:".as_ptr());
                msg_send_request_access(cls, request_sel, AVMediaTypeAudio, block as *const _);
            } else {
                // Denied or Restricted — OS won't re-show dialog; open Settings instead.
                let result = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
                    .spawn();
                if let Err(e) = result {
                    tracing::warn!("Failed to open Microphone settings: {}", e);
                }
            }
        }
    }
}

/// Tauri command: check if Microphone permission is granted.
#[tauri::command]
pub async fn check_microphone_permission() -> Result<bool, String> {
    Ok(check_microphone())
}

/// Tauri command: request Microphone permission or open System Settings if denied.
#[tauri::command]
pub async fn request_microphone_permission() -> Result<(), String> {
    request_microphone_access();
    Ok(())
}

// ── Permission Change Monitoring ──────────────────────────────────────────────

/// Payload for `permission_revoked` and `permission_restored` IPC events.
/// `permission_name` is either `"Accessibility"` or `"Microphone"`.
/// The `camelCase` rename ensures JS receives `permissionName`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionChangedPayload {
    pub permission_name: String,
}

/// Starts a background thread that polls Accessibility and Microphone permission
/// status every 5 seconds. On state transitions emits `permission_revoked` or
/// `permission_restored` Tauri events to all windows.
///
/// Uses `std::thread::spawn` rather than `tokio::spawn` — this is a long-running
/// background loop, not a short async task.
pub fn start_permission_monitor(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        use tauri::Emitter;

        let mut last_accessibility = check_accessibility();
        let mut last_microphone = check_microphone();

        // Emit initial state so the frontend shows warnings for permissions
        // that were already revoked before the monitor started.
        if !last_accessibility {
            if let Err(e) = app.emit(
                "permission_revoked",
                PermissionChangedPayload {
                    permission_name: "Accessibility".to_string(),
                },
            ) {
                tracing::warn!("Failed to emit initial accessibility revocation: {}", e);
            }
        }
        if !last_microphone {
            if let Err(e) = app.emit(
                "permission_revoked",
                PermissionChangedPayload {
                    permission_name: "Microphone".to_string(),
                },
            ) {
                tracing::warn!("Failed to emit initial microphone revocation: {}", e);
            }
        }

        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));

            let accessibility = check_accessibility();
            let microphone = check_microphone();

            if last_accessibility && !accessibility {
                if let Err(e) = app.emit(
                    "permission_revoked",
                    PermissionChangedPayload {
                        permission_name: "Accessibility".to_string(),
                    },
                ) {
                    tracing::warn!("Failed to emit permission_revoked: {}", e);
                }
                tracing::warn!("Accessibility permission revoked — notified frontend");
            } else if !last_accessibility && accessibility {
                if let Err(e) = app.emit(
                    "permission_restored",
                    PermissionChangedPayload {
                        permission_name: "Accessibility".to_string(),
                    },
                ) {
                    tracing::warn!("Failed to emit permission_restored: {}", e);
                }
                tracing::info!("Accessibility permission restored — notified frontend");
            }

            if last_microphone && !microphone {
                if let Err(e) = app.emit(
                    "permission_revoked",
                    PermissionChangedPayload {
                        permission_name: "Microphone".to_string(),
                    },
                ) {
                    tracing::warn!("Failed to emit permission_revoked: {}", e);
                }
                tracing::warn!("Microphone permission revoked — notified frontend");
            } else if !last_microphone && microphone {
                if let Err(e) = app.emit(
                    "permission_restored",
                    PermissionChangedPayload {
                        permission_name: "Microphone".to_string(),
                    },
                ) {
                    tracing::warn!("Failed to emit permission_restored: {}", e);
                }
                tracing::info!("Microphone permission restored — notified frontend");
            }

            last_accessibility = accessibility;
            last_microphone = microphone;
        }
    });
}

// ── Combined Permission Check ─────────────────────────────────────────────────

/// Snapshot of both required permission states, returned to the onboarding
/// validation step. Serialized by Tauri to `{ "accessibility": bool, "microphone": bool }`.
#[derive(Debug, serde::Serialize)]
pub struct PermissionsStatus {
    pub accessibility: bool,
    pub microphone: bool,
}

/// Tauri command: check both Accessibility and Microphone permissions in one call.
/// Used by the onboarding validation step to determine final setup state.
#[tauri::command]
pub async fn check_all_permissions() -> Result<PermissionsStatus, String> {
    Ok(PermissionsStatus {
        accessibility: check_accessibility(),
        microphone: check_microphone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_returns_bool() {
        // Verifies the FFI call (or non-macOS stub) does not panic.
        // Does not assert true/false — depends on system TCC grant state.
        let _ = check_accessibility();
    }

    #[test]
    fn test_check_microphone_returns_bool() {
        // Verifies the AVCaptureDevice FFI chain (or non-macOS stub) does not panic.
        // Does not assert true/false — depends on system TCC grant state.
        let _ = check_microphone();
    }

    #[test]
    fn test_permissions_status_fields() {
        // Verifies the struct constructs and serializes with the expected JSON field names.
        let status = PermissionsStatus { accessibility: true, microphone: false };
        let json = serde_json::to_value(&status).expect("serialize ok");
        assert_eq!(json["accessibility"], true);
        assert_eq!(json["microphone"], false);
    }

    #[test]
    fn test_permission_changed_payload_fields() {
        // Verifies the struct serializes with camelCase field name ("permissionName").
        let payload = PermissionChangedPayload {
            permission_name: "Accessibility".to_string(),
        };
        let json = serde_json::to_value(&payload).expect("serialize ok");
        assert_eq!(json["permissionName"], "Accessibility");
    }
}
