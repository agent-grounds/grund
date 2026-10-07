#!/usr/bin/env python3
"""Upload one verified, rehearsed candidate's npm and PyPI artifacts, and nothing else.

§FS-distribution-candidate.8.3: the candidate is verified as a release and every
registry row's receipt must name its manifest digest before any registry is asked
anything; this script only reads and uploads the candidate's files and never runs a
build tool. §FS-distribution-candidate.8.4: the crates must already resolve; npm
platform packages go up and resolve before their umbrellas, and each Python
distribution goes up as one complete set. §FS-distribution-candidate.8.5: a rerun
skips what is already there with the same digest and stops on anything that differs.

It is reachable only from the disabled `cross-registry-publish.yml`
(§FS-distribution-candidate.8.1).
"""

import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import matrix  # noqa: E402
import registries  # noqa: E402
import verify  # noqa: E402
from registries import Refused  # noqa: E402

ATTEMPTS = 90
CHECKS = ("inventory", "install", "cli-parity", "api-parity", "lsp-lifecycle", "provenance",
          "source-install")


def gate(root, digest, sha):
    """§FS-distribution-candidate.8.3, before any registry is asked: verify, digest, receipts."""
    problems = verify.verify(root, release=True, sha=sha)
    manifest, actual = verify.read_manifest(root) if not problems else (None, None)
    if manifest is not None and actual != digest:
        problems.append(f"--manifest-sha256 {digest} is not the candidate's manifest ({actual})")
    for row in matrix.registry_rows() if manifest is not None else []:
        path = root / "receipts" / f"{row['row']}.json"
        try:
            receipt = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            problems.append(f"no receipt for row {row['row']}: it was not rehearsed on {row['runner']}")
            continue
        if receipt.get("manifest_sha256") != actual or receipt.get("row") != row["row"]:
            problems.append(f"the receipt for {row['row']} names manifest "
                            f"{receipt.get('manifest_sha256')}, not this candidate's {actual}")
        elif any((receipt.get("checks") or {}).get(c) != "passed" for c in CHECKS):
            problems.append(f"the receipt for {row['row']} does not record every check passed")
    return manifest, problems


