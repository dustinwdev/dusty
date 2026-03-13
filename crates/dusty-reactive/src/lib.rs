//! Fine-grained reactive primitives for Dusty.
//!
//! This crate provides the core reactive system: signals that hold values
//! and automatically notify subscribers when those values change.
//!
//! # Quick start
//!
//! ```
//! # use dusty_reactive::*;
//! initialize_runtime();
//!
//! let count = create_signal(0).expect("runtime initialized");
//! assert_eq!(count.get().unwrap(), 0);
//!
//! count.set(5).unwrap();
//! assert_eq!(count.get().unwrap(), 5);
//!
//! count.update(|n| *n += 1).unwrap();
//! assert_eq!(count.get().unwrap(), 6);
//!
//! dispose_runtime();
//! ```

pub mod batch;
pub mod effect;
pub mod error;
pub mod memo;
pub mod resource;
pub(crate) mod runtime;
pub mod scope;
pub mod signal;
pub(crate) mod subscriber;

pub use batch::batch;
pub use resource::{create_resource, dispose_resource, Resource, ResourceResolver, ResourceState};
pub use subscriber::untrack;

pub use effect::{create_effect, dispose_effect, on_cleanup, Effect};
pub use error::ReactiveError;
pub use memo::{create_memo, dispose_memo, Memo};
pub use runtime::{dispose_runtime, initialize_runtime};
pub use scope::{create_child_scope, create_scope, dispose_scope, Scope};
pub use signal::{
    create_signal, create_signal_split, dispose_signal, ReadSignal, Signal, WriteSignal,
};

/// Internal helpers exposed for benchmarking. Not part of the public API.
#[doc(hidden)]
pub mod bench_support {
    pub use crate::subscriber::{pop_tracking, push_tracking, register_subscriber};
}
