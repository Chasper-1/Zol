use crate::editor::line_utils;
use std::time::{Duration, Instant};

pub struct Cursor {
    pub raw: usize,
    pub line: usize,
    col_visual: f32,
    blink_on: bool,
    last_blink: Instant,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            raw: 0,
            line: 0,
            col_visual: 0.0,
            blink_on: true,
            last_blink: Instant::now(),
        }
    }

    pub fn col_visual(&self) -> f32 {
        self.col_visual
    }

    pub fn set_col_visual(&mut self, x: f32) {
        self.col_visual = x;
    }

    pub fn reset_col_visual(&mut self) {
        self.col_visual = 0.0;
    }

    pub fn blink(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_blink) > Duration::from_millis(530) {
            self.blink_on = !self.blink_on;
            self.last_blink = now;
            true
        } else {
            false
        }
    }

    pub fn force_blink_on(&mut self) {
        self.blink_on = true;
        self.last_blink = Instant::now();
    }

    pub fn move_left(&mut self, content: &str) {
        if self.raw == 0 {
            return;
        }
        if let Some((idx, _)) = content[..self.raw].char_indices().last() {
            self.raw = idx;
        }
        self.update_line(content);
    }

    pub fn move_right(&mut self, content: &str) {
        if self.raw >= content.len() {
            return;
        }
        if let Some((next, _c)) = content[self.raw..].char_indices().nth(1) {
            self.raw += next;
        } else if !content[self.raw..].is_empty() {
            self.raw = content.len();
        }
        self.update_line(content);
    }

    pub fn move_home(&mut self, content: &str) {
        let start = self.line_start_byte(content);
        self.raw = start;
        self.col_visual = 0.0;
    }

    pub fn move_end(&mut self, content: &str) {
        let end = self.line_end_byte(content);
        self.raw = end;
        self.col_visual = f32::MAX;
    }

    pub fn update_line(&mut self, content: &str) {
        self.line = line_utils::line_of_byte(content, self.raw);
    }

    pub fn line_start_byte(&self, content: &str) -> usize {
        line_utils::line_start_byte(content, self.line)
    }

    pub fn line_end_byte(&self, content: &str) -> usize {
        line_utils::line_end_byte(content, self.line)
    }
}
