use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tui_hn_app::internal::ui::view::calculate_wrapped_title;

fn benchmark_wrap_title(c: &mut Criterion) {
    let title = "This is a very long title that needs to be wrapped across multiple lines to test the performance of the text wrapping logic in the application. It should handle various lengths and constraints gracefully.";

    c.bench_function("calculate_wrapped_title short", |b| {
        b.iter(|| calculate_wrapped_title(black_box(title), black_box(100), black_box(10)))
    });

    let long_title = title.repeat(10);
    c.bench_function("calculate_wrapped_title long", |b| {
        b.iter(|| calculate_wrapped_title(black_box(&long_title), black_box(100), black_box(10)))
    });
}

criterion_group!(benches, benchmark_wrap_title);
criterion_main!(benches);
