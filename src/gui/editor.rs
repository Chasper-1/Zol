// src/webkit/editor.rs
use gtk4::prelude::*;
use gtk4::{CssProvider, TextBuffer, TextTag, style_context_add_provider_for_display};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub fn setup_markdown_engine(buffer: &TextBuffer, css_styles: &str) {
    let provider = CssProvider::new();
    provider.load_from_data(css_styles);

    if let Some(display) = gtk4::gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let tag_table = buffer.tag_table();
    tag_table.add(&TextTag::new(Some("h1")));
    tag_table.add(&TextTag::new(Some("bold")));
    tag_table.add(&TextTag::new(Some("italic")));
    tag_table.add(&TextTag::new(Some("code_block")));
}

pub fn render_markdown_to_buffer(buffer: &TextBuffer, markdown_text: &str) {
    let (start, end) = buffer.bounds();
    buffer.remove_all_tags(&start, &end);

    let parser = Parser::new(markdown_text);
    let mut active_tags: Vec<String> = Vec::new();

    // Нам нужны смещения в байтах, чтобы сопоставить их с pulldown-cmark
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } if level == pulldown_cmark::HeadingLevel::H1 => {
                    active_tags.push("h1".to_string());
                }
                Tag::Strong => active_tags.push("bold".to_string()),
                Tag::Emphasis => active_tags.push("italic".to_string()),
                Tag::CodeBlock(_) => active_tags.push("code_block".to_string()),
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    active_tags.retain(|t| t != "h1");
                }
                TagEnd::Strong => {
                    active_tags.retain(|t| t != "bold");
                }
                TagEnd::Emphasis => {
                    active_tags.retain(|t| t != "italic");
                }
                TagEnd::CodeBlock => {
                    active_tags.retain(|t| t != "code_block");
                }
                _ => {}
            },
            Event::Text(_) => {
                // Конвертируем байтовые индексы диапазона в символьные индексы UTF-8 для GTK
                let start_char = markdown_text[..range.start].chars().count() as i32;
                let end_char = markdown_text[..range.end].chars().count() as i32;

                let tag_start = buffer.iter_at_offset(start_char);
                let tag_end = buffer.iter_at_offset(end_char);

                for tag_name in &active_tags {
                    buffer.apply_tag_by_name(tag_name, &tag_start, &tag_end);
                }
            }
            _ => {}
        }
    }
}
