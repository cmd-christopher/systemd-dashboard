use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_format(c: &mut Criterion) {
    let t_unit = "sysstat-collect.timer";
    let t_activates = "sysstat-collect.service";
    let t_schedule = "*-*-* *:00:00";
    let t_last_abs = "2024-01-01 10:00:00";
    let t_last_rel = "10min ago";
    let t_next_abs = "2024-01-01 11:00:00";
    let t_next_rel = "50min left";
    let t_status = "active";
    let detail_status = "Some complex detail status\nwith multiple lines\nand extra info.";

    c.bench_function("format_status_text", |b| {
        b.iter(|| {
            let s = format!(
                "Unit: {}\nService: {}\nSchedule: {}\n\nLast Run: {} ({})\nNext Run: {} ({})\nStatus: {}\n\n{}",
                black_box(t_unit),
                black_box(t_activates),
                black_box(t_schedule),
                black_box(t_last_abs),
                black_box(t_last_rel),
                black_box(t_next_abs),
                black_box(t_next_rel),
                black_box(t_status),
                black_box(detail_status)
            );
            black_box(s);
        })
    });

    // Simulate cache hit
    let cached_s = format!(
        "Unit: {}\nService: {}\nSchedule: {}\n\nLast Run: {} ({})\nNext Run: {} ({})\nStatus: {}\n\n{}",
        t_unit,
        t_activates,
        t_schedule,
        t_last_abs,
        t_last_rel,
        t_next_abs,
        t_next_rel,
        t_status,
        detail_status
    );

    c.bench_function("cached_status_text", |b| {
        b.iter(|| {
            let s = black_box(&cached_s);
            black_box(s);
        })
    });
}

criterion_group!(benches, bench_format);
criterion_main!(benches);
