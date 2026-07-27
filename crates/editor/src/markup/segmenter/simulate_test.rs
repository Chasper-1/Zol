//! Тест, имитирующий точный сценарий пользователя:
//! набор `**текст**` по символам, потом пробел, потом бэкспейс.
//! Проверяет сегменты на каждом шаге.

#[cfg(test)]
mod simulate {
    use crate::markup::segmenter::incremental_to_cache;
    use zoll::incremental::IncrementalDoc;
    use zoll::viewport::Viewport;

    fn dump(inc: &IncrementalDoc, label: &str) {
        let cache = incremental_to_cache(inc);
        println!("=== {} ===", label);
        println!("  source: {:?}", inc.source);
        println!("  line_asts: {} lines", inc.line_asts.len());
        for (i, ast) in inc.line_asts.iter().enumerate() {
            println!("  line {}: {:?}", i, ast);
        }
        for (i, lc) in cache.lines.iter().enumerate() {
            println!("  cache line {}: {} segments", i, lc.segments.len());
            for (j, seg) in lc.segments.iter().enumerate() {
                println!(
                    "    seg {}: text={:?} raw=[{},{}] left_marker={} right_marker={} cat={:?}",
                    j,
                    seg.text,
                    seg.raw_start,
                    seg.raw_end,
                    seg.left_marker_len,
                    seg.right_marker_len,
                    seg.category
                );
            }
        }
        println!("  block_of_line: {:?}", cache.block_of_line);
    }

    #[test]
    fn simulate_typing_bold_text() {
        let vp = Viewport {
            first_line: 0,
            last_line: 10,
        };
        let mut inc = IncrementalDoc::new("");

        // 1. Типаем **текст** по символам
        let chars: Vec<char> = "**текст**".chars().collect();
        let mut pos = 0usize;

        for (i, &ch) in chars.iter().enumerate() {
            let s = ch.to_string();
            inc.edit_visible(pos, pos, &s, &vp);
            pos += s.len();
        }

        dump(&inc, "after typing **текст**");

        let cache = incremental_to_cache(&inc);
        assert_eq!(cache.lines.len(), 1);
        assert_eq!(
            cache.lines[0].segments.len(),
            1,
            "должен быть 1 сегмент для bold"
        );
        let seg = &cache.lines[0].segments[0];
        assert_eq!(
            seg.left_marker_len, 2,
            "left_marker_len должен быть 2 после полного ввода"
        );

        // 2. Пробел
        inc.edit_visible(pos, pos, " ", &vp);
        pos += 1; // pos = 15
        dump(&inc, "after space");

        let cache = incremental_to_cache(&inc);
        assert_eq!(
            cache.lines[0].segments.len(),
            2,
            "пробел создаёт 2-й сегмент (plain text)"
        );
        let seg = &cache.lines[0].segments[0];
        assert_eq!(
            seg.left_marker_len, 2,
            "left_marker_len должен сохраниться после пробела"
        );
        assert_eq!(
            seg.text, "текст",
            "текст сегмента не должен включать пробел"
        );

        // 3. Backspace (удаляем пробел)
        inc.edit_visible(pos - 1, pos, "", &vp); // удаляем с 14 по 15
        pos -= 1; // pos = 14
        dump(&inc, "after backspace");

        let cache = incremental_to_cache(&inc);
        assert_eq!(
            cache.lines[0].segments.len(),
            1,
            "после удаления пробела должен быть 1 сегмент"
        );
        let seg = &cache.lines[0].segments[0];
        assert_eq!(
            seg.left_marker_len, 2,
            "left_marker_len = 2 после возврата к исходному тексту"
        );
        assert_eq!(seg.raw_start, 2);
        assert_eq!(seg.raw_end, 12);
        assert_eq!(seg.text, "текст");
    }

    #[test]
    fn simulate_typing_then_move_away_and_back() {
        use crate::layout::compute::compute_line_runs;
        use crate::layout::reveal::RevealCtx;
        use crate::theme::EditorTheme;

        let vp = Viewport {
            first_line: 0,
            last_line: 10,
        };
        let mut inc = IncrementalDoc::new("");

        // Типаем **текст**
        for ch in "**текст**".chars() {
            let s = ch.to_string();
            inc.edit_visible(inc.source.len(), inc.source.len(), &s, &vp);
        }

        let theme = EditorTheme::default();
        let cache = incremental_to_cache(&inc);

        // Собираем compute_line_runs с reveal: cursor на строке
        let ctx_on = RevealCtx {
            cursor_raw: Some(14),
            cursor_line: Some(0),
            block_of_line: &cache.block_of_line,
        };
        let runs = compute_line_runs(
            "**текст**",
            0,
            0,
            Some(&cache.lines[0]),
            14.0,
            22.0,
            false,
            Some(&ctx_on),
            &theme,
        );
        assert_eq!(runs.len(), 3, "cursor на строке → 3 runs");
        assert_eq!(runs[0].text, "**");
        assert_eq!(runs[1].text, "текст");
        assert_eq!(runs[2].text, "**", "закрывающие маркеры ДОЛЖНЫ быть");

        // cursor ушёл
        let ctx_off = RevealCtx {
            cursor_raw: Some(0),
            cursor_line: Some(5),
            block_of_line: &cache.block_of_line,
        };
        let runs = compute_line_runs(
            "**текст**",
            0,
            0,
            Some(&cache.lines[0]),
            14.0,
            22.0,
            false,
            Some(&ctx_off),
            &theme,
        );
        assert_eq!(runs.len(), 1, "cursor не на строке → 1 run");
        assert_eq!(runs[0].text, "текст");

        // cursor вернулся
        let runs = compute_line_runs(
            "**текст**",
            0,
            0,
            Some(&cache.lines[0]),
            14.0,
            22.0,
            false,
            Some(&ctx_on),
            &theme,
        );
        assert_eq!(runs.len(), 3, "cursor вернулся → снова 3 runs");
        assert_eq!(runs[0].text, "**");
        assert_eq!(runs[1].text, "текст");
        assert_eq!(
            runs[2].text, "**",
            "закрывающие маркеры должны появиться снова"
        );
    }
}
