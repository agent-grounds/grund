"""The PGO training workload of every product.

§FS-distribution-candidate.7.1: each product trains on its own workload, so a profile
describes the code that product runs. §AR-benchmarks.1.5: the `grund` executable runs the
scenario catalog's `pgo` subset against this repository's tree, and the Node addon and the
Python extension run the same subset through native API calls; this is the catalog's one
PGO reader. The language server is driven through an editor session over stdio.
`scripts/pgo-build.sh` runs this with the instrumented payload, and reads `describe` for
the evidence it records.

§AR-benchmarks.1.7: the benchmark harness reads the same definitions. `session --root <dir>`
prints the editor session as framed JSON-RPC for `grund-lsp`'s stdin, and `python-call`
runs one catalog row through the Python extension, once.

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
    """A row's `args` with `{id}` this repository's subject rather than the fixture's.
    The subject is read only for an argument that names `{id}`, so a row that needs one and
    gives none fails with a `KeyError` naming the row instead of training on ''."""
    def subject():
        if "id" not in row["pgo"]:
            raise KeyError("pgo row %r uses {id} but names no `id`" % row["name"])
        return row["pgo"]["id"]
    return [arg.replace("{id}", subject()) if "{id}" in arg else arg for arg in row["args"]]


def step(args, ident=None):
    """A row's arguments as the native call that runs it: the operation, its ID, its mode."""
    return {"op": args[0], "id": ident,
            "mode": next((MODES[arg] for arg in args if arg in MODES), "lead")}


def steps():
    """Each `pgo` row as the native call that runs it."""
    return [step(row["args"], row["pgo"].get("id")) for row in subset()]


def workload(product):
    """The training `describe` prints, which the release evidence records byte for byte."""
    if product == "grund":
        return ["grund " + " ".join(arguments(row)) for row in subset()]
    if product == "grund-lsp":
        return ["grund-lsp " + step for step in LSP]
    prefix = {"node-addon": "node ", "python-extension": "python "}[product]
    return [prefix + row["pgo"]["api"] for row in subset()]


DOCUMENTS = ("README.md", "docs/functional-spec/FS-check.md", "crates/grund-core/src/lib.rs")


def documents(repo):
    """The documents the session opens: the training ones in this repository, or, in a
    generated fixture that has none of them, its first three Markdown files by path."""
    if all(os.path.isfile(os.path.join(repo, document)) for document in DOCUMENTS):
        return list(DOCUMENTS)
    found = []
    for directory, folders, files in os.walk(repo):
        folders[:] = [name for name in folders if not name.startswith(".")]
        found += [os.path.relpath(os.path.join(directory, name), repo).replace(os.sep, "/")
                  for name in files if name.endswith(".md")]
    return sorted(found)[:3]


def session(repo, process_id=None):
    """§AR-benchmarks.1.7, §DA-pgo-release: the editor session `LSP` describes, written once
    for the training client and the benchmark: `(method, params, is_request)` in order."""
    messages = [("initialize", {"processId": process_id, "rootUri": uri(repo),
                                "workspaceFolders": [{"uri": uri(repo), "name": "grund"}],
                                "capabilities": {}}, True),
                ("initialized", {}, False)]
    for document in documents(repo):
        path = os.path.join(repo, document)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        address = uri(path)
        language = "markdown" if document.endswith(".md") else "rust"
        messages.append(("textDocument/didOpen", {"textDocument": {
            "uri": address, "languageId": language, "version": 1, "text": text}}, False))
        # The server publishes only where it finds something, and answers in order: the
        # hover's answer is what says this document was checked and published.
        line, character = first_citation(text)
        messages.append(("textDocument/hover", {"textDocument": {"uri": address},
                                                "position": {"line": line,
                                                             "character": character}}, True))
    return messages + [("shutdown", {}, True), ("exit", {}, False)]


def frame(message):
    body = json.dumps(message).encode("utf-8")
    return b"Content-Length: %d\r\n\r\n" % len(body) + body


def framed_session(repo):
    """§AR-benchmarks.1.7: the session as the bytes a client writes, requests numbered from 1,
    with no process ID, so one fixture always gives the server one input."""
    out, next_id = [], 0
    for method, params, request in session(os.path.abspath(repo)):
        message = {"jsonrpc": "2.0", "method": method, "params": params}
        if request:
            next_id += 1
            message["id"] = next_id
        out.append(frame(message))
    return b"".join(out)


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
        self.process.stdin.write(frame(message))
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
    client = Session([payload])
    for method, params, request in session(repo, os.getpid()):
        if request:
            client.request(method, params)
        else:
            client.send(method, params, request=False)
    client.process.stdin.close()
    return client.process.wait(timeout=60)


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

# The Python call of each operation, shared by the training script and `python-call`.
PYTHON_CALLS = r"""
call = {
    "check": lambda step: grund.check(root),
    "list": lambda step: grund.list_ids(root=root),
    "show": lambda step: grund.show(step["id"], mode=step["mode"], root=root),
    "refs": lambda step: grund.refs(step["id"], root=root),
    "cover": lambda step: grund.cover(root=root),
    "fmt": lambda step: grund.fmt(root=root),
}
"""
PYTHON_SCRIPT = r"""
import json
import sys
sys.path.insert(0, sys.argv[1])
import grund
root = sys.argv[2]
""" + PYTHON_CALLS + r"""
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


def stage_python(payload, repo, work):
    """The `grund` package as an install lays it out, its extension `payload`."""
    stage = os.path.join(work, "python")
    shutil.rmtree(stage, ignore_errors=True)
    shutil.copytree(os.path.join(repo, "python", "grund"), os.path.join(stage, "grund"))
    suffix = ".pyd" if payload.endswith(".dll") else ".abi3.so"
    shutil.copyfile(payload, os.path.join(stage, "grund", "_native" + suffix))
    return stage


def python_extension(payload, repo, work):
    stage = stage_python(payload, repo, work)
    python = os.environ.get("GRUND_PGO_PYTHON") or sys.executable
    return subprocess.call([python, "-c", PYTHON_SCRIPT, stage, repo, json.dumps(steps())])


def python_call(payload, repo, work, root, args):
    """§AR-benchmarks.1.7: one row's operation through the extension, once, in this
    interpreter, so Callgrind toggled on at the extension's entry point counts that call."""
    sys.path.insert(0, stage_python(payload, repo, work))
    import grund  # noqa: F401 - the staged package, read by PYTHON_CALLS
    scope = {"grund": grund, "root": root}
    exec(PYTHON_CALLS, scope)
    ident = args[1] if args[0] in ("show", "refs") else None
    scope["call"][args[0]](step(args, ident))
    return 0


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
    if argv[:2] == ["session", "--root"] and len(argv) == 3:
        out = getattr(sys.stdout, "buffer", sys.stdout)
        out.write(framed_session(argv[2]))
        out.flush()
        return 0
    if argv[:1] == ["python-call"]:
        payload, repo, work, root = argv[1:5]
        return python_call(payload, repo, work, root, argv[5:])
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
