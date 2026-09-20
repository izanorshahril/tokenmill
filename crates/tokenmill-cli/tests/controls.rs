use std::io::Write;
use std::process::{Command, Stdio};

struct Home(std::path::PathBuf);

impl Home {
    fn new() -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "tokenmill-controls-{}-{unique}",
            std::process::id()
        )))
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_tokenmill"));
        command.env("TOKENMILL_HOME", &self.0);
        command
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn routing_off_skips_context_and_acp() {
    let home = Home::new();
    let output = home
        .command()
        .args([
            "acp-context-prompt",
            "missing-github-copilot.exe",
            "missing-workspace",
            "missing-context.json",
            "100",
            "--routing",
            "off",
            "--report",
            "missing-directory/report.jsonl",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Routing OFF: request not submitted"));
    assert!(stdout.contains("No context read, ACP process started, or report written"));
}

#[test]
fn saved_policy_and_cli_overrides_are_explicit() {
    let home = Home::new();
    let show = home.command().args(["settings", "show"]).output().unwrap();
    let state: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(state["configured"], false);
    assert_eq!(state["routing_enabled"], true);
    assert!(!home.0.exists());
    for args in [
        vec!["settings", "init"],
        vec!["settings", "set", "saver", "off"],
        vec!["settings", "set", "mode", "compatible"],
    ] {
        assert!(home.command().args(args).output().unwrap().status.success());
    }
    let state: serde_json::Value = serde_json::from_slice(
        &home
            .command()
            .args(["settings", "show"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(state["routing_enabled"], false);
    assert_eq!(state["saver_enabled"], false);
    assert_eq!(state["mode"], "compatible");
    let request_lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(home.0.join("request.lock"))
        .unwrap();
    request_lock.lock().unwrap();
    let running: serde_json::Value = serde_json::from_slice(
        &home
            .command()
            .args(["settings", "show"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(running["running"], true);
    drop(request_lock);
    let idle: serde_json::Value = serde_json::from_slice(
        &home
            .command()
            .args(["settings", "show"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(idle["running"], false);
    let request = [
        "acp-context-prompt",
        "missing-github-copilot.exe",
        ".",
        "missing-context.json",
        "100",
    ];
    let off = home.command().args(request).output().unwrap();
    assert!(off.status.success());
    assert!(String::from_utf8_lossy(&off.stdout).contains("Routing OFF"));
    let on = home
        .command()
        .args(request)
        .args(["--routing", "on"])
        .output()
        .unwrap();
    assert!(!on.status.success());
    assert!(String::from_utf8_lossy(&on.stderr).contains("context JSON failed"));
    let failed: serde_json::Value = serde_json::from_slice(
        &home
            .command()
            .args(["settings", "show"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(failed["running"], false);
    assert_eq!(failed["last_request"]["outcome"], "incomplete");
    assert_eq!(failed["last_request"]["route_status"], "unverified");
    let original = std::fs::read(home.0.join("settings.json")).unwrap();
    assert!(
        !home
            .command()
            .args(["settings", "set", "routing", "invalid"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        original,
        std::fs::read(home.0.join("settings.json")).unwrap()
    );
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(home.0.join("settings.lock"))
        .unwrap();
    lock.lock().unwrap();
    assert!(
        !home
            .command()
            .args(["settings", "set", "routing", "on"])
            .output()
            .unwrap()
            .status
            .success()
    );
    drop(lock);
    assert_eq!(
        original,
        std::fs::read(home.0.join("settings.json")).unwrap()
    );
    std::fs::create_dir(home.0.join("settings.json.tmp")).unwrap();
    assert!(
        !home
            .command()
            .args(["settings", "set", "routing", "on"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        original,
        std::fs::read(home.0.join("settings.json")).unwrap()
    );
    std::fs::write(home.0.join("settings.json"), b"{broken").unwrap();
    let malformed = home
        .command()
        .args(request)
        .args(["--routing", "off"])
        .output()
        .unwrap();
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("invalid settings JSON"));
}

#[test]
fn tui_feedback_stays_visible_until_next_command() {
    for (input, feedback) in [
        ("h\nq\n", "[h] show these controls"),
        ("invalid\nq\n", "Unknown command. Use r, h, or q."),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tokenmill"))
            .args([
                "tui",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../docs/fixtures/visual-evidence-history.jsonl"
                ),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let after_feedback = stdout.split_once(feedback).unwrap().1;
        assert!(!after_feedback.contains("\x1b[2J"));
        assert!(after_feedback.contains("Command [r]efresh"));
    }
}
