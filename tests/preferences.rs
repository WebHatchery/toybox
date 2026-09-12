use toybox_after_hours::data::GameData;
use toybox_after_hours::preferences::*;

#[test]
fn older_preferences_receive_accessible_defaults() {
    let preferences: ToyboxPreferences = serde_json::from_str(r#"{"fov_degrees":95}"#).unwrap();
    assert_eq!(preferences.fov_degrees, 95.0);
    assert_eq!(preferences.mouse_sensitivity, 1.0);
    assert!(!preferences.high_contrast);
    assert!(!preferences.tutorial_complete);
}

#[test]
fn externally_edited_values_are_clamped() {
    let mut preferences = ToyboxPreferences {
        fov_degrees: 200.0,
        mouse_sensitivity: 0.1,
        ..Default::default()
    };
    preferences.sanitize_with_config(&GameData::load().unwrap().config);
    assert_eq!(preferences.fov_degrees, 110.0);
    assert_eq!(preferences.mouse_sensitivity, 0.5);
}
