//! Integration tests for the profiler.

#![allow(clippy::unwrap_used)]

use std::time::Duration;

use dusty_devtools::profiler::{diff_snapshots, snapshot_runtime, FrameTimer};
use dusty_reactive::{create_signal, dispose_runtime, initialize_runtime};

#[test]
fn empty_timer_returns_none() {
    let timer = FrameTimer::new(60);
    assert!(timer.stats().is_none());
    assert!(timer.is_empty());
}

#[test]
fn push_duration_records_frame() {
    let mut timer = FrameTimer::new(60);
    timer.push_duration(Duration::from_millis(16));

    let stats = timer.stats().unwrap();
    assert_eq!(stats.frame_count, 1);
    assert_eq!(stats.min, Duration::from_millis(16));
    assert_eq!(stats.max, Duration::from_millis(16));
    assert_eq!(stats.avg, Duration::from_millis(16));
    assert_eq!(stats.last, Duration::from_millis(16));
}

#[test]
fn begin_end_records_frame() {
    let mut timer = FrameTimer::new(60);
    timer.begin_frame();
    timer.end_frame();

    assert_eq!(timer.len(), 1);
    let stats = timer.stats().unwrap();
    assert_eq!(stats.frame_count, 1);
}

#[test]
fn capacity_eviction() {
    let mut timer = FrameTimer::new(3);
    timer.push_duration(Duration::from_millis(10));
    timer.push_duration(Duration::from_millis(20));
    timer.push_duration(Duration::from_millis(30));
    assert_eq!(timer.len(), 3);

    timer.push_duration(Duration::from_millis(40));
    assert_eq!(timer.len(), 3);

    let stats = timer.stats().unwrap();
    // 10ms was evicted
    assert_eq!(stats.min, Duration::from_millis(20));
    assert_eq!(stats.max, Duration::from_millis(40));
}

#[test]
fn min_max_avg() {
    let mut timer = FrameTimer::new(10);
    timer.push_duration(Duration::from_millis(10));
    timer.push_duration(Duration::from_millis(20));
    timer.push_duration(Duration::from_millis(30));

    let stats = timer.stats().unwrap();
    assert_eq!(stats.min, Duration::from_millis(10));
    assert_eq!(stats.max, Duration::from_millis(30));
    assert_eq!(stats.avg, Duration::from_millis(20));
    assert_eq!(stats.last, Duration::from_millis(30));
}

#[test]
fn p95_with_100_frames() {
    let mut timer = FrameTimer::new(200);
    for i in 1..=100 {
        timer.push_duration(Duration::from_millis(i));
    }

    let stats = timer.stats().unwrap();
    assert_eq!(stats.frame_count, 100);
    assert_eq!(stats.p95, Duration::from_millis(95));
}

#[test]
fn p95_with_single_frame() {
    let mut timer = FrameTimer::new(60);
    timer.push_duration(Duration::from_millis(16));

    let stats = timer.stats().unwrap();
    assert_eq!(stats.p95, Duration::from_millis(16));
}

#[test]
fn begin_without_end_no_recording() {
    let mut timer = FrameTimer::new(60);
    timer.begin_frame();
    // No end_frame
    assert!(timer.is_empty());
}

#[test]
fn end_without_begin_no_recording() {
    let mut timer = FrameTimer::new(60);
    timer.end_frame();
    assert!(timer.is_empty());
}

#[test]
fn snapshot_reads_runtime() {
    initialize_runtime();
    let _sig = create_signal(42).unwrap();

    let snap = snapshot_runtime().unwrap();
    assert_eq!(snap.live_signals, 1);
    assert!(!snap.signals.is_empty());

    dispose_runtime();
}

#[test]
fn diff_detects_signal_changes() {
    initialize_runtime();
    let sig = create_signal(0).unwrap();

    let before = snapshot_runtime().unwrap();
    sig.set(1).unwrap();
    sig.set(2).unwrap();
    let after = snapshot_runtime().unwrap();

    let report = diff_snapshots(&before, &after);
    assert_eq!(report.signal_deltas.len(), 1);
    assert_eq!(report.signal_deltas[0].version_delta, 2);

    dispose_runtime();
}

#[test]
fn diff_no_changes_empty_deltas() {
    initialize_runtime();
    let _sig = create_signal(0).unwrap();

    let before = snapshot_runtime().unwrap();
    let after = snapshot_runtime().unwrap();

    let report = diff_snapshots(&before, &after);
    assert!(report.signal_deltas.is_empty());

    dispose_runtime();
}

#[test]
fn diff_multiple_signals() {
    initialize_runtime();
    let sig_a = create_signal(0).unwrap();
    let sig_b = create_signal(0).unwrap();
    let _sig_c = create_signal(0).unwrap(); // unchanged

    let before = snapshot_runtime().unwrap();
    sig_a.set(1).unwrap();
    sig_b.set(1).unwrap();
    sig_b.set(2).unwrap();
    let after = snapshot_runtime().unwrap();

    let report = diff_snapshots(&before, &after);
    assert_eq!(report.signal_deltas.len(), 2);

    let delta_a = report.signal_deltas.iter().find(|d| d.index == 0).unwrap();
    assert_eq!(delta_a.version_delta, 1);

    let delta_b = report.signal_deltas.iter().find(|d| d.index == 1).unwrap();
    assert_eq!(delta_b.version_delta, 2);

    dispose_runtime();
}
