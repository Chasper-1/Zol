//! Модель разобранного документа zoll.
//!
//! Не AST — а плоское представление, где каждая строка хранит
//! свой тип и список стилизованных сегментов (диапазоны в исходном тексте).
//!
//! Три категории маркеров:
//! - **Inline** — внутри строки: `**`, `//`, `~~`, `==`, `$`, `__`, `''`, `,,`, `++`, `--`
//! - **Line** — вся строка: `#1#`, `%%`, `-`, `1.`, `>`, `---`, `#:`, `|`
//! - **Block** — несколько строк: `%%%..%%%`, `$$$..$$$`, `!!!..!!!`, ` ```..``` `

mod doc;
mod line;
mod markers;
mod seg;

pub use doc::{ParsedDoc, assign_block_roles};
pub use line::{BlockContainer, BlockKind, BlockRole, ParsedLine};
pub use markers::{MARKERS, MarkerCategory, MarkerDef};
pub use seg::{MarkStyle, Segment};
