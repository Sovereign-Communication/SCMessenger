use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn emit_git_watch_paths() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let git_entry = manifest_dir
        .parent()
        .map(|parent| parent.join(".git"))
        .unwrap_or_else(|| PathBuf::from(".git"));
    println!("cargo:rerun-if-changed={}", git_entry.display());

    let git_dir = if git_entry.is_file() {
        fs::read_to_string(&git_entry)
            .ok()
            .and_then(|contents| {
                contents.lines().find_map(|line| {
                    line.strip_prefix("gitdir:")
                        .map(str::trim)
                        .filter(|path| !path.is_empty())
                        .map(str::to_owned)
                })
            })
            .map(|path| {
                let path = PathBuf::from(path);
                if path.is_absolute() {
                    path
                } else {
                    git_entry
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(path)
                }
            })
            .unwrap_or_else(|| git_entry.clone())
    } else {
        git_entry.clone()
    };

    let head_path = git_dir.join("HEAD");
    println!("cargo:rerun-if-changed={}", head_path.display());
    let Ok(head) = fs::read_to_string(&head_path) else {
        return;
    };

    let common_dir = fs::read_to_string(git_dir.join("commondir"))
        .ok()
        .map(|path| {
            let path = PathBuf::from(path.trim());
            if path.is_absolute() {
                path
            } else {
                git_dir.join(path)
            }
        })
        .unwrap_or_else(|| git_dir.clone());

    if let Some(reference) = head.strip_prefix("ref:").map(str::trim) {
        println!(
            "cargo:rerun-if-changed={}",
            common_dir.join(reference).display()
        );
        if common_dir != git_dir {
            println!(
                "cargo:rerun-if-changed={}",
                git_dir.join(reference).display()
            );
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=SCM_GIT_HASH");
    println!("cargo:rerun-if-env-changed=SCM_BUILD_TIME");
    emit_git_watch_paths();

    let git_hash = std::env::var("SCM_GIT_HASH")
        .ok()
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let build_time =
        std::env::var("SCM_BUILD_TIME").unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    println!("cargo:rustc-env=SCM_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=SCM_BUILD_TIME={}", build_time);
}
