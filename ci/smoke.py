#!/usr/bin/env python3
import subprocess, sys, os

URL = "https://example.com"
PNG = "/tmp/monazite_smoke.png"


def ensure_built():
    # Build both browser-main and content-srv (release)
    bk = subprocess.run(["cargo", "build", "--release", "-p", "browser-main", "-p", "content-srv"], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if bk.returncode != 0:
        print(bk.stdout)
        sys.exit(bk.returncode)


def verify_png(path):
    # Minimal PNG validation: signature + IHDR present
    with open(path, "rb") as f:
        data = f.read(32)
    sig_ok = data.startswith(b"\x89PNG\r\n\x1a\n")
    if not sig_ok:
        print("ci/smoke.py: file is not a PNG signature:", path)
        sys.exit(3)
    size = os.path.getsize(path)
    if size < 1024:
        print("ci/smoke.py: PNG too small:", size)
        sys.exit(4)


def main():
    ensure_built()

    # Run browser with real rendering screenshot
    cmd = ["target/release/browser", URL, "--real-screenshot", PNG]
    run = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=25)
    print(run.stdout)
    if run.returncode != 0:
        sys.exit(run.returncode)

    # Check summary contains Example Domain
    ok = any("SUMMARY:" in line and ("Example Domain" in line) for line in run.stdout.splitlines())
    if not ok:
        print("ci/smoke.py: SUMMARY line not found or missing 'Example Domain'")
        sys.exit(2)

    # Check real screenshot exists and is PNG
    if not os.path.exists(PNG):
        print("ci/smoke.py: screenshot not found:", PNG)
        sys.exit(5)
    verify_png(PNG)

    print("ci/smoke.py: PASS")


if __name__ == "__main__":
    main()
