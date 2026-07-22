use super::data::EditorInner;
use api::editor as api_editor;
use editor::state::EditMode;

impl EditorInner {
    pub fn get_mode(&self) -> EditMode {
        self.mode.get()
    }

    pub fn set_mode(&self, new_mode: EditMode) {
        let mut current = self.mode.get();
        api_editor::mode_set(&mut current, new_mode);
        self.mode.set(current);
        self.mark_dirty();
    }

    pub fn cycle_mode(&self) {
        let mut current = self.mode.get();
        api_editor::mode_cycle(&mut current);
        self.mode.set(current);
        self.mark_dirty();
    }
}
