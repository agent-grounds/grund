"""The PGO training workload of every product.

§FS-distribution-candidate.7.1: each product trains on its own workload, so a profile
describes the code that product runs. §AR-benchmarks.1.5: the `grund` executable runs the
scenario catalog's `pgo` subset against this repository's tree, and the Node addon and the
Python extension run the same subset through native API calls; this is the catalog's one
PGO reader. The language server is driven through an editor session over stdio.
`scripts/pgo-build.sh` runs this with the instrumented payload, and reads `describe` for
the evidence it records.

Plain Python 3.6 syntax and the standard library only, because `pgo-build.sh`'s
`find_python` accepts any Python 3.6 or newer to run it; in the manylinux2014 image the
Linux release builds in, that is `/opt/python/cp312-cp312/bin/python3`.
"""

import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time

CATALOG = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "crates", "grund-cli", "benches", "scenarios.json")
LSP = ["initialize", "initialized", "textDocument/didOpen README.md",
       "textDocument/didOpen docs/functional-spec/FS-check.md",
       "textDocument/didOpen crates/grund-core/src/lib.rs",
       "textDocument/hover on the first citation of each document, answered after its diagnostics",
       "shutdown", "exit"]
# `show`'s depth flags, as the native APIs name the mode (§AR-benchmarks.1.2).
MODES = {"--brief": "brief", "--full": "full"}


def subset():
    """§AR-benchmarks.1.5: the catalog's `pgo` rows, in catalog order."""
    with open(CATALOG, encoding="utf-8") as handle:
        return [row for row in json.load(handle)["scenarios"] if row["pgo"]]


def arguments(row):
    """A row's `args` with `{id}` this repository's subject rather than the fixture's."""
    return [arg.replace("{id}", row["pgo"].get("id", "")) for arg in row["args"]]


def steps():
    """Each `pgo` row as the native call that runs it: the operation, its ID, its mode."""
    return [{"op": row["args"][0], "id": row["pgo"].get("id"),
             "mode": next((MODES[arg] for arg in row["args"] if arg in MODES), "lead")}
            for row in subset()]


def workload(product):
    """The training `describe` prints, which the release evidence records byte for byte."""
    if product == "grund":
        return ["grund " + " ".join(arguments(row)) for row in subset()]
    if product == "grund-lsp":
        return ["grund-lsp " + step for step in LSP]
    prefix = {"node-addon": "node ", "python-extension": "python "}[product]
    return [prefix + row["pgo"]["api"] for row in subset()]


DOCUMENTS = ("README.md", "docs/functional-spec/FS-check.md", "crates/grund-core/src/lib.rs")


class Session:
    """A minimal LSP client: framed JSON-RPC over the server's stdio."""

    def __init__(self, command):
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL)
        self.messages, self.lock, self.next_id = [], threading.Condition(), 0
        threading.Thread(target=self.read, daemon=True).start()

    def read(self):
        stream = self.process.stdout
        while True:
            length = None
            while True:
                line = stream.readline()
                if not line:
                    return
                if not line.strip():
                    break
                if line.lower().startswith(b"content-length:"):
                    length = int(line.split(b":", 1)[1])
            message = json.loads(stream.read(length).decode("utf-8"))
            with self.lock:
                self.messages.append(message)
                self.lock.notify_all()

    def send(self, method, params=None, request=True):
        message = {"jsonrpc": "2.0", "method": method, "params": params or {}}
        if request:
            self.next_id += 1
            message["id"] = self.next_id
        body = json.dumps(message).encode("utf-8")
        self.process.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.process.stdin.flush()
        return message.get("id")

    def wait(self, match, timeout=60):
        deadline = time.time() + timeout
        with self.lock:
            while True:
                for message in self.messages:
                    if match(message):
                        return message
                left = deadline - time.time()
                if left <= 0:
                    return None
                self.lock.wait(left)

    def request(self, method, params=None):
        key = self.send(method, params)
        return self.wait(lambda m: m.get("id") == key and "method" not in m)


def uri(path):
    path = os.path.abspath(path).replace(os.sep, "/")
    return "file://" + ("/" if not path.startswith("/") else "") + re.sub(
        r"[^A-Za-z0-9/._~:-]", lambda m: "".join("%%%02X" % b for b in m.group().encode()), path)


def first_citation(text):
    for number, line in enumerate(text.splitlines()):
        at = line.find("§")
        if at >= 0:
            return number, len(line[:at + 2].encode("utf-16-le")) // 2
    return 0, 0


