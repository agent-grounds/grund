# Pixelbin Constitution

Pixelbin is a self-hosted photo library: a Python package with a command line in front of it.

## Core Principles

### ART-library-first: I. Library-First

Every feature starts as a module of the `pixelbin` package that can be imported and tested
without the command line. A module exists to serve a feature, never only to organize others.

### ART-cli-interface: II. CLI Interface

Every library feature is reachable from the `pixelbin` command: arguments in, results on
stdout, and errors on stderr with a non-zero exit.

### ART-test-first: III. Test-First (NON-NEGOTIABLE)

Tests are written, and seen to fail, before the code they test. Every functional requirement
has a test that cites it.

### ART-privacy-by-default: IV. Privacy by Default

Nothing a user uploads leaves the instance unless its owner shares it, and location data is
removed before a photo is stored.

## Additional Constraints

Python 3.12, the standard library and Pillow. No network service is called at run time.

## Development Workflow

Every feature goes through `/speckit.specify`, `/speckit.plan` and `/speckit.tasks`, in that
order, on its own branch. `grund check` runs on every pull request.

## Governance

This constitution supersedes every other practice. An amendment bumps the version and
re-checks every plan. Each plan's Constitution Check weighs every principle by name, as [§RULE-constitution-gate](rules.md#rule-constitution-gate-the-constitution-check-chapter-of-each-plan-must-cite-each-art-at-least-once) holds it to.

**Version**: 1.1.0 | **Ratified**: 2026-09-14 | **Last Amended**: 2026-10-01
