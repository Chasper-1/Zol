// src/webkit/mod.rs
use gtk::Application;
use gtk::prelude::*;
use webkit6::prelude::*;
use webkit6::{Settings, WebView};

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
    sidebar_container.set_width_request(200);

    let preview_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    preview_container.set_vexpand(true);
    preview_container.set_hexpand(true);
    preview_container.set_width_request(450);

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

    let webview = WebView::builder()
        .settings(&settings)
        .vexpand(true)
        .hexpand(true)
        .build();

    let css_styles = include_str!("render.css");
    let editor_html = editor::generate_editor_html(css_styles);

    webview.load_html(&editor_html, None);
    preview_container.append(&webview);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    main_paned.set_shrink_start_child(false);
    main_paned.set_shrink_end_child(false);

    let webview_clone1 = webview.clone();
    window.connect_closure(
        "notify::default-width",
        false,
        glib::closure_local!(
            move |_win: gtk::ApplicationWindow, _param: &glib::ParamSpec| {
                webview_clone1.reload_bypass_cache();
            }
        ),
    );

    let webview_clone2 = webview.clone();
    window.connect_closure(
        "notify::default-height",
        false,
        glib::closure_local!(
            move |_win: gtk::ApplicationWindow, _param: &glib::ParamSpec| {
                webview_clone2.reload_bypass_cache();
            }
        ),
    );

    window.set_child(Some(&main_paned));
    window.present();
}
