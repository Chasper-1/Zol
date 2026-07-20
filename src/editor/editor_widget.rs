use std::time::Duration;

use crate::editor::cache::DocumentCache;
use crate::editor::cursor::Cursor;
use crate::editor::utils::line_utils;
use crate::editor::render::{self, ShapedDocument};
use crate::editor::state::EditMode;
use crate::editor::state::EditorState;

pub struct EditorWidget {
    pub(crate) content: String,
    pub(crate) cursor: Cursor,
    pub(crate) document_cache: DocumentCache,
    pub(crate) shaped_doc: ShapedDocument,
    pub(crate) dirty: bool,
    last_active_line: usize,
}

impl EditorWidget {
    pub fn new(text: &str) -> Self {
        let content = text.to_string();
        let cursor = Cursor::new();
        let document_cache = crate::editor::markup::parse_document(&content);
        let metrics = cosmic_text::Metrics::new(14.0, 19.6);
        let empty_buffer = cosmic_text::Buffer::new_empty(metrics);
        let shaped_doc = ShapedDocument::new(empty_buffer);

        Self {
            content,
            cursor,
            document_cache,
            shaped_doc,
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

        let old_raw = self.cursor.raw();
        let had_input = crate::editor::input::handle_input(self, mode, ui);

        if had_input || self.cursor.raw() != old_raw {
            ui.ctx().request_repaint();
        }

        if self.cursor.line() != self.last_active_line && mode == EditMode::LivePreview {
            self.dirty = true;
        }
        self.last_active_line = self.cursor.line();

        let needs_rebuild = had_input || self.dirty;

        let theme = &state.theme;
        let base_size = theme.text.size;
        let heading_size = base_size * 1.6;
        let text_color = {
            use eframe::egui::Color32;
            let c = &theme.text.color;
            Color32::from_rgba_unmultiplied(
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
                (c.a * 255.0) as u8,
            )
        };

        let height = self.shaped_doc.total_height().max(ui.available_height());

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
                &mut self.shaped_doc,
                &self.content,
                &self.document_cache,
                mode,
                self.cursor.line(),
                theme,
                base_size,
                heading_size,
            );
            self.dirty = false;
        }

        if self.cursor.should_blink() {
            ui.ctx().request_repaint();
        }

        render::paint(
            &self.shaped_doc,
            &self.cursor,
            &painter,
            response.rect.min,
            text_color,
            mode,
            &self.content,
        );

        self.repaint_control(ui.ctx(), mode);
    }

    fn handle_click(&mut self, local_pos: eframe::egui::Vec2) {
        if let Some((line, byte_offset)) =
            render::click_position(&self.shaped_doc, eframe::egui::pos2(local_pos.x, local_pos.y))
        {
            let (line_start, _) = self.line_bounds(line);
            let new_raw = (line_start + byte_offset).min(self.content.len());
            self.cursor.set_raw(&self.content, new_raw);
            self.cursor.set_line(line);
            self.cursor.reset_col_visual();
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
