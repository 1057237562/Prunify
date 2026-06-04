use std::path::PathBuf;
use std::process::Command;

/// Run prunify with the given args and return (stdout, stderr, exit_code)
pub fn run_prunify(args: &[&str]) -> (String, String, i32) {
    let output = Command::new("./target/debug/prunify")
        .args(args)
        .output()
        .expect("Failed to run prunify");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    (stdout, stderr, exit_code)
}

/// Create a temporary directory for test isolation
pub fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("prunify-test-{}", name));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Create a scheme file in the given directory and return its path
pub fn scheme_fixture(dir: &PathBuf, name: &str, json: &str) -> PathBuf {
    let path = dir.join(format!("{}.json", name));
    std::fs::write(&path, json).expect("Failed to write scheme fixture");
    path
}
