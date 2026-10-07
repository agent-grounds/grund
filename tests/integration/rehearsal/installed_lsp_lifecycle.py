"""§FS-distribution-candidate.5.5, §FS-distribution-candidate.3.4, §FS-lsp.2.1 — every
installed `grund-lsp` of this row holds the protocol's lifecycle.

Each install — Cargo, npm, wheel and archive — is started on a copy of the
`json-report` tree, initialized, handed the document that cites a missing ID, asked
for a hover on its declaration, shut down and told to exit. Every byte it writes to
standard output must belong to a framed protocol message.

It runs only in the manual rehearsal lane, never in push or pull-request CI
(§AR-ci.6).
"""

import json
import queue
import shutil
import subprocess
import threading
import unittest

from rehearsal_support import CASES, acquire, installed, keep

DOCUMENT = "docs/functional-spec/FS-001-alpha.md"


def frame(message):
    body = json.dumps(message).encode()
    return b"Content-Length: %d\r\n\r\n%s" % (len(body), body)


def reader(stream, messages, raw):
    """Parse frames off stdout; anything that is not a frame is kept as raw bytes."""
    while True:
        header = b""
        while not header.endswith(b"\r\n\r\n"):
            byte = stream.read(1)
            if not byte:
                messages.put(None)
                return
            header += byte
        fields = dict(line.split(b": ", 1) for line in header.strip().split(b"\r\n") if b": " in line)
        if b"Content-Length" not in fields:
            raw.append(header)
            continue
        messages.put(json.loads(stream.read(int(fields[b"Content-Length"]))))


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        acquire()

    def test_every_install_holds_the_lifecycle(self):
        for install, binary in installed("grund-lsp").items():
            with self.subTest(install=install):
                self.lifecycle(binary)

    def lifecycle(self, binary):
        root = keep("grund-lsp-lifecycle-") / "répo-雪"
        shutil.copytree(CASES / "json-report" / "repo", root)
        document = root / DOCUMENT
        uri = document.as_uri()
        server = subprocess.Popen([str(binary)], cwd=root, stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        messages, raw = queue.Queue(), []
        threading.Thread(target=reader, args=(server.stdout, messages, raw), daemon=True).start()

        def send(message):
            server.stdin.write(frame({"jsonrpc": "2.0", **message}))
            server.stdin.flush()

        def answer(request_id):
            while True:
                message = messages.get(timeout=60)
                self.assertIsNotNone(message, "the server closed its output early")
                if message.get("id") == request_id:
                    return message
                seen.append(message)

        seen = []
        try:
            send({"id": 1, "method": "initialize",
                  "params": {"processId": None, "rootUri": root.as_uri(), "capabilities": {}}})
            self.assertIn("capabilities", answer(1)["result"])
            send({"method": "initialized", "params": {}})
            send({"method": "textDocument/didOpen", "params": {"textDocument": {
                "uri": uri, "languageId": "markdown", "version": 1,
                "text": document.read_text(encoding="utf-8")}}})
            send({"id": 2, "method": "textDocument/hover", "params": {
                "textDocument": {"uri": uri}, "position": {"line": 0, "character": 5}}})
            self.assertIsNotNone(answer(2).get("result"), "no hover on the declaration")
            send({"id": 3, "method": "shutdown", "params": None})
            self.assertIsNone(answer(3).get("result"))
            published = [d for m in seen if m.get("method") == "textDocument/publishDiagnostics"
                         and m["params"]["uri"].endswith("FS-001-alpha.md")
                         for d in m["params"]["diagnostics"]]
            self.assertIn("dangling", [d.get("code") for d in published])
            send({"method": "exit", "params": None})
            self.assertEqual(0, server.wait(timeout=60))
        finally:
            if server.poll() is None:
                server.kill()
            server.stdin.close()
            server.stderr.close()
        self.assertEqual([], raw, "bytes on standard output that are not protocol messages")


if __name__ == "__main__":
    unittest.main()
