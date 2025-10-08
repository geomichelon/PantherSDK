//! PantherSDK aggregator crate.
//! This crate mirrors the requested folder structure and re-exports items
//! from the internal workspace crates (domain, core, adapters, API, FFI).

pub mod domain {
    pub mod metrics;
    pub mod storage;
    pub mod bias;
    pub mod testgen;
    pub mod runtime;
    pub mod integration;
}

pub mod infrastructure {
    pub mod db_adapter;
    pub mod prometheus_adapter;
    pub mod grafana_adapter;
    pub mod integration;
}

pub mod api {
    pub mod fastapi_app;
    pub mod python_bindings;
}

pub mod prelude {
    pub use panther_core::Engine;
    pub use panther_domain::entities::*;
    pub use panther_domain::ports::*;
}

/// Common re-exports for convenience
pub mod providers {
    pub use panther_providers::NullProvider;
}

pub mod observability {
    pub use panther_observability::{init_logging, LogSink};
}
