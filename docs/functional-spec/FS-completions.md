# FS-completions: grund completes declared IDs in shells

Shell completion makes the read loop cheap: after `grund list` tells a user what exists and `grund <ID>` reads one body, the shell completes `<ID>` so the user never copies it. Completion in an editor is [§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)'s.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations)
(value, component, fetcher, snapshot).

## 1. User-facing command

```
grund completions <bash|zsh|fish>
```

Prints the completion script for the shell on stdout, the same bytes on every run, empty stderr, exit `0`. It installs nothing; where the script goes is the user's choice:

```bash
source <(grund completions bash)
grund completions zsh > ~/.zfunc/_grund
grund completions fish > ~/.config/fish/completions/grund.fish
```

Any other shell is a CLI-level error ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)): both lines on stderr, nothing on stdout, exit `2`:

```text
error: unsupported shell `<shell>`
known shells: bash, zsh, fish
```

The scripts complete every top-level subcommand [§FS-cli.1](FS-cli.md#1-the-default-subcommand) dispatches, and declared IDs from the helper of [§FS-completions.2](FS-completions.md#2-internal-dynamic-helper) in the first ID position of `grund <ID>`, the explicit `show` form, `grund refs <ID>` and `grund fetch <ID>`. For the first argument they offer the subcommands and, once the typed word is non-empty and not a flag, whatever its case, the helper's IDs too, so an empty prompt stays cheap and a lowercase alias (`api/`) completes as an uppercase ID does; after `show`, `refs` or `fetch` the first positional argument is always an ID slot, so they ask for any prefix. They complete IDs for no other shell word.

## 2. Internal dynamic helper

The scripts call a hidden machine surface:

```
grund complete ids [<path>] [--prefix <prefix>] [--sections] [--path <path>]
```

It prints one candidate per line on stdout, in byte order and deduplicated, with empty stderr and exit `0`. The scripts call it on a Tab press, so when the config cannot be loaded or the tree cannot be scanned it prints nothing and exits `0`, the one failed run allowed to do so ([§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api)). But an invalid helper flag is a CLI-level error, exit `2`, since it means the installed script is broken.

- `<path>` and `--path <path>` choose where config is discovered and what is scanned. With neither, the candidates are the project's ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)); a path narrower than its config root bounds them to the entries `list <path>` prints ([§FS-scan.path.list](FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project)), even though `grund <ID> <path>` resolves an ID the project declares outside it ([§FS-scan.path.show](FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers)).
- `--prefix <prefix>` keeps the candidates that start with it, byte for byte and over the whole rendered candidate.
- Without `--sections`, the candidates are declared IDs, each in its kind's effective format ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A prefix that contains the section separator, or `--sections` whatever the prefix, asks for section candidates instead, `<ID><section_separator><section>`: `FS-cli.1`, or with named sections `FS-plan.goals.performance` ([§FS-config.3.3.1](FS-config.md#331-named-components)).

The helper reads the tree with the config discovery and scan `list`, `grund <ID>` and `refs` use, so the declarations and the recorded, legal section paths they read are its only ID and section candidates ([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)). Among them are a value source's IDs and numbered components ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)), a declaration's `front-matter` coordinates, as sections ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)), an off-grammar declaration the catalog retains, with its sections, exactly as written ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)), and a relative heading at its canonical path ([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown)). In a workspace it also offers alias paths, and IDs qualified by an alias ([§FS-workspace.8.4](FS-workspace.md#84-shell-completions)). The helper never runs a fetcher, even for a kind with `fetch` ([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)), so a committed snapshot is a candidate like any declaration and one not yet fetched is none.
