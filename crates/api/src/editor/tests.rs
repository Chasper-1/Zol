use editor::state::EditMode;
use super::*;

fn make_mode() -> EditMode {
    EditMode::Live
}

#[test]
fn mode_set_changes_value() {
    let mut m = make_mode();
    assert_eq!(m, EditMode::Live);
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
    assert_eq!(mode_name(EditMode::Live), "live");
}

#[test]
fn mode_name_source() {
    assert_eq!(mode_name(EditMode::Source), "source");
}

#[test]
fn mode_cycle_from_preview() {
    let mut m = EditMode::Preview;
    mode_cycle(&mut m);
    assert_eq!(m, EditMode::Live);
}

#[test]
fn mode_cycle_from_live_preview() {
    let mut m = EditMode::Live;
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
    let mut count = 0;
    while count < 3 {
        mode_cycle(&mut m);
        count += 1;
    }
    assert_eq!(m, EditMode::Preview);
}

#[test]
fn mode_is_editable_preview_is_false() {
    assert!(!mode_is_editable(EditMode::Preview));
}

#[test]
fn mode_is_editable_live_preview_is_true() {
    assert!(mode_is_editable(EditMode::Live));
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