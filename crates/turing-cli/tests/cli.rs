//! End-to-end tests of the `tm` binary.
//!
//! The snapshot cases live in `tests/cmd`: each `*.toml` names the arguments and the working
//! directory (the repository root, so that the gallery under `machines/` is found), and the
//! `*.stdout` and `*.stderr` files next to it hold the expected output. The usage examples in
//! the crate README are checked the same way. Regenerate snapshots with
//! `TRYCMD=overwrite cargo test -p turing-cli --test cli` and review the diff.

use std::path::{Path, PathBuf};
use std::process::Command;

fn tm() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_tm"))
}

#[test]
fn snapshots() {
    trycmd::TestCases::new()
        .register_bin("tm", tm())
        .env("TM_NO_TIMING", "1")
        .env("NO_COLOR", "1")
        .case("tests/cmd/*.toml")
        .case("README.md");
}

/// A fresh directory for files written by a test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tm-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn render_writes_png_and_svg() {
    let dir = scratch("render");
    let png = dir.join("bb3.png");
    let svg = dir.join("bb3.svg");
    let output = Command::new(tm())
        .args(["render", "--standard", "1RB1RZ_0RC1RB_1LC1LA", "--spacetime"])
        .arg(&png)
        .args(["--width", "64", "--height", "48", "--exact", "0", "--diagram"])
        .arg(&svg)
        .env("TM_NO_TIMING", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "tm render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&png).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "not a PNG file");
    // The IHDR chunk follows the signature: length, type, width, height.
    assert_eq!(&bytes[12..16], b"IHDR");
    assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 64);
    assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 48);
    let svg = std::fs::read_to_string(&svg).unwrap();
    assert!(svg.trim_start().starts_with("<svg") || svg.trim_start().starts_with("<?xml"));
    assert!(svg.contains("</svg>"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn render_dark_overview() {
    let dir = scratch("overview");
    let png = dir.join("bb4.png");
    let output = Command::new(tm())
        .args([
            "render",
            "--standard",
            "1RB1LB_1LA0LC_1RZ1LD_1RD0RA",
            "--theme",
            "dark",
            "--width",
            "100",
            "--height",
            "80",
            "--overview",
            "1e4",
            "--spacetime",
        ])
        .arg(&png)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "tm render failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&png).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_write_rewrites_in_place() {
    let dir = scratch("fmt");
    let file = dir.join("walk.tm");
    std::fs::write(&file, "δ(q0,x)=(q0,X,R)\n").unwrap();
    let status = Command::new(tm())
        .args(["fmt", "--write"])
        .arg(&file)
        .status()
        .unwrap();
    assert!(status.success());
    let formatted = std::fs::read_to_string(&file).unwrap();
    let check = Command::new(tm())
        .args(["fmt", "--check"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "not formatted after --write:\n{formatted}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn colours_follow_the_color_flag() {
    let run = |color: &str| {
        Command::new(tm())
            .args(["--color", color, "decide", "1RB1LB_1LA1RZ"])
            .env_remove("NO_COLOR")
            .env("TM_NO_TIMING", "1")
            .output()
            .unwrap()
    };
    let always = run("always");
    let never = run("never");
    assert!(always.status.success() && never.status.success());
    assert!(always.stdout.contains(&0x1b), "no escape codes with --color always");
    assert!(!never.stdout.contains(&0x1b), "escape codes with --color never");
}

#[test]
fn closed_pipe_is_not_an_error() {
    use std::io::Read;
    use std::process::Stdio;
    let mut child = Command::new(tm())
        .args(["enumerate", "--states", "2", "--holdouts"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Read one byte and close the pipe.
    let mut stdout = child.stdout.take().unwrap();
    let mut byte = [0u8; 1];
    let _ = stdout.read(&mut byte);
    drop(stdout);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.code().is_some_and(|c| c == 0));
    assert!(output.stderr.is_empty());
}
