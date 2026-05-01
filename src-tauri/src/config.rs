// src-tauri/src/config.rs
use tauri_plugin_store::StoreExt;

pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85;
pub const DEFAULT_PTT_HOTKEY: &str = "AltRight";
const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub ptt_hotkey: String,
    pub selected_model: String,
    pub confidence_threshold: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ptt_hotkey: String::new(),
            selected_model: "base".to_string(),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
        }
    }
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    let Ok(store) = app.store(STORE_FILE) else {
        tracing::warn!("Failed to open settings store, using defaults");
        return Settings::default();
    };
    match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).unwrap_or_else(|e| {
            tracing::warn!(err = %e, "Settings deserialization failed, using defaults");
            Settings::default()
        }),
        None => Settings::default(),
    }
}

pub fn save(
    app: &tauri::AppHandle,
    settings: &Settings,
) -> Result<(), crate::error::AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    let val = serde_json::to_value(settings)
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    store.set(SETTINGS_KEY, val);
    store
        .save()
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    tracing::debug!("Settings saved");
    Ok(())
}

pub fn ensure_defaults(app: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let settings = load(app);
    save(app, &settings)?;
    tracing::info!(
        model = %settings.selected_model,
        confidence_threshold = settings.confidence_threshold,
        "Config initialized"
    );
    Ok(())
}

/// Tauri command: update the PTT hotkey binding in persistent settings.
///
/// The hotkey string format is a `KeyboardEvent.code`-derived value from the
/// onboarding frontend (e.g., `"AltRight"`, `"Alt+Space"`). Story 3.1 (`hotkey.rs`)
/// is responsible for parsing this string into a platform-specific event tap.
#[tauri::command]
pub async fn save_ptt_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    // Validate the hotkey string is parseable before persisting, so unsupported
    // key codes are rejected immediately rather than failing on next launch.
    if !hotkey.is_empty() {
        crate::hotkey::parse_hotkey(&hotkey).map_err(|e| e.to_string())?;
    }
    let mut settings = load(&app);
    settings.ptt_hotkey = hotkey;
    save(&app, &settings).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_confidence_threshold() {
        let settings = Settings::default();
        assert_eq!(settings.confidence_threshold, DEFAULT_CONFIDENCE_THRESHOLD);
        assert_eq!(DEFAULT_CONFIDENCE_THRESHOLD, 0.85_f32);
    }

    #[test]
    fn test_default_selected_model() {
        let settings = Settings::default();
        assert_eq!(settings.selected_model, "base");
    }

    #[test]
    fn test_default_ptt_hotkey_is_empty() {
        let settings = Settings::default();
        assert!(settings.ptt_hotkey.is_empty());
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let original = Settings::default();
        let serialized = serde_json::to_value(&original).expect("serialize failed");
        let deserialized: Settings =
            serde_json::from_value(serialized).expect("deserialize failed");
        assert_eq!(deserialized.ptt_hotkey, original.ptt_hotkey);
        assert_eq!(deserialized.selected_model, original.selected_model);
        assert_eq!(deserialized.confidence_threshold, original.confidence_threshold);
    }

    #[test]
    fn test_settings_roundtrip_with_custom_values() {
        let original = Settings {
            ptt_hotkey: "RightOption".to_string(),
            selected_model: "small".to_string(),
            confidence_threshold: 0.75,
        };
        let serialized = serde_json::to_value(&original).expect("serialize failed");
        let deserialized: Settings =
            serde_json::from_value(serialized).expect("deserialize failed");
        assert_eq!(deserialized.ptt_hotkey, "RightOption");
        assert_eq!(deserialized.selected_model, "small");
        assert_eq!(deserialized.confidence_threshold, 0.75_f32);
    }

    #[test]
    fn test_default_ptt_hotkey_constant_value() {
        assert_eq!(DEFAULT_PTT_HOTKEY, "AltRight");
        assert!(!DEFAULT_PTT_HOTKEY.is_empty());
    }
}
