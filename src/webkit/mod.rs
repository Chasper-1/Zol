// src/webkit/mod.rs
use gtk::Application;
use gtk::prelude::*;
use webkit6::prelude::*;
use webkit6::{Settings, WebView};

// Подключаем отдельный файл редактора
mod editor;

pub fn build_ui(app: &Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Flint Notes")
        .default_width(1024)
        .default_height(768)
        .build();

    let main_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    main_paned.set_position(250);

    let sidebar_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let preview_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let settings = Settings::new();
    settings.set_enable_html5_database(false);
    settings.set_enable_html5_local_storage(false);
    settings.set_enable_offline_web_application_cache(false);
    settings.set_enable_media_stream(false);
    settings.set_enable_media_capabilities(false);
    settings.set_enable_hyperlink_auditing(false);
    settings.set_enable_dns_prefetching(false);
    settings.set_enable_webgl(false);
    settings.set_auto_load_images(true);
    settings.set_enable_javascript(true);

    let webview = WebView::new();
    webview.set_settings(&settings);

    let css_styles = include_str!("render.css");

    // Вызываем функцию из отдельного файла для сборки разметки
    let editor_html = editor::generate_editor_html(css_styles);

    webview.load_html(&editor_html, None);
    preview_container.append(&webview);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}
