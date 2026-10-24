---
description: "Task list for the album share feature"
---

# Tasks: Album share

**Input**: Design documents from `/specs/002-album-share/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Create src/pixelbin/share.py and tests/test_share.py per
  [§PLAN-002-album-share.project-structure](plan.md#project-structure-project-structure)

---

## Phase 2: User Story 1 - Share an album by link (Priority: P1) MVP

**Goal**: An owner gets a link that shows the album for 7 days

### Tests for User Story 1

- [X] T002 [P] [US1] Test that only the owner can share in tests/test_share.py
  ([§SPEC-002-album-share.requirements.functional.fr-001](spec.md#requirementsfunctionalfr-001-fr-001))
- [X] T003 [P] [US1] Test that a 7-day-old link serves nothing in tests/test_share.py
  ([§SPEC-002-album-share.requirements.functional.fr-002](spec.md#requirementsfunctionalfr-002-fr-002))
- [ ] T004 [P] [US1] Test that a link lists its album's photos and no others in tests/test_share.py
  ([§SPEC-002-album-share.requirements.functional.fr-004](spec.md#requirementsfunctionalfr-004-fr-004))

### Implementation for User Story 1

- [X] T005 [US1] Implement `share()` in src/pixelbin/share.py
  ([§SPEC-002-album-share.requirements.functional.fr-001](spec.md#requirementsfunctionalfr-001-fr-001))
- [X] T006 [US1] Implement the lifetime check of `serves()` in src/pixelbin/share.py
  ([§SPEC-002-album-share.requirements.functional.fr-002](spec.md#requirementsfunctionalfr-002-fr-002))
- [ ] T007 [US1] List the album's photos, and no others, for a live link
  ([§SPEC-002-album-share.requirements.functional.fr-004](spec.md#requirementsfunctionalfr-004-fr-004))

---

## Phase 3: User Story 2 - Revoke a link (Priority: P2)

### Tests for User Story 2

- [X] T008 [P] [US2] Test that a revoked link serves nothing in tests/test_share.py
  ([§SPEC-002-album-share.requirements.functional.fr-003](spec.md#requirementsfunctionalfr-003-fr-003))

### Implementation for User Story 2

- [X] T009 [US2] Honour `revoked` in `serves()` in src/pixelbin/share.py
  ([§SPEC-002-album-share.requirements.functional.fr-003](spec.md#requirementsfunctionalfr-003-fr-003))

---

## Dependencies & Execution Order

- **User Story 2 (P2)**: Extends the `serves()` of [§SPEC-002-album-share.user-scenarios.us1](spec.md#user-scenariosus1-user-story-1---share-an-album-by-link-priority-p1), so T009
  starts after T006
