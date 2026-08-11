// core/build.rs
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=src/api.udl");
    println!("cargo:rerun-if-env-changed=SCM_GIT_HASH");
    println!("cargo:rerun-if-env-changed=SCM_GIT_REF");
    println!("cargo:rerun-if-env-changed=SCM_BUILD_TIME");

    // Prefer CI/container-provided provenance when the source checkout does
    // not include .git, then fall back to the local checkout for development.
    let git_hash = std::env::var("SCM_GIT_HASH").unwrap_or_else(|_| {
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .map(|output| {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    "unknown".to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let git_branch = std::env::var("SCM_GIT_REF").unwrap_or_else(|_| {
        std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map(|output| {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    "unknown".to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let build_time = std::env::var("SCM_BUILD_TIME").unwrap_or_else(|_| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    });

    // Format the stamp as "hash:ref:build-time".
    let stamp = format!("{}:{}:{}", git_hash, git_branch, build_time);

    println!("cargo:rustc-env=SCM_BUILD_STAMP={}", stamp);

    if let Err(e) = uniffi::generate_scaffolding("src/api.udl") {
        eprintln!("error: UniFFI scaffolding failed for src/api.udl");
        eprintln!("  {e}");
        process::exit(1);
    }
}
