// src/webkit/mod.rs
use gtk4::Application;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod editor;

pub fn build_ui(app: &Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Flint Notes")
        .default_width(1024)
        .default_height(768)
        .build();

    let main_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    main_paned.set_position(250);

    // Левая панель с ограничением (не сожмется меньше 150px)
    let sidebar_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar_container.set_width_request(150);

    // Правая панель редактора с ограничением (не сожмется меньше 400px)
    let preview_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    preview_container.set_width_request(400);
    preview_container.set_vexpand(true);
    preview_container.set_hexpand(true);
    preview_container.set_css_classes(&["workspace-grid"]);

    // Интерактивный текстовый виджет карточки
    // Интерактивный текстовый виджет карточки
    let text_view = gtk4::TextView::builder()
        .editable(true)
        .cursor_visible(true)
        .wrap_mode(gtk4::WrapMode::Word)
        .vexpand(true)
        .hexpand(true)
        .css_classes(vec!["editor-content".to_string()]) // <-- Вот так правильно для Builder
        .build();

    // Обертка-карточка для визуального оформления
    let editor_card = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    editor_card.set_css_classes(&["editor-card"]);
    editor_card.append(&text_view);

    let scrolled_window = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&editor_card)
        .build();

    let css_styles = include_str!("render.css");
    let text_buffer = text_view.buffer();

    // Подключаем стили
    editor::setup_markdown_engine(&text_buffer, css_styles);

    // Удален какой-либо стартовый текст блять — буфер изначально пустой
    text_buffer.set_text("");

    // Отслеживаем ввод текста для моментального обновления разметки
    // Отслеживаем ввод текста для моментального обновления разметки
    let is_updating = Rc::new(RefCell::new(false));

    text_buffer.connect_changed(move |buf| {
        if *is_updating.borrow() {
            return;
        }

        *is_updating.borrow_mut() = true;

        let (start, end) = buf.bounds();
        let text = buf.text(&start, &end, false).to_string();

        // ИСПРАВЛЕНО: Получаем стандартный маркер курсора "insert"
        let cursor_mark = buf.mark("insert").unwrap();
        let cursor_iter = buf.iter_at_mark(&cursor_mark);
        let cursor_offset = cursor_iter.offset();

        // Парсим и накладываем CSS-теги на лету
        editor::render_markdown_to_buffer(buf, &text);

        // Возвращаем курсор на место
        let new_cursor_iter = buf.iter_at_offset(cursor_offset);
        buf.place_cursor(&new_cursor_iter);

        *is_updating.borrow_mut() = false;
    });

    preview_container.append(&scrolled_window);

    main_paned.set_start_child(Some(&sidebar_container));
    main_paned.set_end_child(Some(&preview_container));

    window.set_child(Some(&main_paned));
    window.present();
}
