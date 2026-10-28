# AR-ci-self-links: The link gate resolves both self-link forms in the checkout

The internal pass of [§AR-ci.3.1](AR-ci.md#31-citations-and-links) maps a same-repository link, in its `blob/main` form
and its `tree/main` form alike, to its path in the current checkout. This point
says what each form names and how the gate treats the edges, so a directory a
branch adds is linkable in the form the CLI prints for directories ([§FS-cli.2.3](../functional-spec/FS-cli.md#23-a-subcommands-help-page)).

## placement: What the gate decides

```text
lychee --dump ─► [ self-link collection ] ─► checkout path ─► local lychee
                         │
                         └─► every other URL ─► changed-line pass / sweep
```

`scripts/check_links.py` collects the self-links of the dump, maps each into the
checkout, and keeps them off every network pass. This is the repository's own
gate, a meter outside the shipped CLI and core ([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **Self-link**: a `https://github.com/agent-grounds/grund/blob/main/<path>` or
  `https://github.com/agent-grounds/grund/tree/main/<path>` URL, the two forms
  [§AR-ci.3.1](AR-ci.md#31-citations-and-links) names.

## 1. One set of forms

The internal pass collects a `tree/main/<path>` self-link exactly as it collects
the `blob/main` form. One set of forms decides what is a self-link, read alike by
the collection, the mapping into the checkout, and the printed-link test of
[§FS-cli.2.3](../functional-spec/FS-cli.md#23-a-subcommands-help-page), so no step can keep a narrower copy of it. A `tree/main` link whose
directory is in the checkout passes the internal pass and is excluded from the
network, whether the change adds the directory or `main` already has it. A
`tree/main` link to a path the checkout lacks fails the internal pass with
`canonical self-link target is missing`, before any network request, as the
`blob/main` form does.

## 2. The `tree/main` form names a directory

A `tree/main` link to a file fails as missing, as a `blob/main` link to a
directory does. A trailing slash names the same directory: `tree/main/examples/`
is `tree/main/examples`.

## 3. A `tree/main` link carries no fragment

GitHub renders a directory's README under its listing, but which file that is
stays GitHub's choice, so the gate fails a fragment on the `tree/main` form as a
malformed self-link and says it is the fragment. A link to a heading names its
file: `blob/main/<dir>/README.md#<heading>`.

## 4. Where it is held

`tests/integration/test_check_links_tree.py` holds both forms against the gate as
the hook runs it, with a lychee that answers every network request itself and
records each one, so the test knows exactly which URLs would have left the
machine.
