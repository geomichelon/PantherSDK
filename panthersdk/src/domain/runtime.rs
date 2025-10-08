use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub index: usize,
    pub passed: bool,
    pub output: String,
}

pub fn run_test_suite<F>(suite: &[TestCase], runner: F) -> Vec<TestResult>
where
    F: Fn(&str) -> String,
{
    suite
        .iter()
        .enumerate()
        .map(|(i, tc)| {
            let output = runner(&tc.input);
            let passed = tc
                .expected
                .as_ref()
                .map(|e| &output == e)
                .unwrap_or(true);
            TestResult { index: i, passed, output }
        })
        .collect()
}

pub fn monitor_realtime<C>(mut callback: C, ticks: usize, interval_ms: u64) -> JoinHandle<()>
where
    C: FnMut(String) + Send + 'static,
{
    std::thread::spawn(move || {
        for i in 0..ticks {
            callback(format!("tick:{}", i));
            std::thread::sleep(Duration::from_millis(interval_ms));
        }
    })
}
