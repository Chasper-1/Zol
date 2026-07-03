use crate::editor::renderer::Renderer;
use crate::editor::{state::EditorState, theme};
use crate::editor::input::Input;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea, EventControllerKey};

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

    // ---------- Theme ----------

    let src = fs::read_to_string("theme.rhai").expect("theme.rhai not found");

    let engine = Engine::new();

    let ast = engine.compile(&src).expect("Rhai compile error");

    let rhai_map: rhai::Map = engine.eval_ast(&ast).expect("Rhai runtime error");

    let theme = theme::parse_theme(rhai_map);

    // ---------- Document ----------

    let text = fs::read_to_string("notes.md").unwrap_or_else(|_| String::new());

    let state = Rc::new(RefCell::new(EditorState::new(theme, text)));

    // ---------- Drawing ----------

    let area = DrawingArea::new();

    area.set_focusable(true);
    area.set_hexpand(true);
    area.set_vexpand(true);

    {
        let state = state.clone();

        area.set_draw_func(move |_, cr, width, height| {
            Renderer::draw(cr, width, height, &state.borrow());
        });
    }

    // ---------- Keyboard ----------

    {
        let state = state.clone();
        let area_clone = area.clone();

        let controller = EventControllerKey::new();

        controller.connect_key_pressed(move |_, key, _keycode, modifiers| {
            let mut state = state.borrow_mut();
        
            Input::handle_key(&mut state, key, modifiers);
        
            area_clone.queue_draw();
        
            gtk4::glib::Propagation::Stop
        });

        area.add_controller(controller);
    }

    window.set_child(Some(&area));

    window.present();

    area.grab_focus();
}
