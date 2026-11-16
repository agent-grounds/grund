# Architecture

The diagram renders on GitHub and in editor previews, and each label cites the
component it draws.

```mermaid
flowchart LR
  gateway["gateway §AR-gateway"] --> store["store §AR-store"]
  store --> cache["cache §AR-cache"]
  note["was <§>AR-db before the rename"]
```

A fence the configuration does not name is still an illustration:

```text
gateway §AR-cache
```
