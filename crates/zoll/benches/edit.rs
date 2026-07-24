//! Бенчмарки: инкрементальные правки.
//!
//! Сравнивает `edit()` (полный merge) с `edit_visible()` (только viewport).
//! Ради этих цифр и затевались все оптимизации — посмотрим, сколько
//! реально выигрываем на больших документах.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zoll::incremental::IncrementalDoc;
use zoll::viewport::Viewport;

mod helpers;
use helpers::make_plain;

/// Вставить один символ в середину большого документа через `edit()`.
fn bench_edit_one_char_middle(c: &mut Criterion) {
    let text = make_plain(5000);
    let mut doc = IncrementalDoc::new(&text);
    let mid = doc.source.len() / 2;

    c.bench_function("edit/one_char_middle", |b| {
        b.iter(|| {
            doc.edit(black_box(mid), black_box(mid), black_box("X"));
            black_box(&doc);
        });
    });
}

/// Вставить один символ в середину через `edit_visible()` с viewport 50 строк.
fn bench_edit_visible_50_viewport(c: &mut Criterion) {
    let text = make_plain(5000);
    let mut doc = IncrementalDoc::new(&text);
    let mid = doc.source.len() / 2;
    let vp = Viewport {
        first_line: 2475,
        last_line: 2525,
    };

    c.bench_function("edit/edit_visible_50_viewport", |b| {
        b.iter(|| {
            doc.edit_visible(black_box(mid), black_box(mid), black_box("X"), &vp);
            black_box(&doc);
        });
    });
}

/// Сравнение edit() vs edit_visible() на 10000 строк.
fn bench_edit_vs_visible_10k(c: &mut Criterion) {
    let text = make_plain(10000);
    let mut group = c.benchmark_group("edit/vs_visible_10k");

    // edit() — полный merge
    let mut doc_edit = IncrementalDoc::new(&text);
    let mid = doc_edit.source.len() / 2;
    group.bench_function("full_merge", |b| {
        b.iter(|| {
            doc_edit.edit(black_box(mid), black_box(mid), black_box("X"));
            black_box(&doc_edit);
        });
    });

    // edit_visible() — merge только 50 строк
    let mut doc_vis = IncrementalDoc::new(&text);
    let vp = Viewport {
        first_line: 4975,
        last_line: 5025,
    };
    group.bench_function("viewport_50", |b| {
        b.iter(|| {
            doc_vis.edit_visible(black_box(mid), black_box(mid), black_box("X"), &vp);
            black_box(&doc_vis);
        });
    });

    group.finish();
}

/// Вставка многострочного текста через edit_visible().
fn bench_edit_visible_multiline(c: &mut Criterion) {
    let text = make_plain(5000);
    let mut doc = IncrementalDoc::new(&text);
    let mid = doc.source.len() / 2;
    let vp = Viewport {
        first_line: 2470,
        last_line: 2530,
    };

    c.bench_function("edit/edit_visible_multiline_20", |b| {
        // Вставляем 20 строк
        let insert: String = (0..20).map(|i| format!("inserted line {}\n", i)).collect();
        b.iter(|| {
            doc.edit_visible(black_box(mid), black_box(mid), black_box(&insert), &vp);
            black_box(&doc);
        });
    });
}

criterion_group! {
    name = edit;
    config = Criterion::default().sample_size(50);
    targets =
        bench_edit_one_char_middle,
        bench_edit_visible_50_viewport,
        bench_edit_vs_visible_10k,
        bench_edit_visible_multiline,
}
criterion_main!(edit);
