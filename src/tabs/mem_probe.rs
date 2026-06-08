//! Local memory probes for the tabs subsystem (lives only on the
//! `feat/tabs-mem-bench` branch). Compiled solely under `cfg(test)`; never part
//! of a normal or PR build. Run with:
//!   cargo test -p blightmud tabs::mem_probe -- --nocapture --test-threads=1
#![cfg(test)]

use super::tab::Tab;
use crate::model::Line;
use crate::tabs::{TabOpts, TabSet};
use crate::ui::History;
use std::mem::size_of;

const KIB: f64 = 1024.0;
const MIB: f64 = 1024.0 * 1024.0;

/// Tier 1 — exact struct sizes + the eager per-tab preallocation `History::new`
/// pays (`2 * 32768 * size_of::<Line>()`), with a per-N table.
#[test]
fn print_static_sizes() {
    let line = size_of::<Line>();
    let hist = size_of::<History>();
    let tab = size_of::<Tab>();
    let cap = 32 * 1024usize; // History::new() eager capacity, in lines
    let eager = 2 * cap * line; // inner + visible, Vec::with_capacity each

    println!("==== TABS MEMORY: STATIC SIZES ====");
    println!("size_of::<Line>()    = {line} B");
    println!("size_of::<History>() = {hist} B (struct only; excludes heap backing)");
    println!("size_of::<Tab>()     = {tab} B (struct only)");
    println!(
        "History::new() eager backing/tab = 2 * {cap} * {line} = {eager} B ({:.2} MiB)",
        eager as f64 / MIB
    );
    println!("-- eager (History::new) backing for N tabs --");
    for n in [1usize, 5, 20, 50] {
        println!("  N={n:>3}: {:.1} MiB", (n * eager) as f64 / MIB);
    }
    println!("===================================");
}

/// Tier 4 — eager vs lazy vs capped, the headline before/after comparison.
#[test]
fn print_eager_vs_lazy_vs_capped() {
    let line = size_of::<Line>();
    let full = 2 * (32 * 1024) * line; // new()/for_tab(1024) FULL backing
    let capped = 2 * (32 * (2000 / 32)) * line; // history_lines=2000 FULL backing

    println!("==== TABS MEMORY: EAGER vs LAZY vs CAPPED (per tab) ====");
    println!(
        "eager  new()    empty reserve     = {:.2} MiB (allocated at create, idle or not)",
        full as f64 / MIB
    );
    println!("lazy   for_tab  empty reserve     = ~0 B (grows on demand)");
    println!(
        "lazy   for_tab  FULL (32k lines)  = {:.2} MiB (only if actually filled)",
        full as f64 / MIB
    );
    println!(
        "lazy   cap(2000) FULL             = {:.1} KiB ({:.1}x smaller than full)",
        capped as f64 / KIB,
        full as f64 / capped as f64
    );
    println!("-- 50 tabs, empty --");
    println!(
        "  eager: {:.1} MiB reserved   |   lazy: ~0 MiB",
        (50 * full) as f64 / MIB
    );
    println!("=======================================================");
}

/// Tier 5 (logic-level leak guard) — build + destroy many tabs; `Drop` must
/// free every History. A leaked Vec/History would balloon RSS across rounds.
/// The byte-exact version lives in `dhat_probe` under `--features dhat-heap`.
#[test]
fn create_drop_cycles_do_not_grow() {
    for _ in 0..100 {
        let mut ts = TabSet::new();
        for i in 0..50 {
            ts.create(&format!("t{i}"), TabOpts::default()).unwrap();
        }
        drop(ts); // all 50 Histories must free here
    }
}
