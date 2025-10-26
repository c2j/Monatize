use std::env;
use std::process::{Command, Stdio};

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("sys-deps") => sys_deps(),
        Some("cmake-gen") => {
            println!("xtask: cmake-gen (stub)");
        }
        Some("dist-0") => dist_0(),
        Some("dist-gtk") => dist_gtk(),
        Some("fmt-lint") => fmt_lint(),
        Some("test") => test_all(),
        Some("dist") => dist_p1(),
        Some("dist-2") => dist_2(),
        Some("demo") => demo(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("xtask commands:");
    println!("  sys-deps      - probe system dependencies (non-fatal)");
    println!("  cmake-gen     - generate build scaffolding (stub)");
    println!("  dist-0        - package default binaries (headless demo)");
    println!("  dist-gtk      - package with WebKitGTK-enabled min-web-process");
    println!("  fmt-lint      - run cargo fmt + clippy over workspace");
    println!("  test          - run cargo test --workspace");
    println!("  dist          - package Phase-1 binaries (stub for now)");
    println!("  dist-2        - package Phase-2 (initial stub: same as dist with extras)");
    println!("  demo [mode]   - one-click demo under Xvfb; mode = headless (default) | gtk");
}


fn fmt_lint() {
    let status_fmt = Command::new("cargo").args(["fmt", "--all"]).status().expect("run cargo fmt");
    if !status_fmt.success() { std::process::exit(1); }
    let status_clippy = Command::new("cargo")
        .args(["clippy", "--workspace", "--all-targets", "-A", "clippy::all"]) // do not fail CI yet
        .status()
        .expect("run cargo clippy");
    if !status_clippy.success() { eprintln!("xtask: clippy reported issues (non-fatal)"); }
}

fn test_all() {
    let status = Command::new("cargo").args(["test", "--workspace"]).status().expect("run cargo test");
    if !status.success() { std::process::exit(1); }
}

