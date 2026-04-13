mod common;

use std::time::Duration;

/// Verify that the e2e infrastructure works end to end:
///   - Docker image builds
///   - Container starts with ttyd
///   - A WebSocket terminal session can be established
///   - The app starts and produces visible output
///
/// Run with:
///   cd e2e-test-suite && cargo test --test smoke_test -- --test-threads=1
#[tokio::test]
async fn smoke_test_app_starts() {
    let (env, ctx) = common::setup("smoke").await;

    // -- Minimal config: settings + profile + one trivial service -----------
    ctx.write_file(
        "test.settings.yml",
        r#"
data_dir: .data
load_order: 0
autolaunch_profile: smoke
"#,
    )
    .expect("write settings");

    ctx.write_file(
        "smoke.profile.yml",
        r#"
id: smoke
workdir: "/e2e/smoke/config"
services:
  - id: "ping"
"#,
    )
    .expect("write profile");

    ctx.write_file(
        "services/ping.service.yml",
        r#"
id: ping
workdir: "/e2e/smoke/config"
blocks:
  - id: run
    type: process
    command:
      executable: bash
      args:
        - "-c"
        - "echo hello-from-e2e && sleep infinity"
    status_line:
      symbol: R
      slot: 10
"#,
    )
    .expect("write service");

    // -- Start the app and verify output ------------------------------------
    let mut session = ctx
        .start_app(&env.ttyd_ws_url())
        .await
        .expect("start app session");

    // The TUI should render something within a reasonable time.
    // We check for the service id appearing on screen, which confirms the
    // config was loaded and the UI rendered.
    session
        .wait_for_text("ping", Duration::from_secs(15))
        .await
        .expect("expected 'ping' to appear on screen");
}
