"""A git fixture tree and a loopback host that counts every request, for the
changed-line gate of §AR-ci.3.1.4 and the sweep of §AR-ci.3.1.5.

The host answers 404 under `/dead*` and 200 elsewhere, so a dead link fails at
once, with no retry budget spent, and the request counts say exactly which
external URLs the wrapper sent to the network.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import threading
from collections import Counter
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check_links.py"
LYCHEE = shutil.which("lychee")
WATCHDOG = 300

# The host's git config never reaches the fixture, so a contract test cannot pass or fail on
# this machine's settings; a test that pins a setting passes it through `git_config` (§AR-ci.1.1).
ISOLATED = {"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}


def isolated_env(git_config: str | None = None) -> dict[str, str]:
    env = {k: v for k, v in os.environ.items()
           if k not in ("LINK_BASE", "GIT_CONFIG_PARAMETERS", "GIT_CONFIG_COUNT")}
    env.update(ISOLATED)
    if git_config is not None:
        env["GIT_CONFIG_PARAMETERS"] = git_config
    return env


class LinkFixture:
    """A repository whose `main` is `origin/main` and whose `HEAD` is a branch on it."""

    def __init__(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="grund-link-scope-", dir=scratch)
        self.root = Path(self.temp.name)
        self.counts: Counter[str] = Counter()
        self.lock = threading.Lock()
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                with fixture.lock:
                    fixture.counts[self.path] += 1
                code = 404 if self.path.startswith("/dead") else 200
                body = b"<html><body><h1 id='present'>Present</h1></body></html>"
                try:
                    self.send_response(code)
                    self.send_header("Content-Type", "text/html")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError):
                    pass

            do_HEAD = do_GET

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base = f"http://127.0.0.1:{self.server.server_port}"
        (self.root / "lychee.toml").write_text(
            (ROOT / "lychee.toml").read_text(encoding="utf-8"), encoding="utf-8"
        )

    def close(self):
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()
        self.temp.cleanup()

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", *args], cwd=self.root, env=isolated_env(), check=True,
            capture_output=True, text=True,
        ).stdout.strip()

    def write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text.replace("{host}", self.base), encoding="utf-8")

    def append(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.write_text(
            path.read_text(encoding="utf-8") + text.replace("{host}", self.base), encoding="utf-8"
        )

    def commit_main(self, files: dict[str, str]) -> str:
        """Commit `files` as `main`, point `origin/main` at it, and branch off it."""
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        self.git("config", "commit.gpgsign", "false")
        for relative, text in files.items():
            self.write(relative, text)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "main")
        self.git("update-ref", "refs/remotes/origin/main", "HEAD")
        self.git("checkout", "-q", "-b", "feature")
        return self.git("rev-parse", "HEAD")

    def commit(self, message: str = "change") -> None:
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)

    def requested(self) -> dict[str, int]:
        with self.lock:
            return dict(self.counts)

    def reset(self) -> None:
        with self.lock:
            self.counts.clear()

    def run(
        self, *args: str, link_base: str | None = None, git_config: str | None = None
    ) -> tuple[int, str]:
        env = isolated_env(git_config)
        if link_base is not None:
            env["LINK_BASE"] = link_base
        try:
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--lychee", LYCHEE, *args],
                cwd=self.root, env=env, capture_output=True, text=True, timeout=WATCHDOG,
            )
        except subprocess.TimeoutExpired as exc:
            raise RuntimeError(f"Infrastructure failure: wrapper exceeded {WATCHDOG}s") from exc
        return result.returncode, result.stdout + result.stderr

    def recording_lychee(self) -> Path:
        """A lychee that never reaches the network, written outside the inputs, and the log it
        keeps: local calls go to the real lychee, every other URL is recorded and answered
        200, so `sent()` names exactly what the gate would have requested (§AR-ci-self-links.1)."""
        shim = self.root / "lychee-recording"
        self.network_log = self.root / "lychee-network.log"
        self.network_log.write_text("", encoding="utf-8")
        shim.write_text(
            RECORDING_LYCHEE.format(
                python=sys.executable, real=LYCHEE, log=str(self.network_log)
            ),
            encoding="utf-8",
        )
        shim.chmod(0o755)
        return shim

    def sent(self) -> list[str]:
        return self.network_log.read_text(encoding="utf-8").splitlines()


# A dump, an `--offline` pass and a stdin document of `file://` self-links are local by
# construction; any other call is one the gate sends to the network.
RECORDING_LYCHEE = """#!{python}
import os, re, subprocess, sys
real, log, args = {real!r}, {log!r}, sys.argv[1:]
if "--dump" in args or "--offline" in args:
    os.execv(real, [real, *args])
urls = []
if "-" in args:
    document = sys.stdin.read()
    urls = re.findall(r"\\(<([^>]*)>\\)", document)
    if urls and all(url.startswith("file://") for url in urls):
        sys.exit(subprocess.run([real, *args], input=document, text=True).returncode)
else:
    urls = ["PATHS " + " ".join(args)]
with open(log, "a", encoding="utf-8") as out:
    out.writelines(url + "\\n" for url in urls)
"""