fn dist_p1() {
    use std::fs;
    use std::path::PathBuf;

    // Build release binaries for Phase-1 processes
    let status = Command::new("cargo")
        .args([
            "build", "--release",
            "-p", "network-srv",
            "-p", "gpu-srv",
            "-p", "content-srv",
            "-p", "ai-runtime",
            "-p", "browser-main",
        ])
        .status()
        .expect("failed to run cargo build --release");
    if !status.success() {
        eprintln!("xtask: cargo build --release failed");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from("dist/phase-1");
    fs::create_dir_all(&out_dir).expect("mkdir dist/phase-1");

    let net_bin = PathBuf::from("target/release/network-srv");
    let gpu_bin = PathBuf::from("target/release/gpu-srv");
    let content_bin = PathBuf::from("target/release/content-srv");
    let ai_bin = PathBuf::from("target/release/ai-runtime");
    let browser_bin = PathBuf::from("target/release/browser");

    fs::copy(&net_bin, out_dir.join("network-srv")).expect("copy network-srv");
    fs::copy(&gpu_bin, out_dir.join("gpu-srv")).expect("copy gpu-srv");
    fs::copy(&content_bin, out_dir.join("content-srv")).expect("copy content-srv");
    fs::copy(&ai_bin, out_dir.join("ai-runtime")).expect("copy ai-runtime");
    fs::copy(&browser_bin, out_dir.join("browser")).expect("copy browser");

    // Copy smoke script
    let _ = fs::create_dir_all(&out_dir);
    let _ = fs::copy("ci/smoke.py", out_dir.join("smoke.py"));

    // Optional: write sha256 checksums if sha256sum is available
    if check("sha256sum") {
        let _ = Command::new("sh").arg("-c").arg("(cd dist/phase-1 && sha256sum * > SHA256SUMS)").status();
    }

    println!("xtask: dist (Phase-1) ready at {}/", out_dir.display());
}

fn dist_2() {
    use std::fs;
    use std::path::PathBuf;

    // Build release binaries for Phase-2 (initial subset)
    let status = Command::new("cargo")
        .args([
            "build", "--release",
            "-p", "network-srv",
            "-p", "gpu-srv",
            "-p", "content-srv",
            "-p", "ai-runtime",
            "-p", "browser-main",
            "-p", "update-service",
            "-p", "crash-reporter",
        ])
        .status()
        .expect("failed to run cargo build --release");
    if !status.success() {
        eprintln!("xtask: cargo build --release failed");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from("dist/phase-2");
    fs::create_dir_all(&out_dir).expect("mkdir dist/phase-2");

    for (src, dst) in [
        ("target/release/network-srv", "network-srv"),
        ("target/release/gpu-srv", "gpu-srv"),
        ("target/release/content-srv", "content-srv"),
        ("target/release/ai-runtime", "ai-runtime"),
        ("target/release/browser", "browser"),
        ("target/release/updater", "updater"),
        ("target/release/crash-reporter", "crash-reporter"),
    ] {
        let _ = fs::copy(src, out_dir.join(dst));
    }

    // Drop in smoke-2 skeleton if present
    if std::path::Path::new("ci/smoke-2.py").exists() {
        let _ = fs::copy("ci/smoke-2.py", out_dir.join("smoke-2.py"));
    }

    println!("xtask: dist-2 (stub) ready at {}/", out_dir.display());
}

fn check(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", cmd))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sys_deps() {
    let tools = ["pkg-config", "cmake", "python3", "xvfb-run"];
    let mut missing: Vec<&str> = Vec::new();
    for t in tools {
        let ok = check(t);
        if ok {
            println!("{t:<10}: OK");
        } else {
            println!("{t:<10}: MISSING (non-fatal)");
            missing.push(t);
        }
    }

    if !missing.is_empty() {
        println!("\nxtask: Some tools are missing. You can install them using your package manager. Examples:");
        if check("apt-get") {
            println!("  sudo apt-get update && sudo apt-get install -y pkg-config cmake python3 xvfb");
        }
        if check("dnf") {
            println!("  sudo dnf install -y pkg-config cmake python3 xorg-x11-server-Xvfb");
        }
        if check("pacman") {
            println!("  sudo pacman -S --needed pkgconf cmake python xorg-server-xvfb");
        }
        println!("This is only a hint; build will proceed if possible.");
    }

    // Additional system packages for WebKitGTK development (P3)
    println!("\nWebKitGTK dev packages (for P3):");
    if check("apt-get") {
        println!("  sudo apt-get install -y libwebkit2gtk-4.0-dev libgtk-3-dev");
    }
    if check("dnf") {
        println!("  sudo dnf install -y webkit2gtk3-devel gtk3-devel");
    }
    if check("pacman") {
        println!("  sudo pacman -S --needed webkit2gtk gtk3");
    }
}

fn dist_0() {
    use std::fs;
    use std::path::PathBuf;

    // Build release binaries for the two crates
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "gpui-app-host", "-p", "min-web-process"])
        .status()
        .expect("failed to run cargo build --release");
    if !status.success() {
        eprintln!("xtask: cargo build --release failed");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from("dist/phase-0");
    fs::create_dir_all(&out_dir).expect("mkdir dist/phase-0");

    let host_bin = PathBuf::from("target/release/gpui-app-host");
    let webp_bin = PathBuf::from("target/release/min-web-process");
    let smoke = PathBuf::from("scripts/smoke-0.py");

    fs::copy(&host_bin, out_dir.join("gpui-app-host")).expect("copy host");
    fs::copy(&webp_bin, out_dir.join("min-web-process")).expect("copy webp");
    fs::copy(&smoke, out_dir.join("smoke-0.py")).expect("copy smoke-0");

    println!("xtask: dist-0 ready at {}/", out_dir.display());
}

fn dist_gtk() {
    use std::fs;
    use std::path::PathBuf;

    // Build host normally, build min-web-process with GTK feature
    let status_host = Command::new("cargo")
        .args(["build", "--release", "-p", "gpui-app-host"])
        .status()
        .expect("failed to run cargo build --release for host");
    if !status_host.success() {
        eprintln!("xtask: cargo build --release (host) failed");
        std::process::exit(1);
    }

    let status_webp = Command::new("cargo")
        .args(["build", "--release", "-p", "min-web-process", "--no-default-features", "--features", "port_gtk"])
        .status()
        .expect("failed to run cargo build --release for min-web-process (port_gtk)");
    if !status_webp.success() {
        eprintln!("xtask: cargo build --release (min-web-process port_gtk) failed");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from("dist/phase-0-gtk");
    fs::create_dir_all(&out_dir).expect("mkdir dist/phase-0-gtk");

    let host_bin = PathBuf::from("target/release/gpui-app-host");
    let webp_bin = PathBuf::from("target/release/min-web-process");
    let smoke = PathBuf::from("scripts/smoke-0.py");

    fs::copy(&host_bin, out_dir.join("gpui-app-host")).expect("copy host");
    fs::copy(&webp_bin, out_dir.join("min-web-process")).expect("copy webp");
    fs::copy(&smoke, out_dir.join("smoke-0.py")).expect("copy smoke-0");

    println!("xtask: dist-gtk ready at {}/", out_dir.display());
}


fn demo() {
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    // Parse mode from args (second arg after "demo"): headless (default) | gtk
    let mut args = env::args().skip(2);
    let mode = args.next().unwrap_or_else(|| "headless".to_string());

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../");
    let xvfb = check("xvfb-run");
    let py = check("python3");

    if mode == "headless" {
        // Build debug binaries quickly
        let status = Command::new("cargo")
            .current_dir(&workspace)
            .args(["build", "-q"]) // quiet build
            .status()
            .expect("failed to run cargo build");
        if !status.success() { eprintln!("xtask: cargo build failed"); std::process::exit(1); }

        let mut cmd = if xvfb {
            let mut c = Command::new("xvfb-run");
            c.current_dir(&workspace)
                .args(["-s", "-screen 0 800x600x24", "python3", "scripts/smoke-0.py"]);
            c
        } else if py {
            let mut c = Command::new("python3");
            c.current_dir(&workspace).arg("scripts/smoke-0.py");
            c
        } else {
            eprintln!("xtask: neither xvfb-run nor python3 found; cannot run demo");
            std::process::exit(2);
        };
        let status = cmd.status().expect("failed to run demo");
        if !status.success() { std::process::exit(status.code().unwrap_or(1)); }
        return;
    }

    if mode == "gtk" {
        if !xvfb { eprintln!("xtask: xvfb-run not found; please install it (see `xtask sys-deps`)"); std::process::exit(2); }

        // Build host normally, web-process with GTK feature
        let status_host = Command::new("cargo")
            .current_dir(&workspace)
            .args(["build", "-p", "gpui-app-host"]) // debug is fine for demo
            .status()
            .expect("failed to run cargo build for host");
        if !status_host.success() { eprintln!("xtask: host build failed"); std::process::exit(1); }

        let status_webp = Command::new("cargo")
            .current_dir(&workspace)
            .args(["build", "-p", "min-web-process", "--no-default-features", "--features", "port_gtk"])
            .status()
            .expect("failed to run cargo build for min-web-process (port_gtk)");
        if !status_webp.success() { eprintln!("xtask: min-web-process (port_gtk) build failed"); std::process::exit(1); }

        // Prepare UDS
        let uds = "/tmp/monazite-demo-gtk.sock";
        let _ = fs::remove_file(uds);

        // Start host under Xvfb (inherit stdio to show logs)
        let mut host = Command::new("xvfb-run")
            .current_dir(&workspace)
            .args(["-s", "-screen 0 800x600x24", "target/debug/gpui-app-host", "--uds", uds])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to launch gpui-app-host under xvfb-run");

        thread::sleep(Duration::from_millis(300));

        // Launch GTK-rendering min-web-process (no --headless)
        let status_client = Command::new("target/debug/min-web-process")
            .current_dir(&workspace)
            .args(["--uds", uds])
            .status()
            .expect("failed to run min-web-process (gtk)");
        if !status_client.success() {
            let _ = host.kill();
            std::process::exit(status_client.code().unwrap_or(1));
        }

        // Wait for host to finish
        let _ = host.wait();
        return;
    }

    eprintln!("xtask: unknown demo mode: {mode}. Use `demo` or `demo gtk`");
    std::process::exit(2);
}


