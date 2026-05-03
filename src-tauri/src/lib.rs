pub mod audio;
pub mod config;
pub mod error;
pub mod hotkey;
pub mod permissions;
pub mod stt;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

/// Returns true if this is the first launch (settings file has never been written).
/// Extracted for testability — callers should use this rather than inlining the path check.
fn is_first_launch(data_dir: &std::path::Path) -> bool {
    !data_dir.join("settings.json").exists()
}

/// Tauri command: close the onboarding window and optionally notify the user.
///
/// - `all_granted = true`:  both permissions in place → close window + send macOS notification
/// - `all_granted = false`: permissions missing → close window only (voice mode stays disabled)
#[tauri::command]
async fn finish_onboarding(app: tauri::AppHandle, all_granted: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("onboarding") {
        win.close().map_err(|e| e.to_string())?;
    }
    if all_granted {
        #[cfg(target_os = "macos")]
        {
            let result = std::process::Command::new("osascript")
                .arg("-e")
                .arg(r#"display notification "Hold your PTT key to start dictating." with title "ICantSpell is ready""#)
                .spawn();
            if let Err(e) = result {
                tracing::warn!("Failed to send completion notification: {}", e);
            }
        }
    }
    tracing::info!(all_granted = all_granted, "Onboarding completed");
    Ok(())
}

pub fn run() {
    tracing_subscriber::fmt::init();

    tracing::info!("ICantSpell starting up");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            permissions::check_accessibility_permission,
            permissions::request_accessibility_permission,
            permissions::check_microphone_permission,
            permissions::request_microphone_permission,
            permissions::check_all_permissions,
            config::save_ptt_hotkey,
            finish_onboarding,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;

            let icon = app.default_window_icon().unwrap().clone();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            // First-launch detection: check BEFORE ensure_defaults writes settings.json
            let data_dir = app.path().app_data_dir()?;
            let first_launch = is_first_launch(&data_dir);

            config::ensure_defaults(app.handle())?;
            permissions::start_permission_monitor(app.handle().clone());

            let settings = config::load(app.handle());

            // Start audio capture pipeline — always ready, even if hotkey not configured yet
            let audio_pipeline = match audio::start_audio_pipeline() {
                Ok(pipeline) => pipeline,
                Err(e) => {
                    tracing::error!("Failed to start audio pipeline: {} — voice dictation disabled", e);
                    // Cannot proceed with voice dictation, but don't crash the app
                    return Ok(());
                }
            };

            // Store audio buffer receiver in Tauri managed state for Story 3.3 (Whisper STT).
            // Wrap in Mutex since mpsc::Receiver is not Sync.
            app.manage(std::sync::Mutex::new(Some(audio_pipeline.audio_rx)));

            // Store command_tx so the pipeline stays alive even without a hotkey configured
            let capture_tx = audio_pipeline.command_tx;

            if settings.ptt_hotkey.is_empty() {
                tracing::info!("No PTT hotkey configured — skipping listener startup");
                // Keep capture_tx alive in managed state so pipeline thread doesn't exit
                app.manage(std::sync::Mutex::new(Some(capture_tx)));
            } else {
                let (ptt_tx, ptt_rx) = std::sync::mpsc::channel();
                hotkey::start_hotkey_listener(&settings.ptt_hotkey, ptt_tx)
                    .unwrap_or_else(|e| tracing::error!("Failed to start hotkey listener: {}", e));

                // Bridge: PttEvent → CaptureCommand
                let _bridge_handle = std::thread::Builder::new()
                    .name("ptt-audio-bridge".to_string())
                    .spawn(move || {
                        loop {
                            match ptt_rx.recv() {
                                Ok(hotkey::PttEvent::Pressed) => {
                                    if capture_tx.send(audio::CaptureCommand::Start).is_err() {
                                        tracing::info!("Audio pipeline closed — stopping bridge");
                                        return;
                                    }
                                }
                                Ok(hotkey::PttEvent::Released) => {
                                    if capture_tx.send(audio::CaptureCommand::Stop).is_err() {
                                        tracing::info!("Audio pipeline closed — stopping bridge");
                                        return;
                                    }
                                }
                                Err(_) => {
                                    tracing::info!("PTT channel closed — stopping bridge");
                                    return;
                                }
                            }
                        }
                    });
                if let Err(e) = _bridge_handle {
                    tracing::error!("Failed to spawn PTT-audio bridge thread: {}", e);
                }
            }

            if first_launch {
                tracing::info!("First launch detected — showing onboarding wizard");
                if let Some(win) = app.get_webview_window("onboarding") {
                    win.show()?;
                } else {
                    tracing::warn!("Onboarding window not found in Tauri config");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_first_launch_when_settings_absent() {
        let dir = std::env::temp_dir().join(format!(
            "icantspell_test_absent_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        assert!(
            is_first_launch(&dir),
            "should be first launch when settings.json is absent"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_is_not_first_launch_when_settings_present() {
        let dir = std::env::temp_dir().join(format!(
            "icantspell_test_present_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings.json"), b"{}").unwrap();
        assert!(
            !is_first_launch(&dir),
            "should NOT be first launch when settings.json exists"
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
