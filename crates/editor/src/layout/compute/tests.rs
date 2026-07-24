    use super::*;
    use crate::cache::MarkupCache;
    use crate::theme::EditorTheme;
    use crate::markup::segment::{Segment, STYLE_BOLD, STYLE_CODE, STYLE_COMMENT, STYLE_DELETION, STYLE_DISPLAY_FORMULA, STYLE_FORMULA, STYLE_HIGHLIGHT, STYLE_INSERTION, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_SUBSCRIPT, STYLE_SUPERSCRIPT};

    fn cache(segments: Vec<Segment>) -> MarkupCache {
        MarkupCache { segments }
    }

    fn seg(style: u32, raw_start: usize, raw_end: usize) -> Segment {
        Segment {
            text: String::new(),
            style,
            left_marker_len: 0,
            right_marker_len: 0,
            raw_start,
            raw_end,
        }
    }

    #[test]
    fn plain_line_no_cache() {
        let runs = compute_line_runs("hello", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "hello");
        assert_eq!(runs[0].size, 14.0);
    }

    #[test]
    fn plain_line_with_cache_empty_segments() {
        let runs = compute_line_runs("hello", 0, Some(&cache(vec![])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "hello");
    }

    #[test]
    fn empty_line() {
        let runs = compute_line_runs("", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "");
    }

    #[test]
    fn empty_line_with_markers() {
        let runs = compute_line_runs("", 0, None, 14.0, 22.0, true, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn heading_no_markers() {
        let theme = EditorTheme::default();
        let runs = compute_line_runs("# hi", 0, None, 14.0, 22.0, false, &theme);
        assert_eq!(runs.len(), 2);
        // "# " — маркер, цвета фона (невидим)
        assert_eq!(runs[0].text, "# ");
        assert_eq!(runs[0].color, theme.background);
        assert_eq!(runs[0].size, 22.0);
        assert_eq!(runs[1].text, "hi");
        assert_eq!(runs[1].size, 22.0);
    }

    #[test]
    fn heading_with_markers() {
        let theme = EditorTheme::default();
        let runs = compute_line_runs("# hi", 0, None, 14.0, 22.0, true, &theme);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "# ");
        assert_eq!(runs[0].size, 22.0);
        assert_ne!(runs[0].color, theme.background);
        assert_eq!(runs[1].text, "hi");
    }

    #[test]
    fn heading_empty_after_prefix() {
        let theme = EditorTheme::default();
        let runs = compute_line_runs("# ", 0, None, 14.0, 22.0, false, &theme);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "# ");
        assert_eq!(runs[0].color, theme.background);
    }

    #[test]
    fn not_heading_without_space() {
        // "#no" — не заголовок (нет пробела после #)
        let runs = compute_line_runs("#no", 0, None, 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "#no");
        assert_eq!(runs[0].size, 14.0);
    }

    #[test]
    fn bold_segment_with_markers() {
        // "a **b** c" — байты: a=0, ' '=1, '*=2, '*=3, b=4, '*=5, '*=6, ' '=7, c=8
        let seg = seg(STYLE_BOLD, 2, 8);
        let runs = compute_line_runs(
            "a **b** c",
            0,
            Some(&cache(vec![seg])),
            14.0,
            22.0,
            true,
            &EditorTheme::default(),
        );
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "a "); // маркер (plain text) до сегмента
        assert_eq!(runs[1].text, "**b** "); // сегмент: **b** + пробел (raw 2..8)
        assert_eq!(runs[2].text, "c"); // маркер после сегмента
    }

    #[test]
    fn bold_segment_no_markers() {
        let theme = EditorTheme::default();
        // "a **b** c": a=0, ' '=1, **=2-3, b=4, **=5-6, ' '=7, c=8
        let seg = seg(STYLE_BOLD, 4, 5); // raw "b" = байт 4
        let runs = compute_line_runs(
            "a **b** c",
            0,
            Some(&cache(vec![seg])),
            14.0,
            22.0,
            false,
            &theme,
        );
        // Маркеры и plain-текст между сегментами склеиваются в один run
        // Ожидаем: "a **" (маркер+plain, цвет фона), "b" (BOLD), "** c" (маркер+plain, цв.фона)
        assert_eq!(runs.len(), 3, "len={:?}", runs.iter().map(|r| &r.text).collect::<Vec<_>>());
        assert_eq!(runs[0].text, "a **");
        assert_eq!(runs[0].color, theme.background);
        assert_eq!(runs[1].text, "b");
        assert_ne!(runs[1].style_flags & STYLE_BOLD, 0);
        assert_eq!(runs[2].text, "** c");
        assert_eq!(runs[2].color, theme.background);
    }

    #[test]
    fn bold_segment_color() {
        let seg = seg(STYLE_BOLD, 0, 4);
        let runs = compute_line_runs("bold", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert!(runs[0].color.r > 0.8, "bold should be reddish: {:?}", runs[0].color);
    }

    #[test]
    fn italic_segment_color() {
        let seg = seg(STYLE_ITALIC, 0, 6);
        let runs = compute_line_runs("italic", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.b > 0.8, "italic should be bluish: {:?}", runs[0].color);
    }

    #[test]
    fn code_segment() {
        let seg = seg(STYLE_CODE, 0, 4);
        let runs = compute_line_runs("code", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn insertion_color() {
        let seg = seg(STYLE_INSERTION, 0, 4);
        let runs = compute_line_runs("inst", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.g > 0.8, "insertion should be greenish: {:?}", runs[0].color);
    }

    #[test]
    fn deletion_color() {
        let seg = seg(STYLE_DELETION, 0, 4);
        let runs = compute_line_runs("del", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.r > 0.8, "deletion should be reddish: {:?}", runs[0].color);
    }

    #[test]
    fn comment_color() {
        let seg = seg(STYLE_COMMENT, 0, 7);
        let runs = compute_line_runs("comment", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert!(runs[0].color.r < 0.6, "comment should be gray: {:?}", runs[0].color);
    }

    #[test]
    fn superscript_smaller_size() {
        let seg = seg(STYLE_SUPERSCRIPT, 0, 5);
        let runs = compute_line_runs("super", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 0.7);
    }

    #[test]
    fn subscript_smaller_size() {
        let seg = seg(STYLE_SUBSCRIPT, 0, 5);
        let runs = compute_line_runs("sub", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 0.7);
    }

    #[test]
    fn formula_monospace() {
        let seg = seg(STYLE_FORMULA, 0, 4);
        let runs = compute_line_runs("form", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn display_formula_larger() {
        let seg = seg(STYLE_DISPLAY_FORMULA, 0, 4);
        let runs = compute_line_runs("disp", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs[0].size, 14.0 * 1.3);
        assert_eq!(runs[0].font_family.as_deref(), Some("monospace"));
    }

    #[test]
    fn strikethrough_does_not_change_color() {
        let seg = seg(STYLE_STRIKETHROUGH, 0, 5);
        let runs = compute_line_runs("strike", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        // strikethrough doesn't change the color, so it should stay default
        let default_color = shared::TEXT_DEFAULT;
        assert_eq!(runs[0].color.r, default_color.r);
        assert_eq!(runs[0].color.g, default_color.g);
        assert_eq!(runs[0].color.b, default_color.b);
    }

    #[test]
    fn highlight_does_not_change_color() {
        let seg = seg(STYLE_HIGHLIGHT, 0, 5);
        let runs = compute_line_runs("high", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        let default_color = shared::TEXT_DEFAULT;
        assert_eq!(runs[0].color.r, default_color.r);
        assert_eq!(runs[0].color.g, default_color.g);
        assert_eq!(runs[0].color.b, default_color.b);
    }

    #[test]
    fn multiple_segments_on_line() {
        // line = "hello **world** again"  (21 bytes)
        // seg covers "**world**" (including both marker stars): raw_start=6, raw_end=15
        // bytes: h=0,e=1,l=2,l=3,o=4,' '=5,'*'=6,'*'=7,w=8,o=9,r=10,l=11,d=12,'*'=13,'*'=14,' '=15,...
        let seg = seg(STYLE_BOLD, 6, 15);
        let runs = compute_line_runs("hello **world** again", 0, Some(&cache(vec![seg])), 14.0, 22.0, true, &EditorTheme::default());
        // 3 runs: marker "hello ", bold "**world**", marker " again"
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "hello ");
        assert_eq!(runs[1].text, "**world**");
        assert!(runs[1].style_flags & STYLE_BOLD != 0);
        assert_eq!(runs[2].text, " again");
    }

    #[test]
    fn segments_back_to_back() {
        // "abc**bold** ~~strike~~ end"
        // bytes: a=0,b=1,c=2,*=3,*=4,b=5,o=6,l=7,d=8,*=9,*=10,' '=11,'~'=12,'~'=13,s=14,t=15,r=16,i=17,k=18,e=19,'~'=20,'~'=21,' '=22,e=23,n=24,d=25
        // seg1 bold: raw 3..11 = "**bold**"
        // seg2 strikethrough: raw 11..22 = " ~~strike~~" (includes leading space to separate)
        let seg1 = seg(STYLE_BOLD, 3, 11);
        let seg2 = seg(STYLE_STRIKETHROUGH, 11, 22);
        let runs = compute_line_runs("abc**bold** ~~strike~~ end", 0, Some(&cache(vec![seg1, seg2])), 14.0, 22.0, true, &EditorTheme::default());
        // marker "abc", bold "**bold**", strikethrough " ~~strike~~", marker " end"
        assert_eq!(runs.len(), 4);
        assert_eq!(runs[0].text, "abc");
        assert_eq!(runs[1].text, "**bold**");
        assert!(runs[1].style_flags & STYLE_BOLD != 0);
        assert_eq!(runs[2].text, " ~~strike~~");
        assert!(runs[2].style_flags & STYLE_STRIKETHROUGH != 0);
        assert_eq!(runs[3].text, " end");
    }

    #[test]
    fn segment_at_line_start() {
        let seg = seg(STYLE_BOLD, 0, 4);
        let runs = compute_line_runs("bold", 0, Some(&cache(vec![seg])), 14.0, 22.0, true, &EditorTheme::default());
        assert_eq!(runs.len(), 1); // нет маркеров до сегмента, если сегмент с начала
        assert_eq!(runs[0].text, "bold");
    }

    #[test]
    fn segment_past_line_end_is_clamped() {
        // seg_end выходит за длину строки
        let seg = seg(STYLE_BOLD, 0, 100);
        let runs = compute_line_runs("short", 0, Some(&cache(vec![seg])), 14.0, 22.0, false, &EditorTheme::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "short");
    }

    fn ls(text: &str) -> Vec<usize> {
        let mut v = vec![0usize];
        for (i, c) in text.char_indices() {
            if c == '\n' { v.push(i + 1); }
        }
        v
    }

    #[test]
    fn cursor_line_bounds_works() {
        let text = "abc\ndef\nghi";
        let (start, end) = cursor_line_bounds(text, &ls(text), 1);
        assert_eq!(start, 4);
        assert_eq!(end, 7);
    }

    #[test]
    fn cursor_line_bounds_out_of_range() {
        let text = "abc";
        let (start, end) = cursor_line_bounds(text, &ls(text), 99);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn cursor_line_bounds_single_line() {
        let text = "abc";
        let (start, end) = cursor_line_bounds(text, &ls(text), 0);
        assert_eq!(start, 0);
        assert_eq!(end, 3);
    }

    #[test]
    fn markers_always_in_runs_even_when_hidden() {
        // Сегмент: "bold" (STYLE_BOLD) на строке с маркерами "**bold**"
        let s = seg(STYLE_BOLD, 2, 6); // raw_start=2 (после **), raw_end=6
        let mark_cache = cache(vec![s]);
        let theme = crate::theme::EditorTheme::default();

        // show_markers = false (Preview/LivePreview)
        let runs = compute_line_runs("**bold**", 0, Some(&mark_cache), 14.0, 22.0, false, &theme);
        // Должно быть 3 run: "**" (маркер, цвет фона), "bold" (BOLD), "**" (маркер, цвет фона)
        assert_eq!(runs.len(), 3, "должно быть 3 run: ** bold **");
        assert_eq!(runs[0].text, "**", "первый run — открывающий маркер");
        assert_eq!(
            runs[0].color, theme.background,
            "маркер должен быть цвета фона при show_markers=false"
        );
        assert_eq!(runs[1].text, "bold");
        assert_ne!(runs[1].style_flags & STYLE_BOLD, 0);
        assert_eq!(runs[2].text, "**", "третий run — закрывающий маркер");
        assert_eq!(
            runs[2].color, theme.background,
            "закрывающий маркер тоже цвета фона"
        );
    }

    #[test]
    fn markers_visible_in_source_mode() {
        let s = seg(STYLE_BOLD, 2, 6);
        let mark_cache = cache(vec![s]);
        let theme = crate::theme::EditorTheme::default();

        let runs = compute_line_runs("**bold**", 0, Some(&mark_cache), 14.0, 22.0, true, &theme);
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "**");
        // В Source маркеры серые
        assert_ne!(runs[0].color, theme.background);
        assert_eq!(runs[1].text, "bold");
        assert_eq!(runs[2].text, "**");
    }
