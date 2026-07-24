use crate::ast::{MarkupDoc, MarkupNode, MarkupStyle, Span};
use crate::token::{self, SpannedToken, Token};
use crate::ast::MARKERS;

/// Инкрементальный документ zoll.
///
/// Держит исходный текст, токены с позициями и AST.
/// При правке через [`edit`](Self::edit) перетокенизирует только
/// изменившийся диапазон и перепарсивает только затронутую часть AST.
pub struct IncrementalDoc {
    /// Полный исходный текст.
    pub source: String,
    /// Все токены с байтовыми позициями.
    pub tokens: Vec<SpannedToken>,
    /// AST (перестраивается частично при каждой правке).
    pub ast: MarkupDoc,
}

impl IncrementalDoc {
    /// Создать новый документ с полным парсингом текста.
    pub fn new(source: &str) -> Self {
        let tokens = token::tokenize(source);
        let ast = crate::parser::parse(&tokens);
        Self {
            source: source.to_string(),
            tokens,
            ast,
        }
    }

    /// Применить правку: удалить `[from..to)` и вставить `text`.
    ///
    /// Возвращает ссылку на обновлённый AST.
    ///
    /// ## Алгоритм
    ///
    /// 1. Применить правку к `source`
    /// 2. Найти `dirty_start` — байт, с которого нужно перетокенизировать:
    ///    - начало строки, содержащей `from`
    ///    - или раньше, если строка внутри multiline-маркера (`%%`, `$$`, `!!!`)
    /// 3. Удалить все старые токены с позицией ≥ `dirty_start`
    /// 4. Перетокенизировать `source[dirty_start..]`
    /// 5. Перепарсить только dirty-диапазон токенов инкрементально
    pub fn edit(&mut self, from: usize, to: usize, text: &str) -> &MarkupDoc {
        // 1. Применить правку к source
        self.source.replace_range(from..to, text);

        // 2. Найти dirty_start
        let dirty_start = self.find_dirty_start(from);

        // 3. Найти индекс первого грязного токена
        let dirty_token_idx = self
            .tokens
            .iter()
            .position(|t| t.start >= dirty_start)
            .unwrap_or(self.tokens.len());

        // 4. Удалить старые токены от dirty_token_idx
        self.tokens.truncate(dirty_token_idx);

        // 5. Перетокенизировать от dirty_start до конца
        let new_tokens = token::tokenize_range(&self.source, dirty_start..self.source.len());
        self.tokens.extend(new_tokens);

        // 6. Перепарсить инкрементально: только dirty-хвост токенов
        self.ast = incremental_parse(&self.tokens, &self.ast, dirty_token_idx);

        &self.ast
    }

    /// Определяет байт, с которого нужно начинать перетокенизацию.
    ///
    /// В простом случае — начало строки, содержащей `from`.
    /// Если строка находится внутри multiline-маркера (`%%`, `$$`, `!!!`),
    /// откатывается до открытия этого маркера.
    fn find_dirty_start(&self, from: usize) -> usize {
        // Начало строки, содержащей `from` (в новом source)
        let line_start = self.source[..from]
            .rfind('\n')
            .map(|p| p + 1)
            .unwrap_or(0);

        // Сканируем токены от начала до line_start,
        // отслеживая открытые multiline-маркеры
        let mut open_multiline: Vec<MarkupStyle> = Vec::new();
        let mut earliest_open = line_start;

        for st in &self.tokens {
            if st.start >= line_start {
                break;
            }
            match &st.token {
                Token::Open(style) if is_multiline(*style) => {
                    if open_multiline.is_empty() {
                        earliest_open = st.start;
                    }
                    open_multiline.push(*style);
                }
                Token::Close(style) if is_multiline(*style) => {
                    if let Some(idx) = open_multiline.iter().rposition(|s| s == style) {
                        open_multiline.remove(idx);
                        if open_multiline.is_empty() {
                            earliest_open = line_start;
                        }
                    }
                }
                _ => {}
            }
        }

        if open_multiline.is_empty() {
            line_start
        } else {
            earliest_open
        }
    }
}

/// Проверяет, является ли стиль multiline-маркером.
fn is_multiline(style: MarkupStyle) -> bool {
    MARKERS.iter().any(|m| m.style == style && m.multiline)
}

// ---------------------------------------------------------------------------
// Инкрементальный парсинг
// ---------------------------------------------------------------------------

