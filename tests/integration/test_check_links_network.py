"""Real CLI coverage for §FS-non-goals.1.1 and §AR-ci.3.1.1.

Port the issue-403 triage reproducer's loopback assertions. The production
configuration is copied unchanged for delayed-response and permanent-timeout
evidence. Only the supplementary retry-count fixture scales timeout to 1s.
No subprocess result is mocked; a watchdog expiration is an infrastructure
error. CI installs lychee before the Ubuntu pre-commit Python hook runs these
tests; earlier Python-only matrix jobs may skip the network class without it.
"""

from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
LYCHEE = shutil.which("lychee")
SELF_BASE = "https://github.com/agent-grounds/grund/blob/main/docs/"
WATCHDOG = 240  # Greater than 5 * 30 + (1 + 2 + 4 + 8), with ample overhead.


class LinkGatePolicyTests(unittest.TestCase):
    def test_production_timeout_and_retry_policy_is_explicit(self):
        config = tomllib.loads((ROOT / "lychee.toml").read_text(encoding="utf-8"))
        self.assertEqual(
            {"timeout": 30, "max_retries": 4, "retry_wait_time": 1},
            {key: config.get(key) for key in ("timeout", "max_retries", "retry_wait_time")},
        )


class LoopbackGate:
    """Disposable real HTTP and Markdown fixture for §AR-ci.3.1.1."""

    def __init__(self, scaled_timeout=False):
        scratch_base = Path.home() / "ag/tmp"
        scratch_base.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="grund-link-gate-", dir=scratch_base)
        self.root = Path(self.temp.name)
        self.counts = Counter()
        self.successes = Counter()
        self.lock = threading.Lock()
        self.stop = threading.Event()
        config_text = (ROOT / "lychee.toml").read_text(encoding="utf-8")
        if scaled_timeout:
            # Preserve production retries/backoff/exclusions. Remove only the
            # whole-request timeout, then replace it in the fixture copy.
            config_text = "\n".join(
                line for line in config_text.splitlines()
                if not line.strip().startswith("timeout =")
            ) + "\ntimeout = 1\n"
        (self.root / "lychee.toml").write_text(config_text, encoding="utf-8")
        (self.root / "docs").mkdir()
        (self.root / "docs/guide.md").write_text("# Present\n", encoding="utf-8")
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                with fixture.lock:
                    fixture.counts[self.path] += 1
                    attempt = fixture.counts[self.path]
                if self.path == "/transient" and attempt <= 4:
                    fixture.stop.wait(21)
                if self.path == "/four_timeouts" and attempt <= 4:
                    fixture.stop.wait(2)
                if self.path == "/persistent_timeout":
                    fixture.stop.wait(300)
                code = 404 if self.path == "/missing" else 200
                body = b"<html><body><h1 id='present'>Present</h1></body></html>"
                try:
                    self.send_response(code)
                    self.send_header("Content-Type", "text/html")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                    if code == 200:
                        with fixture.lock:
                            fixture.successes[self.path] += 1
                except (BrokenPipeError, ConnectionResetError):
                    pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base = f"http://127.0.0.1:{self.server.server_port}"

    def close(self):
        self.stop.set()
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()
        self.temp.cleanup()

    def check(self, name, urls):
        document = self.root / f"{name}.md"
        document.write_text("".join(f"[link](<{url}>)\n" for url in urls), encoding="utf-8")
        started = time.monotonic()
        try:
            result = subprocess.run(
                [sys.executable, str(ROOT / "scripts/check_links.py"),
                 "--lychee", LYCHEE, document.name],
                cwd=self.root, capture_output=True, text=True, timeout=WATCHDOG,
            )
        except subprocess.TimeoutExpired as exc:
            raise RuntimeError(
                f"Infrastructure failure: {name} exceeded {WATCHDOG}s watchdog; "
                "no link rejection was observed"
            ) from exc
        output = result.stdout + result.stderr
        print(f"\nCASE {name}: wrapper exit={result.returncode}, "
              f"elapsed={time.monotonic() - started:.1f}s\n{output}", flush=True)
        return result.returncode, output


@unittest.skipUnless(LYCHEE, "Network fixture requires CI-pinned lychee 0.23.0 on PATH")
class LinkGateNetworkTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        version = subprocess.check_output([LYCHEE, "--version"], text=True, timeout=10).strip()
        if version != "lychee 0.23.0":
            raise RuntimeError(f"Infrastructure failure: expected lychee 0.23.0, found {version}")

    def fixture(self, **kwargs):
        gate = LoopbackGate(**kwargs)
        self.addCleanup(gate.close)
        return gate

    def assert_success(self, result):
        code, output = result
        self.assertEqual(0, code, output)

    def assert_rejected(self, result, diagnostic):
        code, output = result
        self.assertEqual(2, code, output)
        self.assertIn(diagnostic, output)

    def test_delayed_response_passes_and_permanent_timeout_is_rejected(self):
        gate = self.fixture()
        with ThreadPoolExecutor(max_workers=2) as executor:
            transient = executor.submit(
                gate.check, "transient", [gate.base + "/transient", SELF_BASE + "guide.md#present"]
            )
            permanent = executor.submit(
                gate.check, "persistent_timeout", [gate.base + "/persistent_timeout"]
            )
            transient_result, permanent_result = transient.result(), permanent.result()

        # Check retained rejection before the intentionally red assertion.
        self.assert_rejected(permanent_result, "[TIMEOUT]")
        self.assertGreater(gate.counts["/persistent_timeout"], 0)
        with self.subTest("21-second response headers must be tolerated"):
            self.assert_success(transient_result)
            self.assertEqual(1, gate.counts["/transient"])
            self.assertGreater(gate.successes["/transient"], 0)
        with self.subTest("persistent timeout exhausts exactly five attempts"):
            self.assertEqual(5, gate.counts["/persistent_timeout"])

        # The original triage control: an unchanged rerun can succeed once
        # the four delayed attempts have been exhausted on the old policy.
        recovered = gate.check("recovered", [gate.base + "/transient"])
        self.assert_success(recovered)

    def test_recovers_after_four_retryable_timeouts(self):
        gate = self.fixture(scaled_timeout=True)
        result = gate.check("four_timeouts", [gate.base + "/four_timeouts"])
        self.assert_success(result)
        self.assertEqual(5, gate.counts["/four_timeouts"])
        self.assertGreater(gate.successes["/four_timeouts"], 0)

    def test_http_404_is_rejected(self):
        gate = self.fixture()
        self.assert_rejected(gate.check("persistent_404", [gate.base + "/missing"]), "[404]")
        self.assertEqual(1, gate.counts["/missing"])

    def test_valid_self_fragment_and_http_200_pass(self):
        gate = self.fixture()
        self.assert_success(
            gate.check("ready", [gate.base + "/ready", SELF_BASE + "guide.md#present"])
        )
        self.assertEqual(1, gate.counts["/ready"])
        self.assertGreater(gate.successes["/ready"], 0)

    def test_missing_self_target_stops_before_network(self):
        gate = self.fixture()
        self.assert_rejected(
            gate.check("missing_self_target", [SELF_BASE + "absent.md", gate.base + "/ready"]),
            "target is missing",
        )
        self.assertEqual(0, gate.counts["/ready"])

    def test_bad_self_fragment_stops_before_network(self):
        gate = self.fixture()
        self.assert_rejected(
            gate.check("bad_self_fragment", [SELF_BASE + "guide.md#absent", gate.base + "/ready"]),
            "Cannot find fragment",
        )
        self.assertEqual(0, gate.counts["/ready"])


if __name__ == "__main__":
    unittest.main()
