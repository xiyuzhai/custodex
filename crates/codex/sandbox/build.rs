use std::env;
use std::path::PathBuf;
use std::process::Command;

const CODEX_GIT: &str = "https://github.com/xiyuzhai/codex";
const CODEX_REV: &str = "4fd5c35c4f6f51048f47c8680ed0f6a26c608f68";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_root = out_dir.join("sandbox-install");
    let bin_path = install_root.join("bin").join("codex-linux-sandbox");

    println!("cargo:rerun-if-changed=build.rs");

    // Skip rebuild if binary already exists. cargo install --force would
    // also re-check, but caching this avoids the install command entirely
    // on incremental builds.
    if bin_path.exists() {
        println!("cargo:rustc-env=CODEX_SANDBOX_EXE={}", bin_path.display());
        return;
    }

    let status = Command::new("cargo")
        .args([
            "install",
            "--git",
            CODEX_GIT,
            "--rev",
            CODEX_REV,
            "codex-linux-sandbox",
            "--locked",
            "--root",
        ])
        .arg(&install_root)
        .arg("--force")
        .status()
        .expect("failed to spawn cargo install for codex-linux-sandbox");

    assert!(status.success(), "cargo install codex-linux-sandbox failed");
    assert!(
        bin_path.exists(),
        "expected sandbox binary at {}",
        bin_path.display()
    );

    println!("cargo:rustc-env=CODEX_SANDBOX_EXE={}", bin_path.display());
}
