# Sibling imports: cite a neighbour the build places beside you

This runnable mini-repository demonstrates [§FS-imports](../../docs/functional-spec/FS-imports.md#fs-imports-a-project-cites-a-sibling-project-its-build-places-beside-it-read-live-from-that-tree). It is for a project whose build checks out
another repository beside it, at a path the build fixes, and that wants its citations into
that neighbour checked without a config above both repositories, a second revision pin, or a
committed copy. Open `repo/enterprise/grund.toml` first, then `repo/enterprise/src/Example.java`.

`repo/` stands for the build's checkout: `graal/substratevm` is the neighbour, a project named
`native-image` with one functional-spec point, and `enterprise` is the consumer. Nothing above
them is configured. The consumer declares the neighbour in its own config, one table per
import ([§FS-imports.declaration.table](../../docs/functional-spec/FS-imports.md#declarationtable-the-import-table)):

```toml
# repo/enterprise/grund.toml
[workspace.imports.native-image]
path = "../graal/substratevm"
project = "native-image"
```

The table key is the alias the consumer cites the neighbour by. `path` is relative to the
consumer's config root, written with `/`, and `project` is the `project_name` the neighbour
must declare there, so a different tree at that path is refused rather than guessed at
([§FS-imports.declaration.identity](../../docs/functional-spec/FS-imports.md#declarationidentity-the-project-is-named-never-inferred)).

Run it with the consumer's path as its argument, from the repository root:

```bash
grund check examples/sibling-imports/repo/enterprise
echo $?    # 0
```

stdout is the golden below; stderr is empty:

```text
grund.toml:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned
```

The citation of `native-image/FS-security-providers` in `Example.java` resolves against the
neighbour's own config and files, read live from that tree with its own ignore files and
nothing of the consumer's ([§FS-imports.mount](../../docs/functional-spec/FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). The one line on stdout is not a defect. It says the verdict
depends on a tree no commit pins, once per import and on every run, and no `--only` or
`--ignore` hides it ([§FS-imports.checks.import-live](../../docs/functional-spec/FS-imports.md#checksimport-live-an-import-is-read-live-unpinned)). Because it is a warning, `check`
prints no `success`, and the exit code stays `0`.

## The trade-offs

- **Live, not pinned.** One changed byte in the neighbour moves the consumer's verdict. Pin
  the revision in the build that places the neighbour; grund reads what is there.
- **A missing neighbour fails.** Delete `repo/graal` and the run exits `1` with
  `import-missing` on the same line, and every citation into `native-image` is left
  unchecked rather than passed ([§FS-imports.checks.import-missing](../../docs/functional-spec/FS-imports.md#checksimport-missing-an-import-names-no-directory)).
- **Read-only.** `fmt`, `init` and `fetch` never write into the neighbour from here; run them
  in `repo/graal/substratevm` itself ([§FS-imports.writes](../../docs/functional-spec/FS-imports.md#writes-the-consumer-writes-only-its-own-files)).
- **The neighbour's findings stay its own.** A defect inside it is reported by its own run;
  here it is suppressed, unless it breaks one of the consumer's citations, which then fails
  at the consumer's site ([§FS-imports.findings](../../docs/functional-spec/FS-imports.md#findings-dependency-findings-are-suppressed-consumer-citations-are-not)).
