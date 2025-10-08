use once_cell::sync::OnceCell;
use panther_core::Engine;
use panther_domain::entities::Prompt;
use panther_observability::{init_logging, LogSink};
use panther_providers::NullProvider;
use std::sync::Arc;

static ENGINE: OnceCell<Engine> = OnceCell::new();

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn panther_init() -> i32 {
    // no-op logging on wasm for now
    let provider = Arc::new(NullProvider);
    let telemetry = Some(Arc::new(LogSink) as Arc<dyn panther_domain::ports::TelemetrySink>);
    let engine = Engine::new(provider, telemetry);
    match ENGINE.set(engine) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn panther_generate(prompt: String) -> String {
    let engine = ENGINE.get().expect("panther_init not called");
    let res = engine.generate(Prompt { text: prompt });
    match res {
        Ok(c) => serde_json::to_string(&c).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    }
}

// Native builds: provide stubs so the crate compiles in workspace
#[cfg(not(target_arch = "wasm32"))]
pub fn panther_init() -> i32 { let _ = &ENGINE; 0 }
#[cfg(not(target_arch = "wasm32"))]
pub fn panther_generate(_prompt: String) -> String { String::new() }

