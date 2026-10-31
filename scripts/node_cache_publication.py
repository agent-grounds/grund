#!/usr/bin/env python3
"""Dependency-only snapshots and non-overwriting restore (§AR-ci.3.4.1.5).

Select third-party package fingerprints and the compiler/build artifacts they
identify. Never infer workspace output from its current target names: persistent
targets also contain earlier names. Nothing in this module invokes Cargo clean
or changes a live target, tagged or otherwise.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def cargo(checkout, *arguments):
    """Read locked/offline metadata with Cargo's diagnostic (§AR-node-cache-metadata.failures)."""
    command = ["cargo", *map(str, arguments), "--locked", "--offline"]
    result = subprocess.run(command, cwd=checkout, capture_output=True, text=True)
    if result.returncode:
        raise ValueError(f"Cargo exited {result.returncode}; owner checkout={checkout}; "
                         f"command={command!r}; stderr:\n{result.stderr}")
    return result.stdout


def owner(target):
    """Resolve the owner's observed features (§AR-node-cache-metadata.resolution)."""
    checkout = Path(json.loads((target / "checkout.json").read_text(encoding="utf-8"))["checkout"])
    if (not checkout.is_absolute() or checkout.resolve() != checkout
            or hashlib.sha256(os.fsencode(checkout)).hexdigest()[:16] != target.name):
        raise ValueError(f"unrecognized checkout ownership: {target}")
    host = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True)
    triple = next(line.removeprefix("host: ") for line in host.stdout.splitlines()
                  if line.startswith("host: "))
    known_targets = set(subprocess.run(["rustc", "--print", "target-list"],
                                      capture_output=True, text=True, check=True).stdout.splitlines())
    triples = {triple} | {p.parent.parent.name for p in target.rglob(".fingerprint")
                         if p.parent.parent.name in known_targets}
    platforms = [argument for value in sorted(triples) for argument in ("--filter-platform", value)]
    workspace = json.loads(cargo(checkout, "metadata", "--no-deps", "--format-version", "1"))
    features = built_features(target, checkout, workspace)
    selected = ["--features", ",".join(features)] if features else []
    metadata = json.loads(cargo(checkout, "metadata", "--format-version", "1", *selected, *platforms))
    return checkout, target_sources(metadata)


def target_sources(metadata):
    """Keep source-qualified Cargo identities for each target (§AR-ci.3.4.1.5)."""
    members = set(metadata["workspace_members"])
    if not members:
        raise ValueError("no workspace packages")
    sources = {}
    for package in metadata["packages"]:
        for unit in package["targets"]:
            source = Path(unit["src_path"]).resolve()
            identity = (package["id"], package["id"] not in members)
            if source in sources and sources[source] != identity:
                raise ValueError(f"ambiguous Cargo target source: {source}")
            sources[source] = identity
    return sources


def built_features(target, checkout, workspace):
    """Resolve observed workspace features (§AR-node-cache-metadata.resolution)."""
    packages = {p["id"]: p for p in workspace["packages"]}
    profiles = {p.parent for p in target.rglob(".fingerprint")}
    units = package_units(profiles, checkout, target_sources(workspace))
    selected = set()
    for profile in profiles:
        for fingerprint in (profile / ".fingerprint").glob("*/*.json"):
            unit_hash = fingerprint.parent.name.rsplit("-", 1)[-1]
            package = packages.get(units[profile].get(unit_hash, (None, False))[0])
            if package is None or fingerprint.name.startswith("run-build-script-"):
                continue
            features = json.loads(json.loads(fingerprint.read_text(encoding="utf-8"))["features"])
            for feature in features:
                if feature in package["features"]:
                    selected.add(package["name"] + "/" + feature)
    return sorted(selected)


def unit_identity(depfile, checkout, sources):
    """Resolve rustc's primary input to its exact Cargo target (§AR-ci.3.4.1.5).

    Rustc depfiles put the target input first. Parse Make escaping rather than
    splitting on ':' (Windows drive letters) or whitespace (escaped spaces).
    The unit hash only joins artifacts after source identity is established.
    """
    lines = depfile.read_text(encoding="utf-8").splitlines()
    if not lines:
        raise ValueError(f"empty Cargo depfile: {depfile}")
    line = lines[0]
    _, prerequisites = line.split(": ", 1)
    match = re.match(r"(?:\\[ #\\]|[^ ])+", prerequisites)
    if not match:
        raise ValueError(f"no primary input in {depfile}")
    source = re.sub(r"\\([ #\\])", r"\1", match[0])
    return sources.get((checkout / source).resolve())


