//! Byte-exact heap attribution for the tabs subsystem (lives only on the
//! `feat/tabs-mem-bench` branch). Compiled only with `--features dhat-heap`,
//! so the global allocator and `dhat` dep never touch normal or PR builds. Run:
//!   cargo test -p blightmud --features dhat-heap tabs::dhat_probe -- --nocapture --test-threads=1
#![cfg(all(test, feature = "dhat-heap"))]

use super::tab::Tab;
use crate::tabs::{TabOpts, TabSet};
use crate::ui::History;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MIB: f64 = 1024.0 * 1024.0;

/// Tier 3 — the single cleanest number for Q1: bytes the allocator actually
/// hands out for an empty `History::new()` (eager) vs an empty lazy `Tab`.
#[test]
fn dhat_eager_vs_lazy_empty() {
    let _p = dhat::Profiler::builder().testing().build();

    let before = dhat::HeapStats::get();
    let eager = History::new(); // current behavior: eager 2x with_capacity
    let after_eager = dhat::HeapStats::get();
    let lazy = Tab::new("chat", TabOpts::default()); // fixed: for_tab (lazy)
    let after_lazy = dhat::HeapStats::get();

    let eager_delta = after_eager.curr_bytes as i64 - before.curr_bytes as i64;
    let lazy_delta = after_lazy.curr_bytes as i64 - after_eager.curr_bytes as i64;
    println!(
        "[dhat] History::new() empty   = +{eager_delta} B ({:.2} MiB)",
        eager_delta as f64 / MIB
    );
    println!("[dhat] Tab::new()    empty   = +{lazy_delta} B (lazy)");
    drop(eager);
    drop(lazy);
}

/// Tier 3/4 — a capped tab (`history_lines = 2000`) filled past its ceiling:
/// shows the bounded peak (backing + line contents) for a busy short tab.
#[test]
fn dhat_capped_tab_full() {
    let _p = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get();
    let mut tab = Tab::new(
        "combat",
        TabOpts {
            history_lines: Some(2000),
            ..Default::default()
        },
    );
    if let Some(h) = tab.history.as_mut() {
        for _ in 0..(32 * (2000 / 32) + 1000) {
            h.append("a representative combat line of moderate length");
        }
    }
    let after = dhat::HeapStats::get();
    let delta = after.curr_bytes as i64 - before.curr_bytes as i64;
    println!(
        "[dhat] cap(2000) tab FULL     = +{delta} B ({:.2} MiB)",
        delta as f64 / MIB
    );
}

/// Tier 5 — byte-exact leak assertion across create/drop cycles.
#[test]
fn dhat_create_drop_no_leak() {
    let _p = dhat::Profiler::builder().testing().build();
    // Warm up once so any one-time lazy initialization is already paid for and
    // can't be mistaken for a leak in the measured loop below.
    {
        let mut ts = TabSet::new();
        ts.create("warm", TabOpts::default()).unwrap();
        drop(ts);
    }
    let baseline = dhat::HeapStats::get().curr_bytes;
    for _ in 0..50 {
        let mut ts = TabSet::new();
        for i in 0..20 {
            ts.create(&format!("t{i}"), TabOpts::default()).unwrap();
        }
        drop(ts);
    }
    let after = dhat::HeapStats::get().curr_bytes;
    println!("[dhat] create/drop leak: baseline={baseline} after={after}");
    assert_eq!(after, baseline, "tab create/drop must not leak heap");
}
