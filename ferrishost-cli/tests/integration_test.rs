use std::process::Command;

fn ferrishost() -> Command {
    let bin = env!("CARGO_BIN_EXE_ferrishost");
    Command::new(bin)
}

// ---------------------------------------------------------------------------
// --help / subcommand help flags
// ---------------------------------------------------------------------------

#[test]
fn top_level_help_exits_zero() {
    let output = ferrishost()
        .arg("--help")
        .output()
        .expect("failed to spawn ferrishost --help");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn top_level_help_mentions_ferrishost() {
    let output = ferrishost()
        .arg("--help")
        .output()
        .expect("failed to spawn ferrishost --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("ferrishost"),
        "expected 'ferrishost' in help output, got: {stdout}"
    );
}

#[test]
fn setup_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["setup", "--help"])
        .output()
        .expect("failed to spawn ferrishost setup --help");
    assert!(output.status.success());
}

#[test]
fn setup_help_mentions_gpu_flag() {
    let output = ferrishost()
        .args(["setup", "--help"])
        .output()
        .expect("failed to spawn ferrishost setup --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("skip-gpu") || stdout.contains("gpu"),
        "expected GPU-related flag in setup help, got: {stdout}"
    );
}

#[test]
fn status_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["status", "--help"])
        .output()
        .expect("failed to spawn ferrishost status --help");
    assert!(output.status.success());
}

#[test]
fn gpu_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["gpu", "--help"])
        .output()
        .expect("failed to spawn ferrishost gpu --help");
    assert!(output.status.success());
}

#[test]
fn upgrade_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["upgrade", "--help"])
        .output()
        .expect("failed to spawn ferrishost upgrade --help");
    assert!(output.status.success());
}

#[test]
fn uninstall_subcommand_help_exits_zero() {
    let output = ferrishost()
        .args(["uninstall", "--help"])
        .output()
        .expect("failed to spawn ferrishost uninstall --help");
    assert!(output.status.success());
}

// ---------------------------------------------------------------------------
// Invalid arguments
// ---------------------------------------------------------------------------

#[test]
fn unknown_subcommand_exits_nonzero() {
    let output = ferrishost()
        .arg("not-a-real-command")
        .output()
        .expect("failed to spawn ferrishost not-a-real-command");
    assert!(
        !output.status.success(),
        "expected non-zero exit for unknown subcommand"
    );
}

#[test]
fn unknown_flag_exits_nonzero() {
    let output = ferrishost()
        .arg("--not-a-real-flag")
        .output()
        .expect("failed to spawn ferrishost --not-a-real-flag");
    assert!(
        !output.status.success(),
        "expected non-zero exit for unknown flag"
    );
}

// ---------------------------------------------------------------------------
// Non-root guard (only meaningful when not running as root)
// ---------------------------------------------------------------------------

/// When not running as root, commands that require root should exit with a
/// non-zero code and print a useful message.  This test is skipped when the
/// test suite itself is running as root (e.g. in some CI containers).
#[test]
fn status_without_root_exits_nonzero() {
    // Skip this assertion if we are root (uid 0) because then the process
    // would actually try to run the command rather than bail out.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let output = ferrishost()
        .arg("status")
        .output()
        .expect("failed to spawn ferrishost status");
    assert!(
        !output.status.success(),
        "expected non-zero exit when not root"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("root"),
        "expected 'root' in stderr, got: {stderr}"
    );
}
