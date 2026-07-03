use crate::editor::{state::EditorState, theme};
use crate::editor::renderer::Renderer;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use rhai::Engine;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Flint Notes")
        .default_width(1200)
        .default_height(800)
        .build();

    // Загружаем тему
    let src = fs::read_to_string("theme.rhai")
        .expect("theme.rhai not found");

    let engine = Engine::new();

    let ast = engine
        .compile(&src)
        .expect("Rhai compile error");

    let rhai_map: rhai::Map = engine
        .eval_ast(&ast)
        .expect("Rhai runtime error");

    let theme = theme::parse_theme(rhai_map);

    let text = std::fs::read_to_string("notes.md")
        .unwrap_or_else(|_| String::from(
            "Создай рядом с программой файл notes.md"
        ));
    
    let state = Rc::new(RefCell::new(
        EditorState::new(theme, text),
    ));

    let area = DrawingArea::new();

    let state = state.clone();
    
    area.set_draw_func(move |_, cr, width, height| {
        Renderer::draw(
            cr,
            width,
            height,
            &state.borrow(),
        );
    });

    area.set_hexpand(true);
    area.set_vexpand(true);

    window.set_child(Some(&area));

    window.present();
}