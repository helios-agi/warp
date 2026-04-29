use super::{derive_agent_attribution_toggle_state, AgentAttributionToggleState};
use super::{extract_yaml_frontmatter, extract_yaml_value};
use crate::workspaces::workspace::AdminEnablementSetting;

#[test]
fn respect_user_setting_returns_user_pref_unlocked() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        true,
        true,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: false,
            is_disabled: false,
        }
    );
}

#[test]
fn respect_user_setting_with_user_off_returns_unchecked_unlocked() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        false,
        true,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: false,
            is_forced_by_org: false,
            is_disabled: false,
        }
    );
}

#[test]
fn team_enable_locks_toggle_on_regardless_of_user_pref() {
    let state = derive_agent_attribution_toggle_state(&AdminEnablementSetting::Enable, false, true);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

#[test]
fn team_disable_locks_toggle_off_regardless_of_user_pref() {
    let state = derive_agent_attribution_toggle_state(&AdminEnablementSetting::Disable, true, true);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: false,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

#[test]
fn ai_globally_disabled_marks_toggle_disabled_but_not_forced() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        true,
        false,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: false,
            is_disabled: true,
        }
    );
}

#[test]
fn team_force_takes_precedence_over_global_ai_disabled() {
    let state =
        derive_agent_attribution_toggle_state(&AdminEnablementSetting::Enable, false, false);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

// YAML parsing tests (V-007)

#[test]
fn test_frontmatter_basic() {
    let content = "---\nname: scout\ndescription: recon agent\n---\n# Body";
    let fm = extract_yaml_frontmatter(content).unwrap();
    assert!(fm.contains("name: scout"));
    assert!(fm.contains("description: recon agent"));
}

#[test]
fn test_frontmatter_no_opening_fence() {
    let content = "no frontmatter here";
    assert!(extract_yaml_frontmatter(content).is_none());
}

#[test]
fn test_frontmatter_mid_block_dashes() {
    // --- inside a value should NOT be treated as closing fence
    let content = "---\nname: test\nnotes: use --- carefully\n---\nbody";
    let fm = extract_yaml_frontmatter(content).unwrap();
    // Should capture up to the closing ---, not the one inside notes
    assert!(fm.contains("name: test"));
    assert!(fm.contains("notes: use --- carefully"));
}

#[test]
fn test_frontmatter_crlf() {
    let content = "---\r\nname: test\r\ndescription: windows\r\n---\r\nbody";
    let fm = extract_yaml_frontmatter(content);
    assert!(fm.is_some());
    let fm = fm.unwrap();
    assert!(fm.contains("name: test"));
}

#[test]
fn test_field_basic() {
    let yaml = "name: scout\ndescription: recon agent";
    assert_eq!(
        extract_yaml_value(yaml, "name"),
        Some("scout".to_string())
    );
    assert_eq!(
        extract_yaml_value(yaml, "description"),
        Some("recon agent".to_string())
    );
}

#[test]
fn test_field_with_colon_in_value() {
    let yaml = "description: bolt://127.0.0.1:7687";
    assert_eq!(
        extract_yaml_value(yaml, "description"),
        Some("bolt://127.0.0.1:7687".to_string())
    );
}

#[test]
fn test_field_quoted() {
    let yaml = "name: \"scout\"";
    assert_eq!(
        extract_yaml_value(yaml, "name"),
        Some("scout".to_string())
    );
}

#[test]
fn test_field_missing() {
    let yaml = "name: scout";
    assert_eq!(extract_yaml_value(yaml, "skills"), None);
}
