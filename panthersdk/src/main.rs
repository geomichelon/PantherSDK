use panthersdk::observability::init_logging;
use panthersdk::prelude::*;
use panthersdk::providers::NullProvider;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    // Init logging for quick manual runs
    let _ = init_logging();

    let provider: Arc<dyn LlmProvider> = Arc::new(NullProvider);
    let telemetry: Option<Arc<dyn TelemetrySink>> = None;
    let engine = Engine::new(provider, telemetry);

    let out = engine.generate(Prompt { text: "Hello Panther".into() })?;
    println!("{}", out.text);
    Ok(())
}

