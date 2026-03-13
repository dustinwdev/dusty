use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn signal_get_untracked(c: &mut Criterion) {
    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(42_i64).expect("runtime initialized");

    c.bench_function("signal_get_untracked", |b| {
        b.iter(|| {
            black_box(sig.with_untracked(|v| *v).expect("signal alive"));
        });
    });

    dusty_reactive::dispose_runtime();
}

fn signal_get_tracked(c: &mut Criterion) {
    use dusty_reactive::bench_support::{pop_tracking, push_tracking, register_subscriber};

    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(42_i64).expect("runtime initialized");
    let sub_id = register_subscriber(|| {}).expect("runtime initialized");

    c.bench_function("signal_get_tracked", |b| {
        push_tracking(sub_id).expect("runtime initialized");
        b.iter(|| {
            black_box(sig.get().expect("signal alive"));
        });
        pop_tracking().expect("runtime initialized");
    });

    dusty_reactive::dispose_runtime();
}

fn signal_set_1_subscriber(c: &mut Criterion) {
    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(0_i64).expect("runtime initialized");

    let _effect = dusty_reactive::create_effect(move || {
        black_box(sig.get().expect("signal alive"));
    })
    .expect("runtime initialized");

    c.bench_function("signal_set_1_subscriber", |b| {
        let mut i = 0_i64;
        b.iter(|| {
            i += 1;
            sig.set(black_box(i)).expect("signal alive");
        });
    });

    dusty_reactive::dispose_runtime();
}

fn signal_set_8_subscribers(c: &mut Criterion) {
    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(0_i64).expect("runtime initialized");

    for _ in 0..8 {
        let _effect = dusty_reactive::create_effect(move || {
            black_box(sig.get().expect("signal alive"));
        })
        .expect("runtime initialized");
    }

    c.bench_function("signal_set_8_subscribers", |b| {
        let mut i = 0_i64;
        b.iter(|| {
            i += 1;
            sig.set(black_box(i)).expect("signal alive");
        });
    });

    dusty_reactive::dispose_runtime();
}

fn signal_set_100_subscribers(c: &mut Criterion) {
    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(0_i64).expect("runtime initialized");

    for _ in 0..100 {
        let _effect = dusty_reactive::create_effect(move || {
            black_box(sig.get().expect("signal alive"));
        })
        .expect("runtime initialized");
    }

    c.bench_function("signal_set_100_subscribers", |b| {
        let mut i = 0_i64;
        b.iter(|| {
            i += 1;
            sig.set(black_box(i)).expect("signal alive");
        });
    });

    dusty_reactive::dispose_runtime();
}

fn batch_flush_100_writes(c: &mut Criterion) {
    dusty_reactive::initialize_runtime();
    let sig = dusty_reactive::create_signal(0_i64).expect("runtime initialized");

    let _effect = dusty_reactive::create_effect(move || {
        black_box(sig.get().expect("signal alive"));
    })
    .expect("runtime initialized");

    c.bench_function("batch_flush_100_writes", |b| {
        let mut i = 0_i64;
        b.iter(|| {
            dusty_reactive::batch(|| {
                for _ in 0..100 {
                    i += 1;
                    sig.set(black_box(i)).expect("signal alive");
                }
            })
            .expect("runtime initialized");
        });
    });

    dusty_reactive::dispose_runtime();
}

criterion_group!(
    benches,
    signal_get_untracked,
    signal_get_tracked,
    signal_set_1_subscriber,
    signal_set_8_subscribers,
    signal_set_100_subscribers,
    batch_flush_100_writes,
);
criterion_main!(benches);
