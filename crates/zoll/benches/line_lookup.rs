//! Бенчмарки: поиск номера строки по байтовой позиции.
//!
//! Сравнивает `IncrementalDoc::line_number()` (O(log n) binary search
//! через `line_starts`) с эмуляцией старого O(n) сканирования (count
//! newlines). Показывает, сколько мы выиграли на замене.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zoll::incremental::IncrementalDoc;

mod helpers;
use helpers::make_plain;

/// O(log n) — через `line_starts.binary_search()`.
fn bench_line_number_middle(c: &mut Criterion) {
    let text = make_plain(10000);
    let doc = IncrementalDoc::new(&text);
    let mid = doc.source.len() / 2;

    c.bench_function("line_lookup/line_number_mid", |b| {
        b.iter(|| {
            black_box(doc.line_number(black_box(mid)));
        });
    });
}

/// O(log n) — начало документа.
fn bench_line_number_start(c: &mut Criterion) {
    let text = make_plain(10000);
    let doc = IncrementalDoc::new(&text);

    c.bench_function("line_lookup/line_number_start", |b| {
        b.iter(|| {
            black_box(doc.line_number(black_box(0)));
        });
    });
}

/// O(log n) — конец документа.
fn bench_line_number_end(c: &mut Criterion) {
    let text = make_plain(10000);
    let doc = IncrementalDoc::new(&text);
    let end = doc.source.len().saturating_sub(1);

    c.bench_function("line_lookup/line_number_end", |b| {
        b.iter(|| {
            black_box(doc.line_number(black_box(end)));
        });
    });
}

/// Эмуляция старого O(n) подхода: count newlines.
fn bench_scan_newlines(c: &mut Criterion) {
    let text = make_plain(10000);
    let doc = IncrementalDoc::new(&text);
    let mid = doc.source.len() / 2;

    c.bench_function("line_lookup/scan_newlines_mid", |b| {
        b.iter(|| {
            let byte = black_box(mid).min(black_box(&text).len());
            // Эмулируем старый content[..byte].bytes().filter(|b| b == b'\n').count()
            let _count = text[..byte].bytes().filter(|&b| b == b'\n').count();
            black_box(_count);
        });
    });
}

criterion_group! {
    name = line_lookup;
    config = Criterion::default().sample_size(50);
    targets =
        bench_line_number_middle,
        bench_line_number_start,
        bench_line_number_end,
        bench_scan_newlines,
}
criterion_main!(line_lookup);
