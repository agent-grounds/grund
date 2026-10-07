# REQ-runs-offline: verification never depends on an external service

The same committed tree and configuration must remain verifiable on a disconnected
machine. A citation is grounded by repository bytes, never by a service response that
can change, disappear, rate-limit, or require credentials. This preserves
[§REQ-deterministic-output](REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and the fast local loop of [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible).

## 1. Read and verification paths execute nothing

Scanning, `check`, ID queries, `refs`, `list`, `cover`, `config show` and `validate`,
`fmt`, shell completion and its dynamic helper, and every LSP request perform no network
I/O and execute no repository-configured process. This remains true for a kind that
configures a fetcher and for a missing snapshot. Those paths read only the tree and
configuration they already own.

## 2. Materialization is explicit

Only a deliberate `grund fetch` invocation may reach outside the tree, and it names what it materializes. `grund fetch <ID>` executes the selected kind's configured integration under [§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot); `grund fetch --remote <alias>`, or `--remote --all`, reads the repository a `[workspace.remotes.<alias>]` table declares, under [§FS-remote-projects](../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection). Neither makes verification online: an integration's output must first become a local Markdown declaration, and a remote must first become a committed projection pinned in `grund.lock`. Subsequent resolution reads those committed bytes through the ordinary scanner.

## 3. No implicit freshness

Grund never asks about freshness on its own: no verification path, timer or editor event checks whether an upstream moved, re-fetches, or silently materializes a missing citation. Freshness is asked only by an explicit `grund fetch`, `--remote --check` included, and a `--check` that cannot obtain the ref fails rather than reporting the pin current. For a kind's integration, the integration decides how to contact its service, and grund decides only whether its complete declaration output is safe to place in the configured home. For a remote, grund contacts the declared source itself: it resolves the declared ref, reads that commit's tracked tree from git objects, runs nothing the remote supplies (no hook, checkout filter, script or integration), and installs the projection only when it is complete and loadable.
