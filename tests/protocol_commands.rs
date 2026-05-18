use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn dectl_bin() -> String {
    env!("CARGO_BIN_EXE_dectl").to_string()
}

fn run_dectl_with_path(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    let bin_path = dectl_bin();
    let bin_dir = std::path::Path::new(&bin_path).parent().unwrap();
    let mut env = std::env::vars().collect::<std::collections::HashMap<_, _>>();
    if let Some(current_path) = env.get("PATH") {
        let new_path = format!("{}:{}", bin_dir.display(), current_path);
        env.insert("PATH".to_string(), new_path);
    }
    let mut cmd = Command::new(&bin_path);
    cmd.args(args).current_dir(cwd);
    for (key, val) in env.iter() {
        cmd.env(key, val);
    }
    cmd.output().expect("Failed to execute dectl")
}

fn run_dectl(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    run_dectl_with_path(args, cwd)
}

#[test]
fn test_exec_from_file_valid_commands() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(
        &script_path,
        "memory add First\nmemory add Second\nmemory list\n",
    )
    .unwrap();
    // Note: trailing newline is required for BufRead::lines() to yield last line

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success());
}

#[test]
fn test_exec_from_file_with_comments() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(
        &script_path,
        "# This is a comment\nmemory add Test\n# Another comment\nmemory list\n",
    )
    .unwrap();

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success());
}

#[test]
fn test_exec_from_file_with_empty_lines() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(
        &script_path,
        "memory add One\n\nmemory add Two\n\nmemory list\n",
    )
    .unwrap();

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success());
}

#[test]
fn test_exec_from_file_invalid_command_stops() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(
        &script_path,
        "memory add First\ninvalid command xyz\nmemory add Third\n",
    )
    .unwrap();

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(!output.status.success());
}

#[test]
fn test_exec_from_file_nonexistent_path() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("nonexistent.txt");

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(!output.status.success());
}

#[test]
fn test_exec_from_file_global_json_flag() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(&script_path, "memory add JSON\nmemory list\n").unwrap();

    let output = run_dectl(
        &["--json", "exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status"));
}

#[test]
fn test_exec_from_file_reads_dectl_args() {
    let tmp = TempDir::new().unwrap();
    let script_path = tmp.path().join("commands.txt");

    fs::write(&script_path, "memory add Arg\nmemory add Another\n").unwrap();

    let output = run_dectl(
        &["exec-from-file", script_path.to_str().unwrap()],
        tmp.path(),
    );
    assert!(output.status.success());
}
