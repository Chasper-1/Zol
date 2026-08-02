//! Одна строка документа: тип + сегменты.

use super::seg::{MarkStyle, Segment};

// Тип строки — чем она является структурно.
//
// Категории маркеров:
// - **Inline**: маркеры внутри строки → отражаются в `segments`
// - **Line**: вся строка помечена → `BlockKind` указывает тип
// - **Block**: многострочный блок → первая и последняя строка имеют маркер,
//   внутренние строки — `BlockContent`
#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    // Обычный текст с inline-маркерами.
    Paragraph,

    // Пустая строка (только пробелы/перенос).
    Empty,

    // ── Line-маркеры ────────────────────────────────────────────
    // Заголовок уровня N: `#N# Title`
    Header(u32),

    // Маркированный список: `- item` или `* item`
    Bullet,

    // Нумерованный список: `1. item`
    Ordered(u32),

    // Цитата: `> text`
    Quote,

    // Горизонтальный разделитель: `---`, `___`, `***`
    ThematicBreak,

    // Тэг: `#:tag_name`
    Tag(String),

    // Строка таблицы: `| a | b |`
    TableRow,

    // ── Line-level комментарий/спойлер/формула ───────────────────
    // `%% комментарий до конца строки`
    CommentLine,
    // `!! спойлер до конца строки (с опциональным заголовком)`
    SpoilerLine(Option<String>),
    // `$$ формула до конца строки`
    FormulaLine,

    // ── Block-маркеры ────────────────────────────────────────────
    // Строка открытия/закрытия или содержимое блок-контейнера.
    // Указывает, какой именно блок (`Comment`, `Spoiler`, `Formula`, `Code`).
    Block {
        kind: BlockContainer,
        // Для `SpoilerBlock` — опциональный заголовок.
        // Для `CodeBlock` — язык (может быть пустым).
        title: Option<String>,
        // Позиция строки в блоке.
        role: BlockRole,
    },
}

// Тип блок-контейнера.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockContainer {
    // %%%...%%%
    Comment,
    // $$$...$$$
    Formula,
    // !!!...!!!
    Spoiler,
    // ```...```
    Code,
}

// Роль строки внутри блок-контейнера.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockRole {
    // Открывающая строка (`%%%`, `$$$`, `!!!`, ` ```lang`).
    Open,
    // Строка содержимого внутри блока.
    Content,
    // Закрывающая строка (`%%%`, `$$$`, `!!!`, ` ``` `).
    Close,
}

impl BlockKind {
    // Можно ли редактировать содержимое строки (не является чистым маркером).
    pub fn is_editable(&self) -> bool {
        !matches!(self, BlockKind::Empty | BlockKind::ThematicBreak)
    }
}

// Результат разбора одной строки.
//
// Владеет исходным текстом и списком сегментов (диапазоны в `source`).
#[derive(Debug, Clone)]
pub struct ParsedLine {
    // Исходный текст строки (без `\n`).
    pub source: String,
    // Тип строки.
    pub kind: BlockKind,
    // Стилизованные сегменты — диапазоны в `source`.
    // Для строк без inline-маркеров: один сегмент на всю строку со стилем `PLAIN`.
    // Для строк, где содержимое не парсится (код, комментарий): сегментов нет.
    pub segments: Vec<Segment>,
}

impl ParsedLine {
    pub fn new(source: &str, kind: BlockKind, segments: Vec<Segment>) -> Self {
        Self {
            source: source.to_string(),
            kind,
            segments,
        }
    }

    // Сегмент, покрывающий всю строку со стилем PLAIN.
    pub fn whole(source: &str, kind: BlockKind) -> Self {
        let len = source.len();
        Self {
            source: source.to_string(),
            kind,
            segments: vec![Segment::new(0, len, MarkStyle::PLAIN)],
        }
    }

    // Пустая строка.
    pub fn empty() -> Self {
        Self {
            source: String::new(),
            kind: BlockKind::Empty,
            segments: Vec::new(),
        }
    }
}
