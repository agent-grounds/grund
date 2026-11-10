"""§FS-distribution-candidate.5.2.1: the rehearsal's fresh `PATH` hides Rust and keeps the rest;
§FS-distribution-candidate.5.2.2: it carries no npm setting the contributor exported, in any case,
nor a variable that chooses which global npmrc npm reads.

Each test makes up a `PATH`, and the npm settings exported, and asks the real `fresh_env` about
it, so what this host has installed or exported does not matter. `fresh_env` needs no
candidate, so although the rehearsal itself is manual (§AR-ci.6) this is one of the cheap,
offline tests the Python gate collects, on every operating system a pull request runs on. A
tool is judged by whether it is found and what it prints when run, never by how `fresh_env`
kept it.
"""

import json
import os
import shutil
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

from distribution_support import REPO, scratch
from rehearsal import rehearsal_support

WINDOWS = os.name == "nt"
RUST = ("cargo", "rustc", "rustup")
# §FS-distribution-candidate.5.2.1: what a directory holding one of `RUST` also hides.
TOOLCHAIN = ("rustdoc", "rustfmt", "cargo-clippy", "clippy-driver", "rls", "rust-gdb",
             "cargo-1.75", "rustc-1.75", "rust-lld", "rust-analyzer", "cargo-nextest")
OWN_COMMANDS = ("grund", "grund-lsp")
NAMED_REGISTRY = "http://named.invalid/"
# §FS-distribution-candidate.5.2.2: spellings npm reads, of a setting `fresh_env` does not name
# and of two it does. On Windows `os.environ` folds names to upper case, so there they arrive as
# `NPM_CONFIG_*` and the test holds that one spelling.
MIXED_CASE = {"Npm_Config_Offline": "true", "NPM_Config_Offline": "true",
              "npm_Config_offline": "true", "nPm_CoNfIg_oFfLiNe": "true",
              "Npm_Config_Cache": "inherited-cache", "Npm_Config_Registry": "http://mixed.invalid/"}
EXACT_CASE = {"npm_config_offline": "true", "NPM_CONFIG_OFFLINE": "true",
              "npm_config_cache": "inherited-cache", "NPM_CONFIG_REGISTRY": "http://exact.invalid/"}
# §FS-distribution-candidate.5.2.2: each chooses the global npmrc, where `offline=true` would
# turn npm offline.
GLOBAL_NPMRC_CHOOSERS = {"PREFIX": "chosen-prefix", "DESTDIR": "chosen-destdir"}
# Asks in a process of its own, whose exit removes what `keep` made, from a cwd of its choosing.
CHILD = """
import json, shutil, subprocess, sys
sys.path.insert(0, sys.argv[1])
from rehearsal import rehearsal_support
env = rehearsal_support.fresh_env(sys.argv[2])
tool = shutil.which("tool", path=env["PATH"])
ran = tool and subprocess.run([tool], env=env, cwd=sys.argv[2], capture_output=True, text=True)
print(json.dumps([tool, ran and ran.stdout.strip()]))
"""


def installed(directory, *names):
    """Commands only ever looked for, named as a Rust install names them: `.exe` on Windows."""
    directory.mkdir(parents=True, exist_ok=True)
    for name in names:
        path = directory / (f"{name}.exe" if WINDOWS else name)
        path.touch()
        path.chmod(0o755)


def script(directory, name, posix, windows):
    """A command that really runs, with the suffix `shutil.which` wants on Windows."""
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / (f"{name}.cmd" if WINDOWS else name)
    path.write_text(f"@{windows}\n" if WINDOWS else f"#!/bin/sh\n{posix}\n", encoding="utf-8")
    path.chmod(0o755)
    return path


def saying(directory, name, word):
    return script(directory, name, f"echo {word}", f"echo {word}")


def found(env, name):
    return shutil.which(name, path=env["PATH"])


def answers(env, *names):
    return {name: bool(found(env, name)) for name in names}


def tree(directory):
    """Every entry under `directory` and what it holds, so a removal through a link shows."""
    return {path.relative_to(directory).as_posix(): path.read_bytes() if path.is_file() else None
            for path in sorted(directory.rglob("*"))}