/// Инкрементально перепарсить токены, начиная с `dirty_start`.
///
/// Вместо полного перепарса:
/// 1. Отрезает от `old_ast` поддерево, начиная с `dirty_start`
/// 2. Инициализирует стек парсера состоянием из отрезанного поддерева
/// 3. Парсит только токены `tokens[dirty_start..]`
fn incremental_parse(
    tokens: &[SpannedToken],
    old_ast: &MarkupDoc,
    dirty_start: usize,
) -> MarkupDoc {
    // Если dirty_start = 0 — перепарсиваем всё
    if dirty_start == 0 {
        return crate::parser::parse(tokens);
    }

    // Отрезаем clean-часть старого AST до dirty_start
    let CutResult {
        children: mut init_children,
        stack: init_stack,
    } = cut_ast(old_ast, dirty_start);

    // Превращаем стек в формат парсера (с open_idx = dirty_start для всех)
    let mut stack: Vec<(Vec<MarkupNode>, MarkupStyle, usize)> =
        init_stack.into_iter().map(|(c, s)| (c, s, dirty_start)).collect();
    let current_children = &mut init_children;

    for (i, st) in tokens[dirty_start..].iter().enumerate() {
        let abs_i = dirty_start + i;
        match &st.token {
            Token::Text(t) => {
                current_children.push(MarkupNode::Text(
                    t.clone(),
                    Span::new(abs_i, abs_i + 1),
                ));
            }
            Token::Newline => {
                current_children.push(MarkupNode::Text(
                    "\n".to_string(),
                    Span::new(abs_i, abs_i + 1),
                ));
            }
            Token::Open(style) => {
                let saved = std::mem::take(current_children);
                stack.push((saved, *style, abs_i));
                *current_children = Vec::new();
            }
            Token::Close(style) => {
                if let Some(idx) = stack.iter().rposition(|(_, s, _)| s == style) {
                    let open_idx = stack[idx].2;
                    let formatted = MarkupNode::Formatted {
                        style: *style,
                        children: std::mem::take(current_children),
                        span: Span::new(open_idx, abs_i + 1),
                    };

                    let mut merged = vec![formatted];

                    for _ in (idx + 1..stack.len()).rev() {
                        let (mut orphan_children, orphan_style, orphan_open_idx) =
                            stack.pop().unwrap();
                        orphan_children.push(MarkupNode::Formatted {
                            style: orphan_style,
                            children: std::mem::take(current_children),
                            span: Span::new(orphan_open_idx, abs_i + 1),
                        });
                        *current_children = orphan_children;
                        merged = std::mem::take(current_children);
                    }

                    let (mut parent_children, _parent_style, _) = stack.pop().unwrap();
                    parent_children.extend(merged);
                    *current_children = parent_children;
                } else {
                    current_children.push(MarkupNode::Text(
                        marker_text_for_close(*style),
                        Span::new(abs_i, abs_i + 1),
                    ));
                }
            }
        }
    }

    // Закрываем оставшиеся на стеке
    while let Some((mut parent_children, orphan_style, orphan_open_idx)) = stack.pop() {
        parent_children.push(MarkupNode::Formatted {
            style: orphan_style,
            children: std::mem::take(current_children),
            span: Span::new(orphan_open_idx, orphan_open_idx + 1),
        });
        *current_children = parent_children;
    }

    MarkupDoc {
        children: std::mem::take(&mut init_children),
    }
}

/// Результат cut_ast: чистые children + стек незакрытых Formatted.
struct CutResult {
    /// Children корневого уровня, полностью чистые (до dirty_start).
    children: Vec<MarkupNode>,
    /// Стек парсера: для каждого незакрытого Formatted —
    /// (children_собранные_до_этого_Formatted, стиль).
    stack: Vec<(Vec<MarkupNode>, MarkupStyle)>,
}

/// Отрезать от AST всё, начиная с `dirty_start` (индекс токена).
///
/// Возвращает `CutResult`:
/// - `children` — узлы корневого уровня, полностью лежащие до `dirty_start`
/// - `stack` — стек незакрытых Formatted (начинаем с самого глубокого),
///   готовый к передаче парсеру
///
/// Если dirty_start падает внутрь Formatted, этот Formatted кладётся на стек,
/// и мы рекурсивно спускаемся в его детей.
fn cut_ast(old_ast: &MarkupDoc, dirty_start: usize) -> CutResult {
    cut_children(&old_ast.children, dirty_start, Vec::new())
}

