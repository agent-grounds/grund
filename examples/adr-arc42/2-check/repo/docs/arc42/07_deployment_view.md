# Deployment View

## Infrastructure Level 1

| Node | Runs |
|---|---|
| `carrel-app-1`, `carrel-app-2` | the Booking API and the door gateway |
| `carrel-db` | PostgreSQL 16, with a nightly dump to the university's backup store ([§QS-4](10_quality_requirements.md#qs-4-the-database-machine-is-lost-overnight)) |

The release job runs on `carrel-app-1` only.
