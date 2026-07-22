    use super::*;
    use std::time::{Duration, Instant};
    use super::grapheme::clamp_to_char_boundary;

    // ------------------------------------------------------------------
    // helpers
    // ------------------------------------------------------------------

    fn cursor_at(raw: usize, line: usize, col_visual: f32) -> Cursor {
        Cursor {
            raw,
            line,
            col_visual,
            last_blink: Instant::now(),
        }
    }

    // ------------------------------------------------------------------
    // new
    // ------------------------------------------------------------------

    #[test]
    fn new_initializes_at_zero() {
        let c = Cursor::new();
        assert_eq!(c.raw(), 0);
        assert_eq!(c.line(), 0);
        assert_eq!(c.col_visual(), 0.0);
    }

    // ------------------------------------------------------------------
    // set_raw
    // ------------------------------------------------------------------

    #[test]
    fn set_raw_moves_to_valid_byte() {
        let mut c = Cursor::new();
        c.set_raw("hello\nworld", 6);
        assert_eq!(c.raw(), 6);
        assert_eq!(c.line(), 1);
    }

    #[test]
    fn set_raw_clamps_to_content_len() {
        let mut c = Cursor::new();
        c.set_raw("abc", 100);
        assert_eq!(c.raw(), 3);
    }

    #[test]
    fn set_raw_clamps_to_char_boundary() {
        // "привет" — кириллица 2 байта на символ
        let mut c = Cursor::new();
        c.set_raw("привет", 3); // байт 3 = внутри 'р' (байты 2-3)
        assert_eq!(c.raw(), 2); // должен откатиться к началу 'р'
    }

    #[test]
    fn set_raw_line_updates() {
        let mut c = Cursor::new();
        c.set_raw("a\nb\nc", 2);
        assert_eq!(c.line(), 1); // b
        c.set_raw("a\nb\nc", 4);
        assert_eq!(c.line(), 2); // c
    }

    // ------------------------------------------------------------------
    // move_left / move_right
    // ------------------------------------------------------------------

    #[test]
    fn move_left_at_start_stays() {
        let mut c = Cursor::new();
        c.move_left("");
        assert_eq!(c.raw(), 0);
        c.move_left("abc");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_right_at_end_stays() {
        let mut c = cursor_at(3, 0, 0.0);
        c.move_right("abc");
        assert_eq!(c.raw(), 3);
    }

    #[test]
    fn move_left_right_ascii() {
        let mut c = cursor_at(2, 0, 0.0);
        c.move_left("abcd");
        assert_eq!(c.raw(), 1);
        c.move_right("abcd");
        assert_eq!(c.raw(), 2);
    }

    #[test]
    fn move_right_updates_line() {
        let mut c = cursor_at(3, 0, 0.0);
        c.move_right("abc\ndef");
        assert_eq!(c.raw(), 4); // \n
        assert_eq!(c.line(), 1);
        c.move_right("abc\ndef");
        assert_eq!(c.raw(), 5); // d
        assert_eq!(c.line(), 1);
    }

    #[test]
    fn move_left_multibyte() {
        // "a👨‍👩‍👧‍👦b" — сложный emoji ZWJ sequence (11 байт)
        let text = "a👨‍👩‍👧‍👦b";
        let mut c = cursor_at(text.len(), 0, 0.0);
        c.move_left(text);
        // должно перескочить через весь кластер
        assert!(c.raw() < text.len());
        assert_eq!(&text[c.raw()..], "b"); // должен быть перед 'b'
    }

    #[test]
    fn move_left_line_updates() {
        // "ab\ncd" → a=0,b=1,\n=2,c=3,d=4
        let mut c = cursor_at(4, 1, 0.0); // 'd'
        c.move_left("ab\ncd"); // → 'c'
        assert_eq!(c.raw(), 3);
        assert_eq!(c.line(), 1); // 'c' still on line 1
        c.move_left("ab\ncd"); // → '\n'
        assert_eq!(c.raw(), 2);
        assert_eq!(c.line(), 0); // '\n' on line 0
    }

    // ------------------------------------------------------------------
    // move_home / move_end
    // ------------------------------------------------------------------

    #[test]
    fn move_home_to_line_start() {
        let mut c = cursor_at(6, 1, 42.0);
        c.move_home("abc\ndef");
        assert_eq!(c.raw(), 4);
        assert_eq!(c.col_visual(), 0.0);
    }

    #[test]
    fn move_end_to_line_end() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_end("abc\ndef");
        assert_eq!(c.raw(), 3);
        assert_eq!(c.col_visual(), f32::MAX);
    }

    #[test]
    fn move_home_on_first_line() {
        let mut c = cursor_at(2, 0, 10.0);
        c.move_home("hello");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_end_last_line() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_end("hello");
        assert_eq!(c.raw(), 5);
    }

    // ------------------------------------------------------------------
    // move_word_left / move_word_right
    // ------------------------------------------------------------------

    #[test]
    fn move_word_left_from_middle() {
        let mut c = cursor_at(6, 0, 0.0);
        c.move_word_left("abc def ghi");
        assert_eq!(c.raw(), 4); // начало def
    }

    #[test]
    fn move_word_left_from_start() {
        let mut c = Cursor::new();
        c.move_word_left("abc");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_word_right_from_middle() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_word_right("abc def ghi");
        assert_eq!(c.raw(), 4); // начало def
    }

    #[test]
    fn move_word_right_from_end() {
        let mut c = cursor_at(11, 0, 0.0);
        c.move_word_right("abc def ghi");
        assert_eq!(c.raw(), 11);
    }

    #[test]
    fn move_word_left_skips_whitespace() {
        // "abc   def" → a=0,b=1,c=2,' '=3,' '=4,' '=5,d=6,e=7,f=8
        let mut c = cursor_at(8, 0, 0.0);
        c.move_word_left("abc   def");
        assert_eq!(c.raw(), 6); // начало 'def'
    }

    #[test]
    fn move_word_right_skips_whitespace() {
        // "abc   def" → a=0,b=1,c=2,' '=3,' '=4,' '=5,d=6,e=7,f=8
        let mut c = cursor_at(3, 0, 0.0);
        c.move_word_right("abc   def");
        assert_eq!(c.raw(), 6); // начало 'def'
    }

    #[test]
    fn move_word_left_empty_content() {
        let mut c = Cursor::new();
        c.move_word_left("");
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn move_word_right_empty_content() {
        let mut c = Cursor::new();
        c.move_word_right("");
        assert_eq!(c.raw(), 0);
    }

    // ------------------------------------------------------------------
    // move_up / move_down
    // ------------------------------------------------------------------

    #[test]
    fn move_up_goes_to_prev_line() {
        let mut c = cursor_at(5, 1, 0.0); // 'i' in "first\nsecond"
        c.move_up("first\nsecond");
        assert_eq!(c.line(), 0);
    }

    #[test]
    fn move_up_at_first_line_goes_home() {
        let mut c = cursor_at(3, 0, 10.0);
        c.move_up("first\nsecond");
        assert_eq!(c.raw(), 0);
        assert_eq!(c.col_visual(), 0.0);
    }

    #[test]
    fn move_up_preserves_col_visual() {
        let mut c = cursor_at(8, 1, 15.0); // cursor_x = 15px
        c.move_up("first\nsecond");
        assert_eq!(c.col_visual(), 15.0);
    }

    #[test]
    fn move_down_goes_to_next_line() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_down("first\nsecond");
        assert_eq!(c.line(), 1);
    }

    #[test]
    fn move_down_at_last_line_goes_end() {
        let mut c = cursor_at(0, 0, 0.0);
        c.move_down("only one line");
        assert_eq!(c.raw(), 13); // длина строки
    }

    #[test]
    fn move_down_infinite_col_visual() {
        let mut c = cursor_at(0, 0, f32::MAX);
        c.move_down("ab\ncdef");
        assert_eq!(c.raw(), 7); // конец второй строки
    }

    // ------------------------------------------------------------------
    // col_visual
    // ------------------------------------------------------------------

    #[test]
    fn col_visual_set_reset() {
        let mut c = Cursor::new();
        c.set_col_visual(123.0);
        assert_eq!(c.col_visual(), 123.0);
        c.reset_col_visual();
        assert_eq!(c.col_visual(), 0.0);
    }

    // ------------------------------------------------------------------
    // blink (фазовая: 530ms visible, 530ms hidden, repeat)
    // ------------------------------------------------------------------

    #[test]
    fn should_blink_initially_visible() {
        let c = Cursor::new();
        // last_blink = Instant::now(), фаза 0 → visible
        assert!(c.should_blink(), "cursor should be visible right after creation");
    }

    #[test]
    fn force_blink_resets_to_visible() {
        let mut c = Cursor::new();
        assert!(c.should_blink());
        c.force_blink();
        assert!(c.should_blink(), "force_blink should reset to visible phase");
    }

    #[test]
    fn blink_phase_hidden_after_600ms() {
        let c = Cursor::new();
        let start = c.last_blink;
        // 600ms спустя — фаза должна быть в hidden-половине (530…1060)
        let later = start + Duration::from_millis(600);
        assert!(!c.should_blink_at(later), "cursor should be hidden at 600ms");
    }

    #[test]
    fn blink_phase_visible_after_1100ms() {
        let c = Cursor::new();
        let start = c.last_blink;
        // 1100ms спустя — следующая visible-фаза (1060…1590)
        let later = start + Duration::from_millis(1100);
        assert!(c.should_blink_at(later), "cursor should be visible at 1100ms (next cycle)");
    }

    #[test]
    fn blink_phase_respects_force_blink_reset() {
        let mut c = Cursor::new();
        let start = c.last_blink;
        let later = start + Duration::from_millis(600);
        // принудительно сбросили в later
        c.force_blink_at(later);
        // сразу после сброса — visible
        assert!(c.should_blink_at(later), "force_blink should reset to visible at that time");
        // 600ms после сброса — hidden
        let later2 = later + Duration::from_millis(600);
        assert!(!c.should_blink_at(later2), "600ms after force_blink should be hidden");
    }

    // ------------------------------------------------------------------
    // set_line
    // ------------------------------------------------------------------

    #[test]
    fn set_line_works() {
        let mut c = Cursor::new();
        c.set_line(5);
        assert_eq!(c.line(), 5);
    }

    // ------------------------------------------------------------------
    // edge cases
    // ------------------------------------------------------------------

    #[test]
    fn move_on_empty_content_does_nothing() {
        let mut c = Cursor::new();
        c.move_left("");
        c.move_right("");
        c.move_home("");
        c.move_end("");
        c.move_word_left("");
        c.move_word_right("");
        // не паникует, raw = 0
        assert_eq!(c.raw(), 0);
    }

    #[test]
    fn set_raw_on_empty_string() {
        let mut c = Cursor::new();
        c.set_raw("", 0);
        assert_eq!(c.raw(), 0);
        assert_eq!(c.line(), 0);
    }

    #[test]
    fn move_left_then_right_roundtrip() {
        let text = "abcdef";
        let mut c = cursor_at(3, 0, 0.0);
        let original = c.raw();
        c.move_left(text);
        c.move_left(text);
        c.move_right(text);
        c.move_right(text);
        assert_eq!(c.raw(), original);
    }

    // ------------------------------------------------------------------
    // grapheme boundary helpers
    // ------------------------------------------------------------------

    #[test]
    fn prev_grapheme_boundary_at_start() {
        assert_eq!(prev_grapheme_boundary("abc", 0), None);
    }

    #[test]
    fn prev_grapheme_boundary_ascii() {
        assert_eq!(prev_grapheme_boundary("abc", 2), Some(1));
    }

    #[test]
    fn next_grapheme_boundary_at_end() {
        assert_eq!(next_grapheme_boundary("abc", 3), None);
    }

    #[test]
    fn next_grapheme_boundary_ascii() {
        assert_eq!(next_grapheme_boundary("abc", 1), Some(2));
    }

    #[test]
    fn clamp_to_char_boundary_already_valid() {
        assert_eq!(clamp_to_char_boundary("hello", 3), 3);
    }

    #[test]
    fn clamp_to_char_boundary_mid_multi_byte() {
        assert_eq!(clamp_to_char_boundary("héllo", 1), 1); // é is 2 bytes, pos 1 is inside it
    }

    #[test]
    fn clamp_to_char_boundary_past_end() {
        assert_eq!(clamp_to_char_boundary("hi", 100), 2);
    }

    #[test]
    fn clamp_to_char_boundary_empty() {
        assert_eq!(clamp_to_char_boundary("", 0), 0);
    }

    #[test]
    fn set_raw_clamps_past_end() {
        let mut c = Cursor::new();
        c.set_raw("abc", 10);
        assert_eq!(c.raw(), 3);
    }
