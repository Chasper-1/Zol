//! Бенчмарки: полный парсинг `IncrementalDoc::new()`.
//!
//! Мерит скорость загрузки документа «с нуля» — сколько времени занимает
//! построчный парсинг + merge для разного объёма и сложности разметки.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use zoll::incremental::IncrementalDoc;

mod helpers;
use helpers::{make_headers, make_markup, make_plain};

fn bench_parse_100_plain(c: &mut Criterion) {
    let text = black_box(make_plain(100));
    c.bench_function("parse/100_plain", |b| {
        b.iter(|| IncrementalDoc::new(&text));
    });
}

fn bench_parse_1000_plain(c: &mut Criterion) {
    let text = black_box(make_plain(1000));
    c.bench_function("parse/1000_plain", |b| {
        b.iter(|| IncrementalDoc::new(&text));
    });
}

fn bench_parse_10000_plain(c: &mut Criterion) {
    let text = black_box(make_plain(10000));
    c.bench_function("parse/10000_plain", |b| {
        b.iter(|| IncrementalDoc::new(&text));
    });
}

fn bench_parse_1000_markup(c: &mut Criterion) {
    let text = black_box(make_markup(1000));
    c.bench_function("parse/1000_markup", |b| {
        b.iter(|| IncrementalDoc::new(&text));
    });
}

fn bench_parse_1000_headers(c: &mut Criterion) {
    let text = black_box(make_headers(1000));
    c.bench_function("parse/1000_headers", |b| {
        b.iter(|| IncrementalDoc::new(&text));
    });
}

criterion_group! {
    name = parse;
    config = Criterion::default().sample_size(50);
    targets =
        bench_parse_100_plain,
        bench_parse_1000_plain,
        bench_parse_10000_plain,
        bench_parse_1000_markup,
        bench_parse_1000_headers,
}
criterion_main!(parse);
