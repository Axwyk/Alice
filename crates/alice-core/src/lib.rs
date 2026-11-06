//! Alice's domain contracts. No runtime, desktop framework, I/O or task spawning.
//!
//! Constructors enforce local invariants. Application services remain responsible
//! for global uniqueness, concurrency, timeouts, retention and provider protocols.
#![forbid(unsafe_code)]

pub mod conversation;
pub mod error;
pub mod message;
pub mod operation;
pub mod provider;
pub mod storage;

use std::{future::Future, pin::Pin};

/// A standard-library future, not an executor or a futures-util dependency.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
