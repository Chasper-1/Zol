use crate::editor::state::EditMode;
use super::*;

fn make_mode() -> EditMode {
    EditMode::LivePreview
}

#[test]
fn mode_set_changes_value() {
    let mut m = make_mode();
    assert_eq!(m, EditMode::LivePreview);
    mode_set(&mut m, EditMode::Preview);
    assert_eq!(m, EditMode::Preview);
}

#[test]
fn mode_get_returns_current() {
    let mut m = make_mode();
    mode_set(&mut m, EditMode::Source);
    assert_eq!(mode_get(&m), EditMode::Source);
}

#[test]
fn mode_name_preview() {
    assert_eq!(mode_name(EditMode::Preview), "preview");
}

#[test]
fn mode_name_live_preview() {
    assert_eq!(mode_name(EditMode::LivePreview), "live_preview");
}

#[test]
fn mode_name_source() {
    assert_eq!(mode_name(EditMode::Source), "source");
}

#[test]
fn mode_cycle_from_preview() {
    let mut m = EditMode::Preview;
    mode_cycle(&mut m);
    assert_eq!(m, EditMode::LivePreview);
}

#[test]
fn mode_cycle_from_live_preview() {
    let mut m = EditMode::LivePreview;
    mode_cycle(&mut m);
    assert_eq!(m, EditMode::Source);
}

#[test]
fn mode_cycle_from_source() {
    let mut m = EditMode::Source;
    mode_cycle(&mut m);
    assert_eq!(m, EditMode::Preview);
}

#[test]
fn mode_cycle_three_times_returns_to_start() {
    let mut m = EditMode::Preview;
    for _ in 0..3 {
        mode_cycle(&mut m);
    }
    assert_eq!(m, EditMode::Preview);
}

#[test]
fn mode_is_editable_preview_is_false() {
    assert!(!mode_is_editable(EditMode::Preview));
}

#[test]
fn mode_is_editable_live_preview_is_true() {
    assert!(mode_is_editable(EditMode::LivePreview));
}

#[test]
fn mode_is_editable_source_is_true() {
    assert!(mode_is_editable(EditMode::Source));
}

#[test]
fn mode_set_to_same_value() {
    let mut m = EditMode::Source;
    mode_set(&mut m, EditMode::Source);
    assert_eq!(m, EditMode::Source);
}