class FreshPathTests(unittest.TestCase):
    """§FS-distribution-candidate.5.2.1, on a `PATH` made up for each test."""

    def setUp(self):
        temp = scratch("grund-fresh-env-")
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.usr = self.root / "usr"

    def fresh(self, *directories):
        with mock.patch.dict(os.environ, PATH=os.pathsep.join(map(str, directories))):
            return rehearsal_support.fresh_env(self.root / "home")

    def answer(self, env, name):
        """What `name` prints when run through the fresh `PATH`."""
        tool = found(env, name)
        self.assertTrue(tool, f"{name} is not on the fresh PATH {env['PATH']!r}")
        return rehearsal_support.run_checked([tool], cwd=self.root, env=env).stdout.strip()

    def child(self, path, cwd):
        """What a process of its own, on this `PATH`, found and heard from `tool`."""
        home = self.root / "child-home"
        result = rehearsal_support.run_checked(
            [sys.executable, "-B", "-c", CHILD, REPO / "tests" / "integration", home],
            cwd=cwd, env={**os.environ, "PATH": path})
        return json.loads(result.stdout)

    def test_rust_beside_a_tool_answers_as_rust_in_a_directory_of_its_own(self):
        """The ticket's example, for each of the three names that mark a toolchain."""
        wanted = {"sh": True, "cargo": False, "rustc": False, "rustup": False}
        for rust in RUST:
            own, beside = self.root / rust / "own", self.root / rust / "beside"
            installed(own / "rust", rust)
            installed(own / "usr", "sh")
            installed(beside, rust, "sh")
            with self.subTest("in a directory of its own", rust=rust):
                self.assertEqual(wanted, answers(self.fresh(own / "rust", own / "usr"), *wanted))
            with self.subTest("beside sh", rust=rust):
                self.assertEqual(wanted, answers(self.fresh(beside), *wanted))

    def test_rust_beside_sh_and_node_leaves_every_one_of_them_found(self):
        """#1234's host B, where Rust beside `sh` and `node` once left nothing on `PATH`."""
        installed(self.usr, "cargo", "sh", "node", "npx", "npm")
        env = self.fresh(self.usr)
        self.assertTrue(env["PATH"], "the fresh PATH is empty")
        self.assertEqual({"sh": True, "node": True, "npx": True, "npm": True, "cargo": False},
                         answers(env, "sh", "node", "npx", "npm", "cargo"))

    def test_the_command_found_first_stays_the_one_found_first(self):
        """A tool in two directories, the one holding Rust first and then last."""
        rust, plain = self.root / "rust", self.root / "plain"
        installed(rust, "cargo")
        saying(rust, "tool", "beside-rust")
        saying(plain, "tool", "plain")
        for first, directories in (("beside-rust", (rust, plain)), ("plain", (plain, rust))):
            with self.subTest(first=first):
                env = self.fresh(*directories)
                self.assertEqual(first, self.answer(env, "tool"))
                self.assertFalse(found(env, "cargo"))

    def test_the_rest_of_the_toolchain_beside_cargo_is_hidden(self):
        """A guard: what keeps the rest of the directory brings none of the toolchain back."""
        installed(self.usr, "cargo", *TOOLCHAIN)
        env = self.fresh(self.usr)
        self.assertEqual({name: False for name in (*RUST, *TOOLCHAIN)},
                         answers(env, *RUST, *TOOLCHAIN))

    def test_the_candidates_own_commands_beside_cargo_are_not_found(self):
        """A guard: a prior `cargo install` must not answer for an install that failed."""
        installed(self.usr, "cargo", *OWN_COMMANDS)
        self.assertEqual({name: False for name in OWN_COMMANDS},
                         answers(self.fresh(self.usr), *OWN_COMMANDS))

    def test_an_unrelated_tool_beside_the_candidates_own_commands_is_found(self):
        """Hiding `grund` hides nothing else `cargo install` put in the same directory."""
        installed(self.usr, "cargo", "just", *OWN_COMMANDS)
        self.assertEqual({"just": True, "grund": False, "grund-lsp": False},
                         answers(self.fresh(self.usr), "just", *OWN_COMMANDS))

    def test_a_directory_holding_only_rust_leaves_nothing_on_the_path_and_makes_no_links(self):
        """A guard: a rustup-only `~/.cargo/bin` needs no link, so no OS can refuse one."""
        rust, plain = self.root / "rust", self.root / "plain"
        installed(rust, *RUST, *TOOLCHAIN, *OWN_COMMANDS)
        installed(plain, "sh")
        with mock.patch("os.symlink") as symlink, mock.patch("os.link") as link:
            self.assertEqual("", self.fresh(rust)["PATH"])
            self.assertEqual(str(plain), self.fresh(rust, plain)["PATH"])
            self.assertEqual(str(plain), self.fresh(plain, rust)["PATH"])
        symlink.assert_not_called()
        link.assert_not_called()

    def test_a_tool_beside_cargo_runs_through_the_fresh_path(self):
        """Being found is not enough: it has to start and print what it prints."""
        installed(self.usr, "cargo")
        saying(self.usr, "tool", "ran")
        self.assertEqual("ran", self.answer(self.fresh(self.usr), "tool"))

    def test_a_shim_beside_cargo_reaches_what_sits_beside_it(self):
        """A Windows npm shim finds its `node_modules` beside itself, so directories stay too."""
        installed(self.usr, "cargo")
        (self.usr / "lib").mkdir(parents=True)
        (self.usr / "lib" / "answer").write_text("reached\n", encoding="utf-8")
        script(self.usr, "tool", 'read line < "${0%/*}/lib/answer"; echo "$line"',
               'type "%~dp0lib\\answer"')
        self.assertEqual("reached", self.answer(self.fresh(self.usr), "tool"))

    @unittest.skipIf(WINDOWS, "the fixture's own symlink needs a privilege Windows may not grant")
    def test_a_relative_link_beside_cargo_answers_by_its_own_name(self):
        """`npm -> ../lib/node_modules/npm/bin/npm-cli.js` leaves the directory; busybox reads `$0`."""
        installed(self.usr, "cargo")
        script(self.root / "opt", "multi", 'echo "${0##*/}"', "")
        (self.usr / "applet").symlink_to(Path("..") / "opt" / "multi")
        self.assertEqual("applet", self.answer(self.fresh(self.usr), "applet"))

    def test_a_relative_path_entry_holding_cargo_still_runs_from_elsewhere(self):
        """The rehearsal starts its commands from a consumer, not from where `PATH` was read."""
        installed(self.usr, "cargo")
        saying(self.usr, "tool", "ran")
        tool, heard = self.child("usr", cwd=self.root)
        self.assertTrue(tool, "the tool is not on the fresh PATH")
        self.assertEqual("ran", heard)

    def test_the_processs_scratch_goes_and_the_original_directory_stays(self):
        """Removing the scratch must not follow a link into `/usr/bin`, nor outlive the process."""
        installed(self.usr, "cargo")
        saying(self.usr, "tool", "ran")
        (self.usr / "lib").mkdir(parents=True)
        (self.usr / "lib" / "answer").write_text("kept\n", encoding="utf-8")
        before = tree(self.usr)
        tool, heard = self.child(str(self.usr), cwd=self.root)
        self.assertTrue(tool, "the tool is not on the fresh PATH")
        self.assertEqual("ran", heard)
        self.assertEqual(before, tree(self.usr))
        self.assertFalse(os.path.lexists(tool), f"{tool} outlived the process that made it")

    def test_a_command_that_cannot_be_kept_fails_the_run_and_never_vanishes(self):
        """A silent drop is the defect: with no link to be had, keep it some other way or refuse."""
        installed(self.usr, "cargo")
        saying(self.usr, "tool", "ran")
        refused = PermissionError("refused by this test")
        with mock.patch("os.symlink", side_effect=refused), \
                mock.patch("os.link", side_effect=refused):
            try:
                env = self.fresh(self.usr)
            except AssertionError as refusal:
                for named in (str(self.usr), "tool", "refused by this test"):
                    self.assertIn(named, str(refusal))
                return
        self.assertTrue(found(env, "tool"), "tool was dropped from the fresh PATH without a word")

    @unittest.skipIf(WINDOWS, "`sh` is the POSIX symptom: npm starts a script through it")
    def test_npm_run_starts_its_script_under_the_fresh_env(self):
        """The shape of `test_a_missing_rust_toolchain_is_named`, with no candidate.

        `npm run`, not the ticket's `npx`: `npx` may ask a registry for `node`, and this stays
        offline. The host's own `sh`, `node` and `npm` are linked into the made-up `PATH`.
        """
        host = {name: shutil.which(name) for name in ("sh", "node", "npm")}
        for name, tool in host.items():
            self.assertTrue(tool, f"environment prerequisite missing: {name}")
        package = self.root / "package"
        package.mkdir()
        (package / "package.json").write_text(
            json.dumps({"private": True, "scripts": {"hello": "echo ran"}}), encoding="utf-8")
        for layout, node_directory in (("node elsewhere", "nodejs"), ("node beside sh", "usr")):
            with self.subTest(layout):
                usr, nodejs = self.root / layout / "usr", self.root / layout / node_directory
                installed(usr, "cargo")
                nodejs.mkdir(exist_ok=True)
                (usr / "sh").symlink_to(host["sh"])
                (nodejs / "node").symlink_to(host["node"])
                (nodejs / "npm").symlink_to(host["npm"])
                env = self.fresh(*dict.fromkeys((usr, nodejs)))
                npm = found(env, "npm")
                self.assertTrue(npm, f"npm is not on the fresh PATH {env['PATH']!r}")
                result = subprocess.run([npm, "run", "hello"], cwd=package, env=env,
                                        capture_output=True, text=True)
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertIn("ran", result.stdout.splitlines())


