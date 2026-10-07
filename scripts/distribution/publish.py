#!/usr/bin/env python3
"""Upload one verified, rehearsed candidate's npm and PyPI artifacts, and nothing else.

§FS-distribution-candidate.8.2: every identity is exchanged before anything waits or uploads,
and each upload then takes a token exchanged immediately before it.
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

    def npm_token(self, name, oidc=None):
        oidc = oidc or registries.identity_token(registries.NPM_AUDIENCE)
        return registries.npm_exchange(self.args.npm_registry, name, oidc)

    def authorize(self):
        """§FS-distribution-candidate.8.2: every identity, once, as the authority check; the
        tokens are dropped, since a readiness wait can outlast them."""
        oidc = registries.identity_token(registries.NPM_AUDIENCE)
        for item in self.items:
            if item["registry"] == "npm":
                self.npm_token(item["package"], oidc)
        registries.pypi_mint(self.args.pypi_url)

    def pypi_token(self, item):
        try:
            return registries.pypi_mint(self.args.pypi_url)
        except Refused as refusal:
            raise Refused(f"{refusal} (stopped before {Path(item['path']).name})") from None

    def publish(self, todo, sets):
        for item in todo:
            self.state[item["path"]] = "failed"
            data = self.data(item)
            # §FS-distribution-candidate.8.2: exchanged now, after the previous package's wait.
            registries.npm_publish(self.args.npm_registry, item["package"], item["version"], data,
                                   self.npm_token(item["package"]))
            integrity = registries.npm_integrity(data)
            self.wait(f"{item['package']}@{item['version']} on npm", lambda i=item: integrity ==
                      registries.npm_held(self.args.npm_registry, i["package"], i["version"]))
            self.state[item["path"]] = "published"
        for name, files in sets.items():
            token = None
            for item in (f for f in files if self.state[f["path"]] == "pending"):
                self.state[item["path"]] = "failed"
                # §FS-distribution-candidate.8.2: minted once per set, after the waits before it;
                # PyPI's trusted-publishing token lives at most 15 minutes, a wait up to 30.
                token = token or self.pypi_token(item)
                registries.pypi_upload(self.args.pypi_upload_url, self.root / item["path"],
                                       self.data(item), token)
                self.state[item["path"]] = "published"
            expected = {Path(f["path"]).name: f["sha256"] for f in files}
            self.wait(f"the complete {name} {files[0]['version']} set on pypi", lambda n=name, f=files:
                      registries.pypi_held(self.args.pypi_url, n, f[0]["version"]) == expected)

    def run(self):
        self.authorize()
        self.crates()
        todo, sets = self.preflight()
        self.publish(todo, sets)


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