/// Обрезает список children по dirty_start, накапливая стек.
///
/// - `children` — узлы текущего уровня
/// - `dirty_start` — индекс токена, с которого всё грязно
/// - `stack` — текущий стек (аккумулятор)
///
/// Возвращает (clean_children_текущего_уровня, итоговый_стек).
fn cut_children(
    children: &[MarkupNode],
    dirty_start: usize,
    mut stack: Vec<(Vec<MarkupNode>, MarkupStyle)>,
) -> CutResult {
    let mut result: Vec<MarkupNode> = Vec::new();

    for node in children {
        let span = node.span();
        if span.end <= dirty_start {
            // Весь узел до dirty — сохраняем
            result.push(node.clone());
        } else if span.start >= dirty_start {
            // Узел начинается с dirty — всё остальное отрезано
            // result содержит children этого уровня до dirty
            return CutResult {
                children: result,
                stack,
            };
        } else {
            // span.start < dirty_start < span.end — частичное пересечение
            // Только Formatted может иметь span > 1
            if let MarkupNode::Formatted {
                style,
                children: sub_children,
                ..
            } = node
            {
                // Кладём текущий уровень на стек и спускаемся
                stack.push((result, *style));
                return cut_children(sub_children, dirty_start, stack);
            } else {
                // Text c span > 1 — невозможно, но на всякий случай отрезаем
                return CutResult {
                    children: result,
                    stack,
                };
            }
        }
    }

    // Все children обработаны, dirty не достигнут
    CutResult {
        children: result,
        stack,
    }
}

