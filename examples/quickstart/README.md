# Declare, cite, check

A small login example of declarations, section retrieval and broken
citations ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases),
[§FS-show.2.1.5](../../docs/functional-spec/FS-show.md#215-a-sections-lead),
[§FS-check.3.2](../../docs/functional-spec/FS-check.md#32-missing-section)).
The fixture's own `grund.toml` is a version-2 file with one row, the `FS` kind in
`docs/functional-spec/`; everything else is a default, so IDs are slug-only, named
sections are on, and the rest of the tree, `src/` included, is scanned
(`grund config show` prints what applies, `grund config explain` every setting).
Its IDs belong to that mini-repository.

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

The declaration in [`docs/functional-spec/FS-login.md`](repo/docs/functional-spec/FS-login.md)
gives the login behavior an ID, a named `format` section, and its numbered child
`format.1`:

```markdown
# FS-login: Login
## format: Username format
### format.1: Surrounding whitespace
Trim surrounding whitespace.
```

The folder's [`README.md`](repo/docs/functional-spec/README.md) lists it, as the index
of a folder kind must.

[`src/login.py`](repo/src/login.py) cites that section in a comment above the line it governs:

```python
def clean_username(username):
    # §FS-login.format.1
    return username.strip()
```

## Read and check

```console
$ grund FS-login.format.1
### format.1: Surrounding whitespace
Trim surrounding whitespace.
$ grund check
success
```

## Break and repair

In the temporary copy's `src/login.py`, misspell the citation's section: change
`format` to `fromat`. Run the check again:

```console
$ grund check
src/login.py:2: error: section not found: FS-login.fromat.1; write <§> before it to show the shape without citing it
```

The command exits `1`: the declaration exists, but the section the code names does
not. Correct the spelling back to `format` and rerun
`grund check`; it prints `success` and exits `0` again.

Returning `username` unchanged in the Python code would still pass the citation check.
Grund checks whether the reference resolves; tests and review establish whether
the code satisfies the spec.

The existing example test runner checks this fixture against its committed
goldens ([§FS-examples.4](../../docs/functional-spec/FS-examples.md#4-maintenance-contract)).