class Run:
    def __init__(self, args, root, manifest):
        self.args, self.root, self.manifest = args, root, manifest
        self.items = [a for a in manifest["artifacts"] if a["registry"] in ("npm", "pypi")]
        self.state = {a["path"]: "pending" for a in self.items}

    def data(self, item):
        return (self.root / item["path"]).read_bytes()

    def write_status(self):
        if not self.args.status_out:
            return
        status = {"version": self.manifest["version"], "manifest_sha256": self.args.manifest_sha256,
                  "complete": all(s in ("published", "skipped") for s in self.state.values()),
                  "items": [{"artifact": a["path"], "registry": a["registry"],
                             "package": a["package"], "state": self.state[a["path"]]}
                            for a in self.items]}
        Path(self.args.status_out).write_text(json.dumps(status, indent=2) + "\n", encoding="utf-8")

    def wait(self, what, ready):
        """§FS-distribution-candidate.8.4: 90 attempts, the configured interval apart."""
        for attempt in range(ATTEMPTS):
            if ready():
                return
            if attempt + 1 < ATTEMPTS:
                time.sleep(self.args.readiness_interval_seconds)
        raise Refused(f"{what} never appeared after {ATTEMPTS} attempts")

    def crates(self):
        for item in (a for a in self.manifest["artifacts"] if a["kind"] == "crate"):
            self.wait(f"crate {item['package']} {item['version']} on crates.io",
                      lambda i=item: registries.crate_resolves(self.args.crates_url, i["package"],
                                                               i["version"]))

    def npm_order(self):
        npm = [a for a in self.items if a["registry"] == "npm"]
        return [a for a in npm if a["row"]] + [a for a in npm if not a["row"]]

    def preflight(self):
        """§FS-distribution-candidate.8.5: what each registry already holds, all before any upload."""
        todo = []
        for item in self.npm_order():
            held = registries.npm_held(self.args.npm_registry, item["package"], item["version"])
            if held is None:
                todo.append(item)
            elif held == registries.npm_integrity(self.data(item)):
                self.state[item["path"]] = "skipped"
            else:
                self.state[item["path"]] = "failed"
                raise Refused(f"npm holds {item['package']}@{item['version']} with other bytes; "
                              "nothing is overwritten")
        sets = {}
        for name in ("grund", "grund-lsp"):
            files = [a for a in self.items if a["registry"] == "pypi" and a["package"] == name]
            held = registries.pypi_held(self.args.pypi_url, name, files[0]["version"])
            for item in files:
                found = held.get(Path(item["path"]).name)
                if found == item["sha256"]:
                    self.state[item["path"]] = "skipped"
                elif found is not None:
                    self.state[item["path"]] = "failed"
                    raise Refused(f"pypi holds {Path(item['path']).name} with other bytes; "
                                  "nothing is overwritten")
            sets[name] = files
        return todo, sets

    def publish(self, tokens, pypi_token, todo, sets):
        for item in todo:
            self.state[item["path"]] = "failed"
            data = self.data(item)
            registries.npm_publish(self.args.npm_registry, item["package"], item["version"], data,
                                   tokens[item["package"]])
            integrity = registries.npm_integrity(data)
            self.wait(f"{item['package']}@{item['version']} on npm", lambda i=item: integrity ==
                      registries.npm_held(self.args.npm_registry, i["package"], i["version"]))
            self.state[item["path"]] = "published"
        for name, files in sets.items():
            for item in (f for f in files if self.state[f["path"]] == "pending"):
                self.state[item["path"]] = "failed"
                registries.pypi_upload(self.args.pypi_upload_url, self.root / item["path"],
                                       self.data(item), pypi_token)
                self.state[item["path"]] = "published"
            expected = {Path(f["path"]).name: f["sha256"] for f in files}
            self.wait(f"the complete {name} {files[0]['version']} set on pypi", lambda n=name, f=files:
                      registries.pypi_held(self.args.pypi_url, n, f[0]["version"]) == expected)

    def run(self):
        oidc = registries.identity_token(registries.NPM_AUDIENCE)
        self.crates()
        names = [a["package"] for a in self.items if a["registry"] == "npm"]
        tokens = {name: registries.npm_exchange(self.args.npm_registry, name, oidc) for name in names}
        pypi_token = registries.pypi_mint(self.args.pypi_url)
        todo, sets = self.preflight()
        self.publish(tokens, pypi_token, todo, sets)


def main(argv=None):
    parser = argparse.ArgumentParser(prog="publish.py", description=__doc__.split("\n\n")[0])
    parser.add_argument("--candidate", required=True, help="the verified candidate directory")
    parser.add_argument("--manifest-sha256", required=True, help="the rehearsed manifest's digest")
    parser.add_argument("--sha", required=True, help="the commit the candidate was built from")
    parser.add_argument("--status-out", help="write every npm and PyPI artifact's state here")
    parser.add_argument("--readiness-interval-seconds", type=float, default=20, metavar="SECONDS",
                        help=f"seconds between the {ATTEMPTS} attempts of every readiness wait "
                             "(default: 20)")
    parser.add_argument("--npm-registry", default="https://registry.npmjs.org")
    parser.add_argument("--pypi-url", default="https://pypi.org")
    parser.add_argument("--pypi-upload-url", default="https://upload.pypi.org/legacy/")
    parser.add_argument("--crates-url", default="https://crates.io")
    args = parser.parse_args(argv)
    for name in ("npm_registry", "pypi_url", "crates_url"):
        setattr(args, name, getattr(args, name).rstrip("/"))
    root = Path(args.candidate)
    manifest, problems = gate(root, args.manifest_sha256, args.sha)
    if problems:
        sys.stderr.write("".join(f"error: {p}\n" for p in problems))
        sys.stderr.write("error: refused before asking any registry\n")
        return 1
    run = Run(args, root, manifest)
    try:
        run.run()
    except Refused as refusal:
        sys.stderr.write(f"error: {refusal}\n")
        return 1
    finally:
        run.write_status()
    published = sum(1 for s in run.state.values() if s == "published")
    print(f"ok: {manifest['version']} complete, {published} published, "
          f"{len(run.state) - published} already there")
    return 0


if __name__ == "__main__":
    sys.exit(main())
