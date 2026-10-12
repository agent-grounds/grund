# Declare, cite, check

A small login example of declarations, section retrieval and broken
citations ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases),
[§FS-show.2.1.5](../../docs/functional-spec/FS-show.md#215-a-sections-lead),
[§FS-check.3.2](../../docs/functional-spec/FS-check.md#32-missing-section)).
The fixture's own `grund.toml` maps the `FS` kind to `requirements.md`, enables
`[id] named_sections = true`, and scans `src/`. Its IDs belong to that mini-repository.

## Try it

Install the CLI with `cargo install grund`, or use a binary built from this
checkout. From the root of a Grund checkout, copy the fixture to a temporary
directory so you can edit it freely:

```bash
demo_root=$(mktemp -d)
cp -R examples/quickstart/repo/. "$demo_root/"
cd "$demo_root"
```

If you have no checkout yet, get one with
`git clone https://github.com/agent-grounds/grund.git` and `cd grund` first.

## Declare and cite

The declaration in [`requirements.md`](repo/requirements.md) gives the requirement
an ID and a named `format` section:

```markdown
# FS-login: Login
## format: Username format
Trim surrounding whitespace.
```

[`src/login.py`](repo/src/login.py) cites that section in its docstring:

```python
def clean_username(username):
    """§FS-login.format"""
    return username.strip()
```

## Read and check

```console
$ grund FS-login.format
## format: Username format
Trim surrounding whitespace.
$ grund check
success
```

## Break and repair

In the temporary copy's `src/login.py`, misspell the citation's section: change
`format` to `fromat`. Run the check again:

```console
$ grund check
src/login.py:2: error: section not found: FS-login.fromat; write <§> before it to show the shape without citing it
```

The command exits `1`: the declaration exists, but the section the code names does
not. Correct the spelling back to `format` and rerun
`grund check`; it prints `success` and exits `0` again.

Returning `username` unchanged in the Python code would still pass the citation check.
Grund checks whether the reference resolves; tests and review establish whether
the code satisfies the requirement.

The existing example test runner checks this fixture against its committed
goldens ([§FS-examples.4](../../docs/functional-spec/FS-examples.md#4-maintenance-contract)).
