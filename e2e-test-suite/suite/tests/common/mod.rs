use e2e_harness::TestEnv;
use std::time::Duration;

/// Helper: initialise the shared test environment and return it together
/// with the ttyd WebSocket URL. Call at the top of every test.
pub async fn setup(test_name: &str) -> (&'static TestEnv, e2e_harness::config::TestContext) {
    let env = TestEnv::get().await;
    let ctx = env.new_test_context(test_name);
    (env, ctx)
}

/// Reasonable default timeout for waiting on TUI output.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
