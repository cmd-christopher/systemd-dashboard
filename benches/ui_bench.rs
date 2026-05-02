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

fn count_visual_lines(text: &str, max_width: u16) -> usize {
    let max_width = max_width as usize;
    if max_width == 0 {
        return 0;
    }
    let mut total_lines = 0;

    for line in text.lines() {
        if line.is_empty() {
            total_lines += 1;
            continue;
        }

        let mut line_width = 0;
        let words = line.split_inclusive(' ');

        for word in words {
            let word_len = if word.is_ascii() {
                word.len()
            } else {
                word.chars().count()
            };

            if line_width + word_len > max_width {
                if line_width > 0 {
                    total_lines += 1;
                    line_width = 0;
                }

                if word_len > max_width {
                    let full_lines = word_len / max_width;
                    let remainder = word_len % max_width;
                    if remainder == 0 {
                        total_lines += full_lines;
                    } else {
                        total_lines += full_lines;
                        line_width = remainder;
                    }
                } else {
                    line_width = word_len;
                }
            } else {
                line_width += word_len;
            }
        }

        if line_width > 0 {
            total_lines += 1;
        }
    }

    total_lines
}

fn bench_count_visual_lines(c: &mut Criterion) {
    let mut text = String::new();
    for _ in 0..100 {
        text.push_str("This is a simple ASCII log line indicating a very important event happened in the system.\n");
        text.push_str("This line contains some Unicode 🔥, but mostly ASCII.\n");
    }

    c.bench_function("count_visual_lines", |b| {
        b.iter(|| {
            std::hint::black_box(count_visual_lines(
                std::hint::black_box(&text),
                std::hint::black_box(80),
            ));
        })
    });
}

criterion_group!(benches, bench_format, bench_count_visual_lines);
criterion_main!(benches);