def language_server(payload, repo):
    session = Session([payload])
    session.request("initialize", {"processId": os.getpid(), "rootUri": uri(repo),
                                   "workspaceFolders": [{"uri": uri(repo), "name": "grund"}],
                                   "capabilities": {}})
    session.send("initialized", request=False)
    for document in DOCUMENTS:
        path = os.path.join(repo, document)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        address = uri(path)
        language = "markdown" if document.endswith(".md") else "rust"
        session.send("textDocument/didOpen", {"textDocument": {
            "uri": address, "languageId": language, "version": 1, "text": text}}, request=False)
        # The server publishes only where it finds something, and answers in order: the
        # hover's answer is what says this document was checked and published.
        line, character = first_citation(text)
        session.request("textDocument/hover", {"textDocument": {"uri": address},
                                               "position": {"line": line, "character": character}})
    session.request("shutdown")
    session.send("exit", request=False)
    session.process.stdin.close()
    return session.process.wait(timeout=60)


NODE_SCRIPT = r"""
const grund = require(process.argv[1]);
const root = process.argv[2];
const call = {
  check: () => grund.check(root),
  list: () => grund.list({root}),
  show: (step) => grund.show(step.id, {root, mode: step.mode}),
  refs: (step) => grund.refs(step.id, {root}),
  cover: () => grund.cover({root}),
  fmt: () => grund.fmt({root}),
};
(async () => {
  for (let i = 0; i < 3; i++) {
    for (const step of JSON.parse(process.argv[3])) await call[step.op](step);
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
"""

PYTHON_SCRIPT = r"""
import json
import sys
sys.path.insert(0, sys.argv[1])
import grund
root = sys.argv[2]
call = {
    "check": lambda step: grund.check(root),
    "list": lambda step: grund.list_ids(root=root),
    "show": lambda step: grund.show(step["id"], mode=step["mode"], root=root),
    "refs": lambda step: grund.refs(step["id"], root=root),
    "cover": lambda step: grund.cover(root=root),
    "fmt": lambda step: grund.fmt(root=root),
}
for _ in range(3):
    for step in json.loads(sys.argv[3]):
        call[step["op"]](step)
"""


def workspace_version(repo):
    with open(os.path.join(repo, "Cargo.toml"), encoding="utf-8") as handle:
        text = handle.read()
    return re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', text).group(1)


def node_addon(payload, repo, work, target):
    """Stage the binding the way an install lays it out, its addon the instrumented one."""
    stage = os.path.join(work, "grund-cli")
    shutil.rmtree(stage, ignore_errors=True)
    os.makedirs(os.path.join(stage, "native"))
    node = os.path.join(repo, "crates", "grund-node")
    for name in os.listdir(os.path.join(node, "js")):
        shutil.copyfile(os.path.join(node, "js", name), os.path.join(stage, name))
    version = workspace_version(repo)
    with open(os.path.join(node, "package-api.json"), encoding="utf-8") as handle:
        package = dict({"name": "grund-cli", "version": version}, **json.load(handle))
    with open(os.path.join(stage, "package.json"), "w", encoding="utf-8") as handle:
        json.dump(package, handle)
    shutil.copyfile(payload, os.path.join(stage, "native", "grund.node"))
    with open(os.path.join(stage, "native", "metadata.json"), "w", encoding="utf-8") as handle:
        json.dump({"apiSchemaVersion": 1, "engineVersion": version, "packageVersion": version,
                   "target": target, "napiVersion": 8}, handle)
    node_binary = os.environ.get("GRUND_PGO_NODE") or shutil.which("node") or "node"
    # A directory is required through its `main`, which the package leaves to `exports`.
    return subprocess.call([node_binary, "-e", NODE_SCRIPT, os.path.join(stage, "index.cjs"),
                            repo, json.dumps(steps())])


def python_extension(payload, repo, work):
    stage = os.path.join(work, "python")
    shutil.rmtree(stage, ignore_errors=True)
    shutil.copytree(os.path.join(repo, "python", "grund"), os.path.join(stage, "grund"))
    suffix = ".pyd" if payload.endswith(".dll") else ".abi3.so"
    shutil.copyfile(payload, os.path.join(stage, "grund", "_native" + suffix))
    python = os.environ.get("GRUND_PGO_PYTHON") or sys.executable
    return subprocess.call([python, "-c", PYTHON_SCRIPT, stage, repo, json.dumps(steps())])


def executable(payload, repo):
    """§AR-benchmarks.1.5: the `pgo` subset against this repository's own tree, three times.
    Exit codes are not the point (a non-canonical tree makes `fmt --check` exit 1): only the
    code paths run, and a payload that cannot run leaves no profile for `pgo-build.sh` to find."""
    for _ in range(3):
        for row in subset():
            try:
                subprocess.call([payload] + arguments(row) + [repo], stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL)
            except OSError as error:
                print("error: cannot run %s: %s" % (payload, error), file=sys.stderr)
                return 1
    return 0


def main(argv):
    if argv[:1] == ["describe"]:
        print(json.dumps(workload(argv[1])))
        return 0
    product, payload, repo, work, target = argv
    os.makedirs(work, exist_ok=True)
    if product == "grund":
        return executable(payload, repo)
    if product == "grund-lsp":
        return language_server(payload, repo)
    if product == "node-addon":
        return node_addon(payload, repo, work, target)
    return python_extension(payload, repo, work)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
