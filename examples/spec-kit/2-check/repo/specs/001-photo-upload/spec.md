# SPEC-001-photo-upload: Feature Specification: Photo upload

**Feature Branch**: `001-photo-upload`

**Created**: 2026-10-02

**Status**: Accepted

**Input**: User description: "Let me upload a photo from the command line and get back its ID. Strip
the location first."

## user-scenarios: User Scenarios & Testing *(mandatory)*

### user-scenarios.us1: User Story 1 - Upload a photo (Priority: P1)

A user runs `pixelbin upload beach.jpg` and gets back the new photo's ID, which they use to
open the photo later.

**Why this priority**: Nothing else in Pixelbin works without a stored photo.

**Independent Test**: Upload one JPEG to an empty library and read it back by the printed ID.

**Acceptance Scenarios**:

1. **Given** an empty library, **When** the user uploads a 3 MB JPEG, **Then** the command
   prints a photo ID and exits 0.
2. **Given** a photo taken with location on, **When** it is uploaded, **Then** the stored copy
   carries no GPS tag.

---

### user-scenarios.us2: User Story 2 - Refuse a file that is not a photo (Priority: P2)

A user who uploads the wrong file is told what is wrong with it, and nothing is stored.

**Why this priority**: A library that silently stores a PDF is harder to trust than one that
refuses it.

**Independent Test**: Upload a GIF and a 21 MB JPEG; both are refused and the library stays
empty.

**Acceptance Scenarios**:

1. **Given** a GIF, **When** the user uploads it, **Then** the command names the accepted
   formats and exits 1.

---

### user-scenarios.edge-cases: Edge Cases

- What happens when the same file is uploaded twice? It is stored twice, under two IDs.
- How does the system handle a file cut off mid-image? It is refused as unreadable.

## requirements: Requirements *(mandatory)*

### requirements.functional: Functional Requirements

#### requirements.functional.fr-001: FR-001

System MUST store a JPEG or PNG file of up to 20 MB and return its new photo ID.

#### requirements.functional.fr-002: FR-002

System MUST refuse any other file, and any file over 20 MB, with a message naming the limit it
broke.

#### requirements.functional.fr-003: FR-003

System MUST remove every GPS tag from a photo before storing it, as [§ART-privacy-by-default](../../.specify/memory/constitution.md#art-privacy-by-default-iv-privacy-by-default)
requires.

### requirements.entities: Key Entities *(include if feature involves data)*

- **Photo**: one stored image, with its ID, its media type and its upload time.

## success-criteria: Success Criteria *(mandatory)*

### success-criteria.outcomes: Measurable Outcomes

#### success-criteria.outcomes.sc-001: SC-001

A 20 MB photo is stored, and its ID printed, in under 2 seconds on the reference machine.

#### success-criteria.outcomes.sc-002: SC-002

No stored photo carries a GPS tag.

## assumptions: Assumptions

- One person owns a library; accounts come later.
- The reference machine is the smallest instance the project documents.
