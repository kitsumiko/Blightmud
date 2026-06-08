//! Tabbed scrollable output regions.
//!
//! Provides the data layer (Tab, TabSet, routing) for letting Lua scripts
//! create independently scrollable, named output buffers and bind keys to
//! switch between them. The UI layer borrows the active tab's [`History`]
//! at swap time via [`UserInterface::swap_history`].
//!
//! [`History`]: crate::ui::History
//! [`UserInterface::swap_history`]: crate::ui::UserInterface::swap_history

// Public API:
//   TabOpts — configuration passed to `blight.create_tab` (carried by Event::TabCommand)
//   TabSet  — owned by Session; routes inbound lines + holds inactive tabs' Histories
//   TabInfo — read-only snapshot used by the screen's tab indicator and Lua introspection
//
// `Tab`, `TabError`, and `RouteResult` are intentionally not re-exported here:
// they're used only inside this module (and via TabSet's public methods that
// happen to return them — those types are reachable through return-type
// inference at call sites without needing a direct import).
pub use self::tab::{TabInfo, TabOpts};
pub use self::tab_set::TabSet;

mod tab;
mod tab_set;

// Local memory-measurement probes (feat/tabs-mem-bench branch only). Never part
// of a normal or PR build: `mem_probe` is test-only; `dhat_probe` additionally
// requires the `dhat-heap` feature (which installs a global allocator).
#[cfg(test)]
mod mem_probe;
#[cfg(all(test, feature = "dhat-heap"))]
mod dhat_probe;
