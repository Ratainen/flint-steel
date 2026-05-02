//! Flint testing framework integration for `SteelMC`.
//!
//! This crate provides implementations of the Flint traits (`FlintAdapter`, `FlintWorld`,
//! `FlintPlayer`) that allow running automated tests against the `SteelMC` server.
//!
//! # Architecture
//!
//! This integration uses the **real steel-core World** for testing:
//! - `SteelTestWorld` wraps an `Arc<World>` with RAM-only storage
//! - `SteelTestPlayer` uses the real block/item behavior system
//! - Chunks are created empty on-demand (no disk I/O, no generation)
//!
//! This enables 100% code reuse with steel-core and accurate behavior testing.
//!
//! # Example
//!
//! ```ignore
//!
//! // Initialize registry and behaviors (required before creating adapter)
//! steel_flint::init();
//!
//! // Create adapter
//! let adapter = SteelAdapter::new();
//!
//! // Load and run tests
//! let selector = TestSelector::new("./tests".as_ref()).unwrap();
//! let specs = selector.load_tests(&TestFilter::all()).unwrap();
//!
//! let runner = TestRunner::new(&adapter);
//! let summary = runner.run_tests(&specs);
//! ```

mod adapter;
mod convert;
mod player;
/// Test connection implementation for Flint tests.
pub mod test_connection;
mod world;

pub use adapter::SteelAdapter;
pub use player::SteelTestPlayer;
pub use world::SteelTestWorld;

/// Re-export flint types for convenience
pub use flint_core::{TestLoader, TestRunner};

use std::sync::{Arc, OnceLock};
use steel_core::behavior;
use steel_registry::{REGISTRY, Registry};
use tokio::runtime;
use tokio::runtime::Runtime;

/// Global runtime for flint tests.
static FLINT_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

/// Initialize the `SteelMC` registry and behaviors for testing.
///
/// This must be called before creating any test worlds or adapters.
/// It's safe to call multiple times - subsequent calls are no-ops.
pub fn init() {
    // Initialize registry
    init_registry();

    // Initialize behaviors (requires registry to be initialized)
    init_behaviors();

    // Initialize runtime
    init_runtime();
}

/// Initialize the `SteelMC` registry.
fn init_registry() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Use the full Registry which registers all vanilla data
        let registry = Registry::new_vanilla();

        // Initialize the global registry
        let _ = REGISTRY.init(registry);
    });
}

/// Initialize block and item behaviors.
fn init_behaviors() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Initialize the global behavior registries
        behavior::init_behaviors();
    });
}

/// Initialize the Tokio runtime for async operations.
fn init_runtime() {
    let _ = FLINT_RUNTIME.get_or_init(|| {
        Arc::new(
            runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Flint runtime"),
        )
    });
}

/// Gets the shared Tokio runtime for flint tests.
pub(crate) fn runtime() -> Arc<Runtime> {
    init_runtime();
    FLINT_RUNTIME
        .get()
        .expect("Runtime not initialized")
        .clone()
}

/// Test helper to initialize registries (for use in test modules)
#[cfg(test)]
pub(crate) fn init_test_registries() {
    init();
}
