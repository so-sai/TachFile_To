use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(_c: &mut Criterion) {
    // black_box(render_page_placeholder());
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
