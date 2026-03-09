use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_corefetch"))
}

fn make_temp_home(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("corefetch-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temporary home directory should be created");
    path
}

fn write_config(home: &std::path::Path, file_name: &str) {
    let target = home.join(".config").join("corefetch");
    fs::create_dir_all(&target).expect("config directory should be created");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("config")
        .join(file_name);
    fs::copy(fixture, target.join("config.toml")).expect("config fixture should be copied");
}

fn run_corefetch(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    run_corefetch_with_env(home, args, &[])
}

fn run_corefetch_with_env(
    home: &std::path::Path,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> std::process::Output {
    let mut command = Command::new(binary_path());
    command.env("HOME", home).args(args);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    let output = command.output().expect("corefetch should run");

    fs::remove_dir_all(home).expect("temporary home directory should be cleaned up");
    output
}

#[test]
fn default_runtime_without_config_uses_fetch_output() {
    let home = make_temp_home("default-runtime");

    let output = run_corefetch(&home, &[]);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");

    assert!(output.status.success());
    assert!(stdout.contains("OS:"));
    assert!(stdout.contains("Shell:"));
    assert!(stdout.contains("Terminal:"));
    assert!(stdout.lines().count() >= 6);
}

#[test]
fn configured_runtime_uses_minimal_output_and_module_preferences() {
    let home = make_temp_home("configured-runtime");
    write_config(&home, "basic.toml");

    let output = run_corefetch(&home, &[]);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    let trimmed = stdout.trim_end();

    assert!(output.status.success());
    assert!(!trimmed.contains('\n'));
    assert!(trimmed.starts_with("Shell "));
    assert!(trimmed.contains(" | OS "));
    assert!(!trimmed.contains("Terminal "));
}

#[test]
fn explicit_json_flag_overrides_configured_default_mode() {
    let home = make_temp_home("json-override-runtime");
    write_config(&home, "basic.toml");

    let output = run_corefetch_with_env(
        &home,
        &["--json"],
        &[
            ("TERM_PROGRAM", "Ghostty"),
            ("TERM", "xterm-256color"),
            ("COLORTERM", "truecolor"),
        ],
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("json output should parse");

    assert!(output.status.success());
    assert_eq!(parsed["version"], "0.3.0");
    assert_eq!(parsed["modules"][0]["key"], "shell");
    assert_eq!(parsed["terminal"]["capability"], "truecolor");
    assert_eq!(parsed["terminal"]["unicode"], true);
    assert!(
        parsed["modules"]
            .as_array()
            .is_some_and(|modules| { modules.iter().all(|module| module["key"] != "terminal") })
    );
}

#[test]
fn invalid_config_fails_with_actionable_error() {
    let home = make_temp_home("invalid-config-runtime");
    write_config(&home, "invalid-mode.toml");

    let output = run_corefetch(&home, &[]);
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid utf8");

    assert!(!output.status.success());
    assert!(stderr.contains("corefetch: invalid output.default_mode"));
}
