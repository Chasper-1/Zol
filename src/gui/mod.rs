use crate::editor::EditorWidget;
use gtk4::Application;
use gtk4::prelude::*;

use rhai::Engine;
use std::fs;

pub fn build_ui(app: &Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Flint Notes")
        .default_width(1024)
        .default_height(768)
        .build();

    let src = match fs::read_to_string("theme.rhai") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("theme.rhai load error: {e}");
            String::new()
        }
    };

    let engine = Engine::new();

    let result = match engine.eval::<rhai::Map>(&src) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Rhai error: {e}");
            rhai::Map::new()
        }
    };

    eprintln!("Rhai OK: {:?}", result);

    let main_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);

    let sidebar_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let preview_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let text_view = EditorWidget::new();
    text_view.set_css_classes(&["editor"]);

    let scrolled_window = gtk4::ScrolledWindow::builder().child(&text_view).build();

    preview_container.append(&scrolled_window);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}