def npm_settings(env):
    """Every variable npm reads a setting from: its match ignores case."""
    return {key: value for key, value in env.items() if key.lower().startswith("npm_config_")}


class InheritedNpmSettingTests(unittest.TestCase):
    """§FS-distribution-candidate.5.2.2, with npm settings exported in the spellings npm reads."""

    def setUp(self):
        temp = scratch("grund-fresh-env-npm-")
        self.addCleanup(temp.cleanup)
        self.home = Path(temp.name) / "home"
        self.path = Path(temp.name) / "bin"
        self.path.mkdir()

    def fresh_env(self, exported):
        """`fresh_env` when `exported` are the only npm settings and global-npmrc choosers the
        shell holds, on an empty `PATH` of the test's own, so no host directory is walked."""
        unset = {k: v for k, v in os.environ.items()
                 if not k.lower().startswith("npm_config_") and k not in GLOBAL_NPMRC_CHOOSERS}
        with mock.patch.dict(os.environ, {**unset, **exported, "PATH": str(self.path)},
                             clear=True):
            return rehearsal_support.fresh_env(self.home, npm_config_registry=NAMED_REGISTRY)

    def fresh(self, exported):
        """`fresh_env`'s npm settings when `exported` are the only ones the shell holds."""
        return npm_settings(self.fresh_env(exported))

    def own(self):
        """What `fresh_env` sets itself, each once, when the shell exports no npm setting."""
        own = self.fresh({})
        self.assertEqual(len(own), len({key.lower() for key in own}), own)
        self.assertEqual(NAMED_REGISTRY, own.get("npm_config_registry"), own)
        self.assertEqual(str(self.home / "npm-cache"), own.get("npm_config_cache"), own)
        return own

    def test_no_mixed_case_spelling_the_contributor_exported_survives(self):
        """No spelling in `MIXED_CASE` survives, and what `fresh_env` names keeps its value."""
        own, settings = self.own(), self.fresh(MIXED_CASE)
        self.assertEqual([], sorted(set(settings) - set(own)),
                         "npm settings the contributor exported survived fresh_env")
        self.assertEqual(own, settings)

    def test_the_two_usual_spellings_are_dropped(self):
        """The two usual spellings go too, and what `fresh_env` names keeps its value."""
        own = self.own()
        self.assertEqual(own, self.fresh(EXACT_CASE))

    def test_no_variable_that_chooses_the_global_npmrc_survives(self):
        """`PREFIX` and `DESTDIR` go, and the npm settings stay what `fresh_env` names."""
        own, env = self.own(), self.fresh_env(GLOBAL_NPMRC_CHOOSERS)
        self.assertEqual({}, {k: env[k] for k in GLOBAL_NPMRC_CHOOSERS if k in env})
        self.assertEqual(own, npm_settings(env))