def package_units(profiles, checkout, sources):
    """Join build execution to its identified executable fingerprint (§AR-ci.3.4.1.5)."""
    units = {profile: {} for profile in profiles}
    builds = {}
    depfiles = [(profile, depfile) for profile in profiles for depfile in
                list((profile / "deps").glob("*.d"))
                + list((profile / "build").glob("*/*.d"))]
    for profile, depfile in depfiles:
        match = re.search(r"-([0-9a-f]{16})\.d$", depfile.name)
        if not match:
            continue
        identity = unit_identity(depfile, checkout, sources)
        if identity is None:
            continue
        unit_hash = match[1]
        if unit_hash in units[profile] and units[profile][unit_hash] != identity:
            raise ValueError(f"ambiguous Cargo unit: {depfile}")
        units[profile][unit_hash] = identity
        if depfile.parent.parent.name == "build":
            fingerprint = profile / ".fingerprint" / depfile.parent.name / "build-script-build-script-build"
            if fingerprint.is_file():
                checksum = int.from_bytes(bytes.fromhex(fingerprint.read_text()), "little")
                if checksum in builds and builds[checksum] != identity:
                    raise ValueError(f"ambiguous build-script fingerprint: {fingerprint}")
                builds[checksum] = identity
    # Cross-compilation executes a host build-script unit from another profile.
    for profile in profiles:
        for fingerprint in (profile / ".fingerprint").glob("*/run-build-script-*.json"):
            dependencies = json.loads(fingerprint.read_text(encoding="utf-8"))["deps"]
            linked = [builds[d[-1]] for d in dependencies
                      if d[1] == "build_script_build" and d[-1] in builds]
            if len(linked) == 1:
                units[profile][fingerprint.parent.name.rsplit("-", 1)[1]] = linked[0]
    return units


def copy_dependencies(target, copied, ownership):
    """Select package-owned state across host/cross/nested profiles (§AR-ci.3.4.1.5).

    Compiler depfiles join each unit hash to a metadata target's exact source
    location and opaque package ID. Build execution joins the checksum of that
    package's compiled build script. Names never establish ownership.
    """
    def copy(source):
        destination = copied / source.relative_to(target)
        destination.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(source, destination, copy_function=shutil.copy2)
        else:
            shutil.copy2(source, destination)

    copy(target / "checkout.json")
    fingerprint_roots = sorted(target.rglob(".fingerprint"))
    profiles = {path.parent for path in fingerprint_roots}
    checkout, sources = ownership
    units = package_units(profiles, checkout, sources)
    for path in target.rglob("*"):
        if (path.name in ("CACHEDIR.TAG", ".rustc_info.json") and path.is_file()
                and not any(profile in path.parents for profile in profiles)):
            copy(path)
    for fingerprints in fingerprint_roots:
        profile = fingerprints.parent
        if profile.name not in ("debug", "release"):
            raise ValueError(f"unknown Cargo profile: {profile}")
        hashes = set()
        for fingerprint in fingerprints.iterdir():
            match = re.fullmatch(r"(.+)-([0-9a-f]{16})", fingerprint.name)
            if not match:
                raise ValueError(f"unrecognized Cargo fingerprint: {fingerprint}")
            if units[profile].get(match[2], (None, False))[1]:
                hashes.add(match[2])
                copy(fingerprint)
                build = profile / "build" / fingerprint.name
                if build.is_dir():
                    copy(build)
        deps = profile / "deps"
        if deps.is_dir():
            for artifact in deps.iterdir():
                match = re.search(r"-([0-9a-f]{16})(?:\.|$)", artifact.name)
                if match and match[1] in hashes:
                    copy(artifact)


def prepare(root, snapshot):
    """Publish only a completely cleaned copy, preserving the live bytes (§AR-ci.3.4.1.5)."""
    # Retire the previous restored view before work: a failed preparation must
    # never leave an old or partially filtered view eligible for a new save.
    if snapshot.exists():
        shutil.rmtree(snapshot)
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=snapshot.name + "-", dir=snapshot.parent) as temporary:
        staged = Path(temporary) / "view"
        staged.mkdir()
        for target in sorted(root.iterdir()):
            if target.name == "tools":
                continue
            if not target.is_dir() or target.is_symlink():
                raise ValueError(f"unrecognized Node target: {target}")
            ownership = owner(target)
            if any(p.is_symlink() for p in target.rglob("*")):
                raise ValueError(f"symlink in Node target: {target}")
            copied = staged / target.name
            copy_dependencies(target, copied, ownership)
        staged.rename(snapshot)
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
            output.write("ready=true\n")


def restore(root, snapshot):
    """Materialize at the same relative paths; never overwrite live output (§AR-ci.3.4.1.5)."""
    if not snapshot.is_dir():
        return
    for source in snapshot.rglob("*"):
        if source.is_symlink():
            raise ValueError(f"symlink in Node snapshot: {source}")
        if not source.is_file():
            continue
        destination = root / source.relative_to(snapshot)
        # Refuse live symlink parents too; cached files cannot redirect writes.
        if any(p.is_symlink() for p in (root, *destination.parents)):
            raise ValueError(f"symlink in restore destination: {destination}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        try:
            with destination.open("xb") as output, source.open("rb") as input_file:
                shutil.copyfileobj(input_file, output)
        except FileExistsError:
            continue
        shutil.copystat(source, destination)


def main():
    """A publication failure only warns; the workflow saves only ready views (§AR-ci.3.4.1.5)."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("prepare", "restore"))
    arguments = parser.parse_args()
    root = Path(os.environ.get("GRUND_NODE_TARGET", "~/ag/tmp/grund-node-target")).expanduser()
    snapshot = root.with_name(root.name + "-dependencies")
    try:
        {"prepare": prepare, "restore": restore}[arguments.operation](root, snapshot)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        if arguments.operation == "prepare" and snapshot.exists():
            shutil.rmtree(snapshot, ignore_errors=True)
        diagnostic = str(error).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
        print(f"::warning::Node cache {arguments.operation} skipped: {diagnostic}")


if __name__ == "__main__":
    main()
