//! Local memory-harness driver (lives only on the `feat/tabs-mem-bench`
//! branch): run one headless Blightmud against the script path in argv[1], so
//! an external `/proc/<pid>` sampler can watch this single isolated process.
//! Build: `cargo build --release --example headless_run`.
use blightmud::RuntimeConfig;

fn main() {
    let script = std::env::args()
        .nth(1)
        .expect("usage: headless_run <script.lua>");
    let mut rt = RuntimeConfig::default();
    rt.headless_mode = true;
    rt.integration_test = true; // disables audio; LuaError -> clean quit
    rt.no_update_check = true; // keep the measurement off the network
    rt.script = Some(script);
    if let Err(e) = blightmud::start(rt) {
        eprintln!("blightmud exited with error: {e:?}");
        std::process::exit(1);
    }
}
