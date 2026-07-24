use super::*;

#[test]
fn init_and_list_families() {
    init();
    let families = list_families();
    assert!(!families.is_empty(), "Должен быть хотя бы один шрифт");
    assert!(
        families.iter().any(|f| f.to_lowercase().contains("sans")
            || f.to_lowercase().contains("serif")
            || f.to_lowercase().contains("mono")),
        "Ожидаются стандартные семейства, получено: {families:?}"
    );
}

#[test]
fn with_font_system_works() {
    init();
    let metrics = with_font_system(|_fs| {
        let buf = cosmic_text::Buffer::new_empty(cosmic_text::Metrics::new(14.0, 19.6));
        buf.metrics()
    });
    assert_eq!(metrics.font_size, 14.0);
}
