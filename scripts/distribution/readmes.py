"""What each npm and PyPI package says it holds (§FS-distribution-candidate.2.4).

A package's README is the first thing a registry shows, so each one names what that
package installs, which row a platform package serves, and where the CLI and its
installation guide live. Written here rather than as files under `scripts/`, so the
text a user reads carries no citation of this repository's specification.
"""

REPO = "https://github.com/agent-grounds/grund"
GUIDE = f"{REPO}/blob/main/docs/user-facing/installation.md"
LSP_GUIDE = f"{REPO}/blob/main/docs/user-facing/lsp.md"
NODE_API = f"{REPO}/blob/main/crates/grund-node/README.md"
PYTHON_API = f"{REPO}/blob/main/crates/grund-py/README.md"

_FOOTER = f"""
The `grund` CLI, its documentation and every way to install it are at
[github.com/agent-grounds/grund]({REPO}); the [installation guide]({GUIDE}) lists
the supported platforms and what a source build needs. MIT licensed.
"""


def npm_umbrella(family, version):
    if family == "grund-cli":
        return f"""# grund-cli {version}

The [grund]({REPO}) CLI and its Node API in one package. Installing it puts the `grund`
command on `PATH` and makes `import {{ check }} from "grund-cli"` work, both from one
prebuilt platform package chosen for your OS, CPU and libc — no Rust toolchain, and
nothing is compiled on install.

```sh
npx grund check
```

The API is described in the [Node binding's README]({NODE_API}). It does not include the
language server; that is the separate `grund-lsp` package.

On a platform with no prebuilt package, `npm run build:source` inside the installed
package builds both from the bundled, locked sources; it needs Rust and Cargo.
{_FOOTER}"""
    return f"""# grund-lsp {version}

The [grund]({REPO}) language server, alone: diagnostics, hover, go-to-definition and
completion for ID-based citations. Installing it puts the `grund-lsp` command on `PATH`
for your editor to launch, from one prebuilt platform package chosen for your OS, CPU
and libc. It does not install the `grund` CLI and needs nothing else; the
[editor setup]({LSP_GUIDE}) says how to point an editor at it.

On a platform with no prebuilt package, `npm run build:source` inside the installed
package builds the server from the bundled, locked sources; it needs Rust and Cargo.
{_FOOTER}"""


def npm_platform(family, version, row):
    holds = ("the `grund` executable and the Node addon" if family == "grund-cli"
             else "the `grund-lsp` executable")
    return f"""# @{family}/{row['npm_suffix']} {version}

The {row['npm_suffix']} payload of `{family}`: {holds}, built for
`{row['rust_target']}`. It is installed by `{family}`, which selects the one platform
package that matches the host; install `{family}`, not this package.
{_FOOTER}"""


def pypi(dist, version):
    if dist == "grund":
        return f"""# grund {version}

The [grund]({REPO}) CLI and its Python API in one distribution. Installing the wheel
puts the `grund` executable itself on `PATH` — `pip install`, `pipx install` and
`uv tool install` all run it with no Python launcher in front — and makes
`import grund` work on CPython 3.10 to 3.14 from one stable-ABI wheel per platform.

```python
import grund
report = grund.check(".")
```

The API is described in the [Python binding's README]({PYTHON_API}). The language
server is the separate `grund-lsp` distribution. Building from the sdist
(`pip install --no-binary=:all: grund`) needs Rust and Cargo.
{_FOOTER}"""
    return f"""# grund-lsp {version}

The [grund]({REPO}) language server, alone. Installing the wheel puts the `grund-lsp`
executable itself on `PATH` for your editor to launch; it does not install the `grund`
CLI, and the [editor setup]({LSP_GUIDE}) says how to point an editor at it. Building
from the sdist (`pip install --no-binary=:all: grund-lsp`) needs Rust and Cargo.
{_FOOTER}"""
