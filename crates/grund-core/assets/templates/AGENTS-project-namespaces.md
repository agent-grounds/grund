### Project namespaces

A namespace is a project boundary, not a docs folder. The current project is the local namespace: cite its IDs as `{MARKER}<ID>`.

Create or use a separate namespace when work introduces an independently checked app, package, service, or subproject. Give that project its own `grund.toml`, add it to the workspace root's `[workspace] members`, run `grund init` there, and set a stable `project_name`.

Do not create a namespace for a regular module or component that still belongs to this project. Cite across namespaces as `{MARKER}alias/<ID>` and run `grund check` from the workspace root.
