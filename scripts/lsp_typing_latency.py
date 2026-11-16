#!/usr/bin/env python3
"""Type a citation into an open document over LSP and time each completion answer.

usage: lsp_typing_latency.py <root> <file> <comment-prefix> [paced|burst] [interval-ms]
Nothing is written to disk: every edit is a textDocument/didChange.

The editor typing record of `docs/benchmarks.md` (§DA-benchmark-instruction-counting.3.4): how
long `grund-lsp`, `GRUND_LSP` when set, takes to answer each keystroke's completion, which
§FS-lsp.responsiveness.4 answers from the document's own text without waiting for a rebuild.
This is the driver of agent-grounds/grund#1386. One change: the open is timed by a hover that
waits for its recheck (§FS-lsp.responsiveness.3) rather than by the opened file's first
diagnostics, which a file with no findings never gets.
"""
import json, os, subprocess, sys, threading, time, queue, pathlib

root, path, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
mode = sys.argv[4] if len(sys.argv) > 4 else "paced"
interval = float(sys.argv[5]) / 1000 if len(sys.argv) > 5 else 0.08
lsp = os.environ.get("GRUND_LSP", "grund-lsp")
proc = subprocess.Popen([lsp], cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                        stderr=subprocess.DEVNULL)
inbox = queue.Queue()

def reader():
    out = proc.stdout
    while True:
        headers = {}
        while True:
            line = out.readline()
            if not line:
                return
            line = line.strip()
            if not line:
                break
            k, v = line.decode().split(":", 1)
            headers[k.lower()] = v.strip()
        body = out.read(int(headers["content-length"]))
        msg = json.loads(body)
        msg["_bytes"] = len(body)
        inbox.put((time.perf_counter(), msg))

threading.Thread(target=reader, daemon=True).start()

def send(msg):
    data = json.dumps(msg).encode()
    proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(data) + data)
    proc.stdin.flush()

def wait_for(pred, timeout=120):
    end = time.perf_counter() + timeout
    while True:
        t, msg = inbox.get(timeout=max(0.001, end - time.perf_counter()))
        if pred(msg):
            return t, msg

root_uri = pathlib.Path(root).resolve().as_uri()
uri = pathlib.Path(path).resolve().as_uri()
t0 = time.perf_counter()
send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
    "processId": os.getpid(), "rootUri": root_uri,
    "workspaceFolders": [{"uri": root_uri, "name": "root"}],
    "capabilities": {"textDocument": {"completion": {"completionItem": {}}}}}})
t_init, _ = wait_for(lambda m: m.get("id") == 1)
send({"jsonrpc": "2.0", "method": "initialized", "params": {}})
base = pathlib.Path(path).read_text()
if not base.endswith("\n"):
    base += "\n"
lang = "java" if path.endswith(".java") else "markdown"
send({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {
    "uri": uri, "languageId": lang, "version": 1, "text": base}}})
send({"jsonrpc": "2.0", "id": 2, "method": "textDocument/hover", "params": {
    "textDocument": {"uri": uri}, "position": {"line": 0, "character": 0}}})
t_open, _ = wait_for(lambda m: m.get("id") == 2)
print(f"startup: initialize answered {1000*(t_init-t0):.0f} ms, "
      f"the opened file checked {1000*(t_open-t0):.0f} ms")
# Let any further diagnostics from the open settle.
try:
    while True:
        inbox.get(timeout=1.0)
except queue.Empty:
    pass

line_no = base.count("\n")
lead = prefix + " see "
keys = ["$", "$", "F", "S", "-", "0", "0", "2", "\b", "\b", "Z"]
typed, version, rid = "", 1, 100
sent = []
for key in keys:
    typed = typed[:-1] if key == "\b" else typed + key
    version += 1
    rid += 1
    text = base + lead + typed + "\n"
    t_key = time.perf_counter()
    send({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
        "textDocument": {"uri": uri, "version": version},
        "contentChanges": [{"text": text}]}})
    trigger = {"triggerKind": 2, "triggerCharacter": "$"} if key == "$" else {"triggerKind": 3}
    send({"jsonrpc": "2.0", "id": rid, "method": "textDocument/completion", "params": {
        "textDocument": {"uri": uri},
        "position": {"line": line_no, "character": len(lead + typed)}, "context": trigger}})
    sent.append((rid, "<BS>" if key == "\b" else key, typed, t_key))
    if mode == "paced":
        seen = [0, 0]
        def until(m, r=rid):
            if m.get("method") == "textDocument/publishDiagnostics":
                seen[0] += 1; seen[1] += m["_bytes"]
            return m.get("id") == r
        t_ans, msg = wait_for(until)
        items = len((msg.get("result") or {}).get("items", []))
        print(f"key {sent[-1][1]:>4}  prefix {lead + typed!r:<28} "
              f"answered {1000*(t_ans-t_key):7.1f} ms"
              f"  items {items}  (after {seen[0]} publishDiagnostics, {seen[1]} bytes)")
        try:
            while True:
                inbox.get(timeout=0.5)
        except queue.Empty:
            pass
    else:
        time.sleep(interval)
if mode == "burst":
    answers = {}
    while len(answers) < len(sent):
        t, msg = wait_for(lambda m: m.get("id") in {s[0] for s in sent})
        answers[msg["id"]] = (t, msg)
    for rid, key, typed, t_key in sent:
        t_ans, msg = answers[rid]
        items = len((msg.get("result") or {}).get("items", []))
        print(f"key {key:>4}  prefix {lead + typed!r:<28} answered {1000*(t_ans-t_key):7.1f} ms"
              f"  items {items}")
if mode == "paced":
    # Control: the same request again with no edit before it.
    for n in range(3):
        rid += 1
        t_req = time.perf_counter()
        send({"jsonrpc": "2.0", "id": rid, "method": "textDocument/completion", "params": {
            "textDocument": {"uri": uri},
            "position": {"line": line_no, "character": len(lead + typed)},
            "context": {"triggerKind": 3}}})
        t_ans, msg = wait_for(lambda m, r=rid: m.get("id") == r)
        print(f"control: completion with no preceding edit answered {1000*(t_ans-t_req):.1f} ms")
send({"jsonrpc": "2.0", "id": 9999, "method": "shutdown"})
try:
    wait_for(lambda m: m.get("id") == 9999, timeout=60)
    send({"jsonrpc": "2.0", "method": "exit"})
except queue.Empty:
    pass
proc.wait(timeout=30)
