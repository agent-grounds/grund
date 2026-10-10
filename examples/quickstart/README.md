# Declare, cite, check

A small display-name example of declarations, section retrieval and broken
citations ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases),
[§FS-show.2.1.5](../../docs/functional-spec/FS-show.md#215-a-sections-lead),
[§FS-check.3.2](../../docs/functional-spec/FS-check.md#32-missing-section)).
The fixture's own `grund.toml` maps the `FS` kind to `requirements.md` and scans
`src/`. Its IDs belong to that mini-repository.

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
an ID and a numbered section:

```markdown
# FS-name: Display names
## 1. Trim whitespace
Trim surrounding whitespace.
```

[`src/name.py`](repo/src/name.py) cites that section in its docstring:

```python
def clean_name(name):
    """§FS-name.1"""
    return name.strip()
```

## Read and check

```console
$ grund FS-name.1
## 1. Trim whitespace
Trim surrounding whitespace.
$ grund check
success
```

## Break and repair

In the temporary copy's `requirements.md`, change `## 1. Trim whitespace` to
`## 2. Trim whitespace`, leaving the code's citation unchanged. Run the check again:

```console
$ grund check
src/name.py:2: error: missing section FS-name.1
```

The command exits `1`: the declaration still exists, but the section the code
names does not. Change the heading back to `## 1. Trim whitespace` and rerun
`grund check`; it prints `success` and exits `0` again.

Returning `name` unchanged in the Python code would still pass the citation check.
Grund checks whether the reference resolves; tests and review establish whether
the code satisfies the requirement.

The existing example test runner checks this fixture against its committed
goldens ([§FS-examples.4](../../docs/functional-spec/FS-examples.md#4-maintenance-contract)).
