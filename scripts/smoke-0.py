#!/usr/bin/env python3
import os
import sys
import time
import tempfile
import subprocess

UDS = os.path.join(tempfile.gettempdir(), "monazite-p0.sock")


def main():
    # Optional: build to ensure binaries exist
    subprocess.run(["cargo", "build", "-q"], check=True)

    # Start host (server)
    if os.path.exists(UDS):
        os.unlink(UDS)
    host = subprocess.Popen([
        os.path.join("target", "debug", "gpui-app-host"),
        "--uds", UDS,
    ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    time.sleep(0.1)

    # Start client (web process)
    client = subprocess.run([
        os.path.join("target", "debug", "min-web-process"),
        "--headless", "--uds", UDS,
    ], capture_output=True, text=True)

    if client.returncode != 0:
        print("smoke-0: client failed", file=sys.stderr)
        print(client.stdout)
        print(client.stderr, file=sys.stderr)
        host.kill()
        sys.exit(2)

    # Collect host output up to 10s
    ok = False
    screenshot_ok = False
    quit_seen = False
    deadline = time.time() + 10
    while time.time() < deadline:
        line = host.stdout.readline()
        if not line:
            if host.poll() is not None:
                break
            time.sleep(0.05)
            continue
        line = line.strip()
        print(line)
        if line.startswith("FRAME") and line.endswith("OK"):
            ok = True
        if line == "SCREENSHOT OK":
            screenshot_ok = True
        if line == "QUIT":
            quit_seen = True
            break

    host.wait(timeout=2)

    if ok and screenshot_ok and quit_seen:
        print("smoke-0 PASS")
        sys.exit(0)
    else:
        print("smoke-0 FAIL", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
