use std::time::Duration;

use crate::editor::cache::DocumentCache;
use crate::editor::cursor::Cursor;
use crate::editor::utils::line_utils;
use crate::editor::render::{self, Galleys};
use crate::editor::state::EditMode;
use crate::editor::state::EditorState;
use eframe::egui::text::CCursor;

pub struct EditorWidget {
    pub(crate) content: String,
    pub(crate) cursor: Cursor,
    pub(crate) document_cache: DocumentCache,
    pub(crate) galleys: Galleys,
    pub(crate) dirty: bool,
    last_active_line: usize,
}

impl EditorWidget {
    pub fn new(text: &str) -> Self {
        let content = text.to_string();
        let cursor = Cursor::new();
        let document_cache = crate::editor::markup::parse_document(&content);
        let galleys = Galleys::new();

        Self {
            content,
            cursor,
            document_cache,
            galleys,
            dirty: true,
            last_active_line: 0,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    #[allow(dead_code)]
    pub fn set_content(&mut self, text: &str) {
        self.content = text.to_string();
        self.cursor = Cursor::new();
        self.document_cache = crate::editor::markup::parse_document(&self.content);
        self.dirty = true;
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui, state: &mut EditorState) {
        let mode = state.mode;

        let old_raw = self.cursor.raw;
        let had_input = crate::editor::input::handle_input(self, mode, ui);

        if had_input || self.cursor.raw != old_raw {
            ui.ctx().request_repaint();
        }

        if self.cursor.line != self.last_active_line && mode == EditMode::LivePreview {
            self.dirty = true;
        }
        self.last_active_line = self.cursor.line;

        let needs_rebuild = had_input || self.dirty;

        let theme = &state.theme;
        let base_size = theme.text.size;
        let heading_size = base_size * 1.6;
        let text_color = theme.text.color.to_color32();

        let height = self.galleys.total_height.max(ui.available_height());

        let (response, painter) = ui.allocate_painter(
            eframe::egui::vec2(ui.available_width(), height),
            eframe::egui::Sense::click(),
        );

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let local_pos = pos - response.rect.min;
            self.handle_click(local_pos);
        }

        if needs_rebuild {
            self.document_cache = crate::editor::markup::parse_document(&self.content);
            render::build(
                &mut self.galleys,
                &self.content,
                &self.document_cache,
                mode,
                self.cursor.line,
                ui,
                theme,
                base_size,
                heading_size,
            );
            self.dirty = false;
        }

        self.cursor.blink();

        render::paint(
            &self.galleys,
            &self.cursor,
            &painter,
            response.rect.min,
            text_color,
            &self.content,
            mode,
        );

        self.repaint_control(ui.ctx(), mode);
    }

    fn handle_click(&mut self, local_pos: eframe::egui::Vec2) {
        let mut y_acc = 0.0f32;
        for (i, galley_opt) in self.galleys.galleys.iter().enumerate() {
            if let Some(galley) = galley_opt {
                let h = galley.size().y;
                if local_pos.y >= y_acc && local_pos.y < y_acc + h {
                    let click_in_row = eframe::egui::pos2(local_pos.x, local_pos.y - y_acc);
                    let egui_cursor: CCursor =
                        galley.cursor_from_pos(eframe::egui::vec2(click_in_row.x, click_in_row.y));
                    let (line_start, line_end) = self.line_bounds(i);
                    let line_text = &self.content[line_start..line_end];
                    let byte_offset = char_count_to_byte(line_text, egui_cursor.index.into());
                    self.cursor.raw = (line_start + byte_offset).min(self.content.len());
                    self.cursor.line = i;
                    self.cursor.reset_col_visual();
                    self.cursor.force_blink_on();
                    return;
                }
                y_acc += h;
            }
        }
    }

    fn repaint_control(&self, ctx: &eframe::egui::Context, mode: EditMode) {
        if mode == EditMode::Preview {
            ctx.request_repaint_after(Duration::from_secs(10));
        } else {
            ctx.request_repaint_after(Duration::from_millis(530));
        }
    }

    fn line_bounds(&self, line: usize) -> (usize, usize) {
        line_utils::line_bounds(&self.content, line)
            .map(|b| (b.start, b.end))
            .unwrap_or((0, 0))
    }
}

fn char_count_to_byte(text: &str, char_count: usize) -> usize {
    for (chars_seen, (byte_idx, _)) in text.char_indices().enumerate() {
        if chars_seen >= char_count {
            return byte_idx;
        }
    }
    text.len()
}