fn marker_text_for_close(style: MarkupStyle) -> String {
    MARKERS
        .iter()
        .find(|m| m.style == style)
        .map(|m| m.close.to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MarkupNode;

    /// Извлекает текст из AST (без стилей).
    fn collect_text(doc: &MarkupDoc) -> String {
        fn flatten(node: &MarkupNode) -> String {
            match node {
                MarkupNode::Text(t, _) => t.clone(),
                MarkupNode::Formatted { children, .. } => {
                    children.iter().map(flatten).collect()
                }
            }
        }
        doc.children.iter().map(flatten).collect()
    }

    #[test]
    fn initial_parse() {
        let doc = IncrementalDoc::new("hello world");
        assert_eq!(collect_text(&doc.ast), "hello world");
    }

    #[test]
    fn insert_text() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(5, 5, " beautiful");
        assert_eq!(collect_text(&doc.ast), "hello beautiful world");
    }

    #[test]
    fn delete_text() {
        let mut doc = IncrementalDoc::new("hello beautiful world");
        doc.edit(6, 16, "");
        assert_eq!(collect_text(&doc.ast), "hello world");
    }

    #[test]
    fn replace_text() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(6, 11, "there");
        assert_eq!(collect_text(&doc.ast), "hello there");
    }

    #[test]
    fn insert_inside_bold() {
        let mut doc = IncrementalDoc::new("**bold**");
        doc.edit(4, 4, "X");
        assert_eq!(collect_text(&doc.ast), "boXld");
        assert_eq!(doc.ast.children.len(), 1);
        assert!(matches!(
            &doc.ast.children[0],
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::BOLD)
        ));
    }

    #[test]
    fn replace_with_marker() {
        let mut doc = IncrementalDoc::new("hello world");
        doc.edit(0, 5, "**hello**");
        let text = collect_text(&doc.ast);
        assert_eq!(text, "hello world");
        assert!(doc.ast.children.iter().any(|node| matches!(
            node,
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::BOLD)
        )));
    }

    #[test]
    fn edit_inside_formula() {
        let mut doc = IncrementalDoc::new("$$sum = 5$$\nplain text");
        let pos = doc.source.find("plain").unwrap();
        doc.edit(pos, pos, "X");
        let text = collect_text(&doc.ast);
        assert_eq!(text, "sum = 5\nXplain text");
        assert!(doc.ast.children.iter().any(|node| matches!(
            node,
            MarkupNode::Formatted { style, .. } if style.contains(MarkupStyle::DISPLAY_FORMULA)
        )));
    }

    #[test]
    fn edit_outside_formula() {
        let mut doc = IncrementalDoc::new("$$sum = 5$$\nplain text");
        let pos = doc.source.find("plain").unwrap();
        doc.edit(pos, pos, "X");
        let text = collect_text(&doc.ast);
        assert!(text.contains("Xplain"));
    }

    #[test]
    fn tokens_are_positioned_correctly_after_edit() {
        let mut doc = IncrementalDoc::new("a **b** c");
        doc.edit(4, 5, "xyz");
        assert_eq!(doc.tokens.len(), 5);
        assert_eq!(doc.tokens[0].token, Token::Text("a ".to_string()));
        assert_eq!(doc.tokens[0].start, 0);
        assert_eq!(doc.tokens[1].start, 2);
        assert_eq!(doc.tokens[2].token, Token::Text("xyz".to_string()));
        assert_eq!(doc.tokens[2].start, 4);
        assert_eq!(doc.tokens[2].end, 7);
        assert_eq!(doc.tokens[3].start, 7);
        assert_eq!(doc.tokens[3].end, 9);
        assert_eq!(doc.tokens[4].start, 9);
        assert_eq!(doc.tokens[4].end, 11);
    }

    #[test]
    fn multiple_edits() {
        let mut doc = IncrementalDoc::new("");
        doc.edit(0, 0, "a");
        doc.edit(1, 1, "b");
        doc.edit(2, 2, "c");
        assert_eq!(collect_text(&doc.ast), "abc");
    }

    #[test]
    fn edit_after_comment() {
        let mut doc = IncrementalDoc::new("%%comment%%\nline");
        let pos = doc.source.find("line").unwrap() + "line".len();
        doc.edit(pos, pos, "!");
        let text = collect_text(&doc.ast);
        assert!(text.contains("line!"));
    }

    /// Проверка: инкрементальный парсинг даёт тот же результат, что и полный.
    /// Для набора сценариев правок сравниваем AST из edit() с полным перепарсом.
    #[test]
    fn incremental_parse_matches_full_parse() {
        let cases = [
            // (исходник, позиция_правки, from, to, вставляемый_текст)
            ("hello world", "hello ", 5, 5, " beautiful"),
            ("**bold**", "bo", 2, 2, "X"),
            ("**bold**", "bo", 2, 4, "XYZ"),
            ("**bold**", "", 0, 0, "!!spoiler!!"),
            ("a **b** c", "b", 4, 5, "xyz"),
            ("$$sum = 5$$\nplain text", "plain", 12, 12, "XXX"),
            ("%%comment%%\nline", "line", 12, 12, "!"),
            ("**a //b// c**", "b", 6, 7, "XYZ"),
            ("hello\nworld\nfoo", "world", 6, 11, "there"),
            ("", "", 0, 0, "hello **bold** world"),
        ];

        for (source, _label, from, to, text) in &cases {
            // Инкрементальный путь
            let mut inc_doc = IncrementalDoc::new(source);
            inc_doc.edit(*from, *to, text);

            // Полный перепарс
            let full_tokens = crate::token::tokenize(&inc_doc.source);
            let full_ast = crate::parser::parse(&full_tokens);

            // Сравниваем текст
            let inc_text = collect_text(&inc_doc.ast);
            let full_text = collect_text(&full_ast);
            assert_eq!(inc_text, full_text,
                "Mismatch for source={:?}, edit({}, {}, {:?})",
                source, from, to, text);

            // Сравниваем структуру: общее число Formatted узлов
            fn count_formatted(node: &MarkupNode) -> usize {
                match node {
                    MarkupNode::Formatted { children, .. } => {
                        1 + children.iter().map(count_formatted).sum::<usize>()
                    }
                    MarkupNode::Text(_, _) => 0,
                }
            }
            let inc_count: usize = inc_doc.ast.children.iter().map(count_formatted).sum();
            let full_count: usize = full_ast.children.iter().map(count_formatted).sum();
            assert_eq!(inc_count, full_count,
                "Formatted count mismatch for source={:?}, edit({}, {}, {:?})",
                source, from, to, text);
        }
    }

    /// Проверка спанов: span'ы узлов в инкрементальном AST должны
    /// совпадать с span'ами в полном парсинге.
    #[test]
    fn spans_match_full_parse() {
        let source = "hello **bold** world";
        let mut doc = IncrementalDoc::new(source);
        doc.edit(6, 6, " very"); // "**bold**" → "** very bold**"

        // Полный перепарс
        let full_tokens = crate::token::tokenize(&doc.source);
        let full_ast = crate::parser::parse(&full_tokens);

        // Сравниваем спаны всех узлов рекурсивно
        fn check_spans(inc_node: &MarkupNode, full_node: &MarkupNode) {
            match (inc_node, full_node) {
                (MarkupNode::Text(t1, s1), MarkupNode::Text(t2, s2)) => {
                    assert_eq!(t1, t2, "Text content mismatch");
                    assert_eq!(s1, s2, "Span mismatch for Text({:?})", t1);
                }
                (MarkupNode::Formatted { style: s1, children: c1, span: sp1 },
                 MarkupNode::Formatted { style: s2, children: c2, span: sp2 }) => {
                    assert_eq!(s1, s2, "Style mismatch");
                    assert_eq!(sp1, sp2, "Span mismatch for Formatted({:?})", s1);
                    assert_eq!(c1.len(), c2.len(), "Children count mismatch for Formatted({:?})", s1);
                    for (ic, fc) in c1.iter().zip(c2.iter()) {
                        check_spans(ic, fc);
                    }
                }
                _ => panic!("Node type mismatch: {:?} vs {:?}", inc_node, full_node),
            }
        }

        for (inc_child, full_child) in doc.ast.children.iter().zip(full_ast.children.iter()) {
            check_spans(inc_child, full_child);
        }
    }

    /// Проверка: cut_ast + перепарс dirty-хвоста даёт корректный результат.
    #[test]
    fn cut_and_repair_ast() {
        let source = "hello **bold** world";
        let tokens = crate::token::tokenize(source);
        let ast = crate::parser::parse(&tokens);

        // Симулируем редактирование: заменяем "bold" на "strong"
        let dirty_token_idx = 2; // индекс токена Text("bold")
        let dirty_start = 8;     // байт 'b' в исходнике

        let mut new_tokens: Vec<SpannedToken> = tokens[..dirty_token_idx].to_vec();
        new_tokens.push(SpannedToken {
            token: crate::token::Token::Text("strong".to_string()),
            start: dirty_start,
            end: dirty_start + 6,
        });
        new_tokens.extend_from_slice(&tokens[dirty_token_idx + 1..]);

        let repaired = incremental_parse(&new_tokens, &ast, dirty_token_idx);
        let full_ast = crate::parser::parse(&new_tokens);

        let inc_text = collect_text(&repaired);
        let full_text = collect_text(&full_ast);
        assert_eq!(inc_text, full_text,
            "cut_and_repair: text mismatch. inc={:?}, full={:?}",
            inc_text, full_text);
        assert_eq!(repaired.children.len(), full_ast.children.len(),
            "cut_and_repair: children count mismatch");
    }

    // -----------------------------------------------------------------------
    // Бенчмарк: сравнение производительности инкрементального и полного парсинга
    // -----------------------------------------------------------------------

    /// Генерирует большой документ с повторяющимися маркерами.
    /// Каждая строка: `line_NNN **bold_NNN** //italic_NNN//\n`
    fn generate_large_doc(lines: usize) -> String {
        let mut s = String::with_capacity(lines * 60);
        for i in 0..lines {
            s.push_str(&format!("line_{:04} **bold_{:04}** //italic_{:04}//\n", i, i, i));
        }
        s
    }

    #[test]
    fn benchmark_incremental_vs_full_parse() {
        let lines = 5000; // ~195KB документа
        let source = generate_large_doc(lines);

        // Холодный старт: полный парсинг (токенизация + AST)
        let start = std::time::Instant::now();
        let mut doc = IncrementalDoc::new(&source);
        let full_initial_time = start.elapsed();

        // Правка: вставляем символ в середину документа
        let edit_pos = source.len() / 2;

        // ИНКРЕМЕНТАЛЬНЫЙ ПУТЬ: edit() делает dirty-токенизацию + inc-парсинг
        let start = std::time::Instant::now();
        doc.edit(edit_pos, edit_pos, "X");
        let inc_total_time = start.elapsed();

        // ПОЛНЫЙ ПУТЬ: токенизация всего + полный парсинг
        let start = std::time::Instant::now();
        let full_tokens = crate::token::tokenize(&doc.source);
        let tokenize_time = start.elapsed();

        let start = std::time::Instant::now();
        let _full_ast = crate::parser::parse(&full_tokens);
        let parse_time = start.elapsed();

        let full_total_time = tokenize_time + parse_time;

        // Проверяем корректность
        assert_eq!(collect_text(&doc.ast), collect_text(&_full_ast),
            "Benchmark: AST mismatch after incremental edit");

        // Результаты
        let fmt = |d: std::time::Duration| format!("{:.3} ms", d.as_secs_f64() * 1000.0);

        println!("\n--- BENCHMARK ({} lines, {} KB) ---", lines, source.len() / 1024);
        println!("Full initial (tokenize + parse):  {}", fmt(full_initial_time));
        println!("Full from scratch:");
        println!("  Tokenize:                      {}", fmt(tokenize_time));
        println!("  Parse:                         {}", fmt(parse_time));
        println!("  Total:                         {}", fmt(full_total_time));
        println!("Incremental (tokenize + parse):  {}", fmt(inc_total_time));

        let ratio = if inc_total_time < full_total_time && full_total_time.as_nanos() > 0 {
            full_total_time.as_secs_f64() / inc_total_time.as_secs_f64()
        } else {
            0.0
        };
        if ratio > 1.0 {
            println!("Speedup: {:.1}x over full from scratch", ratio);
        } else {
            println!("No speedup (incremental slower or equal)");
        }
    }
}
