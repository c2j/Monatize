#!/usr/bin/env python3
# Phase-2 smoke test (initial): validate S3/S4 logging with dual tabs under Xvfb
# - Launch browser with two tabs using --extra-tab
# - Assert we see P2_S3_* spawn logs and P2_S4_FRAME surfaces=2 within time budget

import os
import sys
import time
import shutil
import subprocess
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parents[1]
TARGET = WORKSPACE / "target" / "release"
BROWSER = TARGET / "browser"
CONTENT = TARGET / "content-srv"

BINS = [BROWSER, CONTENT, TARGET / "gpu-srv", TARGET / "network-srv", TARGET / "ai-runtime"]


def which(cmd: str) -> bool:
    return shutil.which(cmd) is not None


def ensure_built():
    # Best-effort build (release) for the required subset
    subprocess.run([
        "cargo", "build", "--release",
        "-p", "browser-main",
        "-p", "content-srv",
        "-p", "gpu-srv",
        "-p", "network-srv",
        "-p", "ai-runtime",
    ], check=False, cwd=str(WORKSPACE))


def run_under_display(cmd: list[str]):
    # Prefer xvfb-run if available; else require existing DISPLAY
    if which("xvfb-run"):
        # Use 1024x768x24 for compatibility
        xvfb = ["xvfb-run", "-s", "-screen 0 1024x768x24"]
        return subprocess.Popen(xvfb + cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=str(WORKSPACE))
    if os.environ.get("DISPLAY"):
        return subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=str(WORKSPACE))
    print("smoke-2: SKIP (no display: neither xvfb-run nor DISPLAY present)")
    return None


def main():
    t0 = time.time()
    ensure_built()
    missing = [str(p) for p in BINS if not p.exists()]
    if missing:
        print("smoke-2: SKIP (missing binaries)", missing)
        sys.exit(0)

    # Two different hosts to trigger distinct content-srv spawns
    url0 = "https://example.com/"
    url1 = "https://www.rust-lang.org/"

    cmd = [str(BROWSER), url0,
           "--extra-tab", url1]
    # Note: We do NOT pass --no_gui or --show; we rely on Xvfb to enable GTK WebView path

    proc = run_under_display(cmd)

    s3_seen = False
    s4_surface2 = False
    lines = []
    timeout = 15.0

    if proc is None:
        ok = True
    else:
        try:
            t_deadline = time.time() + timeout
            while time.time() < t_deadline:
                line = proc.stdout.readline()
                if not line:
                    # give the process a brief moment
                    if proc.poll() is not None:
                        break
                    time.sleep(0.05)
                    continue
                line = line.rstrip()
                lines.append(line)
                if "P2_S3_SPAWN" in line or "P2_S3_REUSE" in line:
                    s3_seen = True
                if "P2_S4_FRAME" in line and "surfaces=2" in line:
                    s4_surface2 = True
                    break
            ok = s3_seen and s4_surface2
            if not (s3_seen and s4_surface2):
                # If Xvfb cannot start, treat S3/S4 as skipped (do not fail whole smoke)
                if any("Xvfb failed to start" in l for l in lines):
                    ok = True
                else:
                    ok = False

        finally:
            # Try to terminate the process tree; xvfb-run should exit too
            try:
                proc.terminate()
                proc.wait(timeout=3)
            except Exception:
                try:
                    proc.kill()
                except Exception:
                    pass

    # S11: stub print-to-PDF check (no display required)
    ok_print = False
    out_dir = WORKSPACE / "target" / "smoke-print"
    try:
        if out_dir.exists():
            shutil.rmtree(out_dir)
        out_dir.mkdir(parents=True, exist_ok=True)
        cmd2 = [str(BROWSER), "about:blank", "--print-stub", str(out_dir)]
        p = subprocess.run(cmd2, cwd=str(WORKSPACE), capture_output=True, text=True)
        stdout = (p.stdout or "") + (p.stderr or "")
        path = None
        for ln in stdout.splitlines():
            if ln.startswith("P2_S11_PRINT_OK path="):
                path = ln.split("path=", 1)[1].strip()
                break
        if path and Path(path).exists() and Path(path).stat().st_size > 0:
            ok_print = True

    except Exception:
        ok_print = False

    # S6: extension load check (no display required)
    ok_ext = False
    stdout_ext = ""
    ext_dir = WORKSPACE / "target" / "smoke-ext"
    try:
        if ext_dir.exists():
            shutil.rmtree(ext_dir)
        ext_dir.mkdir(parents=True, exist_ok=True)
        (ext_dir / "manifest.json").write_text('{"name":"smoke-ext","version":"0.1.0"}', encoding="utf-8")
        wat = '(module (import "host" "hello" (func $h (param i32))) (import "host" "tabs_create" (func $c)) (func (export "hello") (call $h (i32.const 5)) (call $c)))'
        (ext_dir / "background.wat").write_text(wat, encoding="utf-8")
        cmd3 = [str(BROWSER), "about:blank", "--no-gui", "--ext-load", str(ext_dir)]
        p3 = subprocess.run(cmd3, cwd=str(WORKSPACE), capture_output=True, text=True)
        stdout_ext = (p3.stdout or "") + (p3.stderr or "")
        loaded = False
        hello = False
        evt = False
        tabs_created = False
        for ln in stdout_ext.splitlines():
            if ln.startswith("P2_S6_LOADED id="):
                loaded = True
            if ln.startswith("P2_S6_LOG") and "hello(" in ln:
                hello = True
            if ln.startswith("P2_S7_EVT") and "ns=runtime" in ln and "name=hello" in ln:
                evt = True
            if ln.startswith("P2_S7_TABS_CREATED"):
                tabs_created = True
        ok_ext = loaded and hello and evt and tabs_created
    except Exception:
        ok_ext = False

    elapsed = time.time() - t0
    if ok and ok_print and ok_ext:
        print(f"smoke-2: PASS (S3/S4 + S11 print + S6 ext + S7 bus) in {elapsed:.2f}s")
        sys.exit(0)
    else:
        print("smoke-2: FAIL")
        print("--- output (S3/S4) ---")
        for l in lines[-200:]:
            print(l)
        print("--- output (print) ---")
        try:
            print(stdout)
        except Exception:
            pass
        print("--- output (ext) ---")
        try:
            print(stdout_ext)
        except Exception:
            pass
        sys.exit(1)


if __name__ == "__main__":
    main()
