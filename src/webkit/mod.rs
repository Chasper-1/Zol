// src/webkit/mod.rs
use gtk::prelude::*;
use gtk::Application;
use webkit6::prelude::*;
use webkit6::{WebView, Settings};

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
    
    // Создаем правый контейнер
    let preview_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    // ЗАСТАВЛЯЕМ КОНТЕЙНЕР РАСТЯГИВАТЬСЯ НА ВСЁ ОКНО
    preview_container.set_vexpand(true);
    preview_container.set_hexpand(true);

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
    // ЗАСТАВЛЯЕМ WEBVIEW РАСТЯГИВАТЬСЯ ВНУТРИ КОНТЕЙНЕРА
    webview.set_vexpand(true);
    webview.set_hexpand(true);

    let css_styles = include_str!("render.css");
    let editor_html = editor::generate_editor_html(css_styles);
    
    webview.load_html(&editor_html, None);
    preview_container.append(&webview);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}