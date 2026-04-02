use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a temporary directory for tests. Returns the path.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jetti-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Create a fake git repo at the given path.
fn make_git_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    // Create an initial commit so HEAD exists
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
}

/// Run jetti with the given args, returning (exit_code, stdout, stderr).
fn jetti(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_jetti"))
        .args(args)
        .output()
        .expect("failed to run jetti");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

// CLI basics

#[test]
fn help_flag() {
    let (code, stdout, _) = jetti(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("clone"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("fetch"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("status"));
}

#[test]
fn version_flag() {
    let (code, stdout, _) = jetti(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("jetti "));
}

#[test]
fn get_alias_works() {
    let (code, stdout, _) = jetti(&["get", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Clone a repository"));
}

#[test]
fn g_alias_works() {
    let (code, stdout, _) = jetti(&["g", "--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Clone a repository"));
}

#[test]
fn unknown_command() {
    let (code, _, stderr) = jetti(&["nonexistent"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("unrecognized") || stderr.contains("invalid"));
}

// root

#[test]
fn root_prints_path() {
    let dir = temp_dir("root");
    let (code, stdout, _) = jetti(&["root", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), dir.to_str().unwrap());
    cleanup(&dir);
}

// list

#[test]
fn list_empty_root() {
    let dir = temp_dir("list-empty");
    fs::create_dir_all(&dir).unwrap();
    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
    cleanup(&dir);
}

#[test]
fn list_nonexistent_root() {
    let (code, stdout, _) = jetti(&["list", "--root", "/tmp/jetti-nonexistent-root-12345"]);
    assert_eq!(code, 0);
    assert!(stdout.trim().is_empty());
}

#[test]
fn list_finds_repos() {
    let dir = temp_dir("list-repos");
    make_git_repo(&dir.join("github.com/owner/repo1"));
    make_git_repo(&dir.join("github.com/owner/repo2"));

    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    // Output is flat because stdout is not a terminal in tests
    assert!(stdout.contains("github.com/owner/repo1"));
    assert!(stdout.contains("github.com/owner/repo2"));
    cleanup(&dir);
}

#[test]
fn list_full_path() {
    let dir = temp_dir("list-full");
    make_git_repo(&dir.join("github.com/owner/repo"));

    let (code, stdout, _) = jetti(&["list", "--full-path", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains(dir.to_str().unwrap()));
    cleanup(&dir);
}

#[test]
fn list_prefix_filter() {
    let dir = temp_dir("list-prefix");
    make_git_repo(&dir.join("github.com/alice/project"));
    make_git_repo(&dir.join("github.com/bob/tool"));

    let (code, stdout, _) = jetti(&[
        "list",
        "--root",
        dir.to_str().unwrap(),
        "-p",
        "github.com/alice",
    ]);
    assert_eq!(code, 0);
    assert!(stdout.contains("alice/project"));
    assert!(!stdout.contains("bob"));
    cleanup(&dir);
}

#[test]
fn list_skips_hidden_dirs() {
    let dir = temp_dir("list-hidden");
    make_git_repo(&dir.join("github.com/owner/repo"));
    fs::create_dir_all(dir.join(".hidden/something")).unwrap();

    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(!stdout.contains(".hidden"));
    cleanup(&dir);
}

#[test]
fn list_does_not_recurse_into_repos() {
    let dir = temp_dir("list-norecurse");
    make_git_repo(&dir.join("github.com/owner/repo"));
    // Create a nested git repo inside — should not be found
    make_git_repo(&dir.join("github.com/owner/repo/subdir/nested"));

    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "github.com/owner/repo");
    cleanup(&dir);
}

#[test]
fn list_sorted_output() {
    let dir = temp_dir("list-sorted");
    make_git_repo(&dir.join("github.com/zzz/repo"));
    make_git_repo(&dir.join("github.com/aaa/repo"));

    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines[0], "github.com/aaa/repo");
    assert_eq!(lines[1], "github.com/zzz/repo");
    cleanup(&dir);
}

#[test]
fn list_multiple_hosts() {
    let dir = temp_dir("list-multi-host");
    make_git_repo(&dir.join("github.com/owner/repo"));
    make_git_repo(&dir.join("gitlab.com/owner/repo"));

    let (code, stdout, _) = jetti(&["list", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains("github.com/owner/repo"));
    assert!(stdout.contains("gitlab.com/owner/repo"));
    cleanup(&dir);
}

// rm

#[test]
fn rm_nonexistent_repo() {
    let dir = temp_dir("rm-nonexist");
    fs::create_dir_all(&dir).unwrap();

    let (code, _, stderr) = jetti(&[
        "rm",
        "owner/repo",
        "--force",
        "--root",
        dir.to_str().unwrap(),
    ]);
    assert_ne!(code, 0);
    assert!(stderr.contains("not found"));
    cleanup(&dir);
}

#[test]
fn rm_force_removes_repo() {
    let dir = temp_dir("rm-force");
    let repo_path = dir.join("github.com/owner/repo");
    make_git_repo(&repo_path);
    assert!(repo_path.exists());

    let (code, _, _) = jetti(&[
        "rm",
        "owner/repo",
        "--force",
        "--root",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    assert!(!repo_path.exists());
    cleanup(&dir);
}

#[test]
fn rm_cleans_empty_parents() {
    let dir = temp_dir("rm-parents");
    let repo_path = dir.join("github.com/owner/repo");
    make_git_repo(&repo_path);

    let (code, _, _) = jetti(&[
        "rm",
        "owner/repo",
        "--force",
        "--root",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    // Owner dir should be cleaned up
    assert!(!dir.join("github.com/owner").exists());
    // Host dir should be cleaned up too
    assert!(!dir.join("github.com").exists());
    cleanup(&dir);
}

#[test]
fn rm_preserves_sibling_repos() {
    let dir = temp_dir("rm-sibling");
    make_git_repo(&dir.join("github.com/owner/repo1"));
    make_git_repo(&dir.join("github.com/owner/repo2"));

    let (code, _, _) = jetti(&[
        "rm",
        "owner/repo1",
        "--force",
        "--root",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    assert!(!dir.join("github.com/owner/repo1").exists());
    assert!(dir.join("github.com/owner/repo2").exists());
    cleanup(&dir);
}

// clone (existing check) ───

#[test]
fn clone_existing_repo_skips() {
    let dir = temp_dir("clone-exists");
    let repo_path = dir.join("github.com/owner/repo");
    make_git_repo(&repo_path);

    let (code, stdout, stderr) = jetti(&["clone", "owner/repo", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("already exists"));
    // stdout should only have the path
    assert!(stdout.trim().ends_with("github.com/owner/repo"));
    cleanup(&dir);
}

#[test]
fn clone_recovers_from_broken_clone() {
    let dir = temp_dir("clone-broken");
    let repo_path = dir.join("github.com/owner/repo");
    // Create directory without .git — simulates a failed partial clone
    fs::create_dir_all(&repo_path).unwrap();
    assert!(!repo_path.join(".git").exists());

    let (code, _, stderr) = jetti(&["clone", "owner/repo", "--root", dir.to_str().unwrap()]);
    // Will fail because there's no real remote, but it should attempt the clone
    // (not just say "already exists")
    assert!(stderr.contains("not a git repository") || stderr.contains("cloning:") || code != 0);
    cleanup(&dir);
}

// config

#[test]
fn config_show() {
    let (code, stdout, _) = jetti(&["config"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("root:"));
    assert!(stdout.contains("default_host:"));
    assert!(stdout.contains("protocol:"));
    assert!(stdout.contains("github.com"));
}

#[test]
fn config_path() {
    let (code, stdout, _) = jetti(&["config", "path"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("jetti"));
    assert!(stdout.contains("config.toml"));
}

#[test]
fn config_init_creates_file() {
    let (code, stdout, _) = jetti(&["config", "init"]);
    // This writes to the real config location, so we just verify it doesn't crash
    // (it may say "exists" if already initialized)
    assert_eq!(code, 0);
    assert!(stdout.contains("config") || stdout.contains("exists"));
}

#[test]
fn cfg_alias_works() {
    let (code, stdout, _) = jetti(&["cfg"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("root:"));
}

// ─── completions ───

#[test]
fn completions_bash() {
    let (code, stdout, _) = jetti(&["completions", "bash"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("_jetti"));
}

#[test]
fn completions_zsh() {
    let (code, stdout, _) = jetti(&["completions", "zsh"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("compdef"));
}

#[test]
fn completions_fish() {
    let (code, stdout, _) = jetti(&["completions", "fish"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("jetti"));
}

// status

#[test]
fn status_empty_root() {
    let dir = temp_dir("status-empty");
    fs::create_dir_all(&dir).unwrap();

    let (code, _, stderr) = jetti(&["status", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("no repositories found"));
    cleanup(&dir);
}

#[test]
fn status_clean_repo() {
    let dir = temp_dir("status-clean");
    make_git_repo(&dir.join("github.com/owner/repo"));

    let (code, _, stderr) = jetti(&["status", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("clean") || stderr.contains("done"));
    cleanup(&dir);
}

#[test]
fn status_dirty_repo() {
    let dir = temp_dir("status-dirty");
    let repo_path = dir.join("github.com/owner/repo");
    make_git_repo(&repo_path);
    fs::write(repo_path.join("dirty.txt"), "hello").unwrap();

    let (code, _, stderr) = jetti(&["status", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("need attention") || stderr.contains("modified"));
    cleanup(&dir);
}

#[test]
fn status_with_prefix_filter() {
    let dir = temp_dir("status-prefix");
    make_git_repo(&dir.join("github.com/alice/repo"));
    make_git_repo(&dir.join("github.com/bob/repo"));

    let (code, _, stderr) = jetti(&[
        "status",
        "--root",
        dir.to_str().unwrap(),
        "-p",
        "github.com/alice",
    ]);
    assert_eq!(code, 0);
    assert!(stderr.contains("1 repos"));
    cleanup(&dir);
}

#[test]
fn st_alias_works() {
    let dir = temp_dir("st-alias");
    fs::create_dir_all(&dir).unwrap();
    let (code, _, _) = jetti(&["st", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    cleanup(&dir);
}

// fetch

#[test]
fn fetch_empty_root() {
    let dir = temp_dir("fetch-empty");
    fs::create_dir_all(&dir).unwrap();

    let (code, _, stderr) = jetti(&["fetch", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("no repositories found"));
    cleanup(&dir);
}

#[test]
fn fetch_local_repo() {
    let dir = temp_dir("fetch-local");
    make_git_repo(&dir.join("github.com/owner/repo"));

    // Fetch on a local-only repo (no remote) should not crash
    let (code, _, stderr) = jetti(&["fetch", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("done"));
    cleanup(&dir);
}

// update

#[test]
fn update_empty_root() {
    let dir = temp_dir("update-empty");
    fs::create_dir_all(&dir).unwrap();

    let (code, _, stderr) = jetti(&["update", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("no repositories found"));
    cleanup(&dir);
}

#[test]
fn update_clean_repo() {
    let dir = temp_dir("update-clean");
    make_git_repo(&dir.join("github.com/owner/repo"));

    let (code, _, stderr) = jetti(&["update", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("done"));
    cleanup(&dir);
}

#[test]
fn update_dirty_repo_skipped() {
    let dir = temp_dir("update-dirty");
    let repo_path = dir.join("github.com/owner/repo");
    make_git_repo(&repo_path);
    fs::write(repo_path.join("dirty.txt"), "hello").unwrap();

    let (code, _, stderr) = jetti(&["update", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stderr.contains("need attention") || stderr.contains("dirty"));
    cleanup(&dir);
}

// jobs flag

#[test]
fn fetch_custom_jobs() {
    let dir = temp_dir("fetch-jobs");
    make_git_repo(&dir.join("github.com/owner/repo"));

    let (code, _, _) = jetti(&["fetch", "--root", dir.to_str().unwrap(), "-j", "2"]);
    assert_eq!(code, 0);
    cleanup(&dir);
}

// global --root flag

#[test]
fn global_root_override() {
    let dir = temp_dir("global-root");
    let (code, stdout, _) = jetti(&["root", "--root", dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), dir.to_str().unwrap());
    cleanup(&dir);
}
