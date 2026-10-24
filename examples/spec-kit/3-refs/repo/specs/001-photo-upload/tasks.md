---
description: "Task list for the photo upload feature"
---

# Tasks: Photo upload

**Input**: Design documents from `/specs/001-photo-upload/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Required, not optional: every story phase opens with its tests ([§ART-test-first](../../.specify/memory/constitution.md#art-test-first-iii-test-first-non-negotiable)).

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create `src/pixelbin/` and `tests/` per [§PLAN-001-photo-upload.project-structure](plan.md#project-structure-project-structure)
- [X] T002 [P] Add Pillow and pytest to `pyproject.toml` per [§PLAN-001-photo-upload.technical-context](plan.md#technical-context-technical-context)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

- [X] T003 Create the Photo entity of [§SPEC-001-photo-upload.requirements.entities](spec.md#requirementsentities-key-entities-include-if-feature-involves-data) in
  src/pixelbin/upload.py

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Upload a photo (Priority: P1) MVP

**Goal**: A JPEG or PNG is stored under a new ID, with its location removed

**Independent Test**: Upload one JPEG to an empty library and read it back by the printed ID

### Tests for User Story 1

- [X] T004 [P] [US1] Test that a JPEG is stored and its ID returned in tests/test_upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-001](spec.md#requirementsfunctionalfr-001-fr-001))
- [X] T005 [P] [US1] Test that GPS tags are removed in tests/test_upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-003](spec.md#requirementsfunctionalfr-003-fr-003))

### Implementation for User Story 1

- [X] T006 [US1] Implement `upload()` in src/pixelbin/upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-001](spec.md#requirementsfunctionalfr-001-fr-001))
- [X] T007 [US1] Implement `strip_gps()` in src/pixelbin/upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-003](spec.md#requirementsfunctionalfr-003-fr-003))
- [ ] T008 [US1] Add `pixelbin upload PATH` to src/pixelbin/cli.py, printing the ID on stdout
  ([§SPEC-001-photo-upload.user-scenarios.us1](spec.md#user-scenariosus1-user-story-1---upload-a-photo-priority-p1))

**Checkpoint**: User Story 1 is fully functional and testable on its own

---

## Phase 4: User Story 2 - Refuse a file that is not a photo (Priority: P2)

**Goal**: The wrong file is refused with the limit it broke, and nothing is stored

**Independent Test**: Upload a GIF and a 21 MB JPEG; both are refused

### Tests for User Story 2

- [X] T009 [P] [US2] Test that a GIF and a 21 MB JPEG are refused in tests/test_upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-002](spec.md#requirementsfunctionalfr-002-fr-002))

### Implementation for User Story 2

- [X] T010 [US2] Raise `Refused` naming the broken limit in src/pixelbin/upload.py
  ([§SPEC-001-photo-upload.requirements.functional.fr-002](spec.md#requirementsfunctionalfr-002-fr-002))

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T011 [P] Time a 20 MB upload on the reference machine
  ([§SPEC-001-photo-upload.success-criteria.outcomes.sc-001](spec.md#success-criteriaoutcomessc-001-sc-001))

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - blocks both stories
- **User Story 2 (P2)**: Raises from the same `upload()` as [§SPEC-001-photo-upload.user-scenarios.us1](spec.md#user-scenariosus1-user-story-1---upload-a-photo-priority-p1),
  so T010 starts after T006
