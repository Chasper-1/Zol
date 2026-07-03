use crate::editor::theme::EditorTheme;
use cosmic_text::{
    Attrs, AttrsList, Buffer, Family, FontSystem, LineEnding, Metrics, Scroll, Shaping, SwashCache,
    fontdb,
};

pub struct EditorState {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub buffer: Buffer,
    pub theme: EditorTheme,
}

impl EditorState {
    pub fn new(theme: EditorTheme) -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let mut font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);

        let metrics = Metrics::new(theme.text.size, theme.text.line_height);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        buffer.lines.clear();

        let lines = [
            "Flint Notes Editor",
            "Модульная структура успешно собрана.",
            "Теперь шрифты должны отображаться корректно.",
        ];

        let attrs = Attrs::new().family(Family::SansSerif);
        let attrs_list = AttrsList::new(&attrs);

        for line in lines {
            buffer.lines.push(cosmic_text::BufferLine::new(
                line.to_string(),
                LineEnding::None,
                attrs_list.clone(),
                Shaping::Advanced,
            ));
        }

        // ИСПРАВИЛИ: создаем структуру Scroll через правильный конструктор .new()
        buffer.set_scroll(Scroll::new(0, 0.0, 0.0));

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            buffer,
            theme,
        }
    }
}
