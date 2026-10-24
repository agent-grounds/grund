# SPEC-002-album-share: Feature Specification: Album share

**Feature Branch**: `002-album-share`

**Created**: 2026-10-08

**Status**: Accepted

**Input**: User description: "Let an owner share an album through a link that stops working after
a while."

## clarifications: Clarifications

### clarifications.session-2026-10-09: Session 2026-10-09

- Q: How long does a share link work? → A: 7 days from when it is created; the owner can revoke
  it sooner.

## user-scenarios: User Scenarios & Testing *(mandatory)*

### user-scenarios.us1: User Story 1 - Share an album by link (Priority: P1)

An owner runs `pixelbin share holiday` and gets a link that anyone can open to see the album.

**Why this priority**: Sharing is the reason most owners make an album at all.

**Independent Test**: Share an album, open the link without signing in, and see its photos.

**Acceptance Scenarios**:

1. **Given** an album with three photos, **When** its owner shares it, **Then** the link shows
   those three photos.
2. **Given** a link created 7 days ago, **When** anyone opens it, **Then** it shows nothing.

---

### user-scenarios.us2: User Story 2 - Revoke a link (Priority: P2)

An owner who shared an album by mistake revokes the link, and it stops working at once.

**Why this priority**: A link sent to the wrong person must not wait out its 7 days.

**Independent Test**: Share an album, revoke the link, and open it.

**Acceptance Scenarios**:

1. **Given** a live link, **When** its owner revokes it, **Then** the next request shows
   nothing.

---

### user-scenarios.edge-cases: Edge Cases

- What happens when a shared album is deleted? Its links stop working.
- How does the system handle a photo added after the link was made? The link shows it.

## requirements: Requirements *(mandatory)*

### requirements.functional: Functional Requirements

#### requirements.functional.fr-001: FR-001

System MUST create a share link for an album when, and only when, its owner asks.

#### requirements.functional.fr-002: FR-002

System MUST stop serving a share link 7 days after it was created.

#### requirements.functional.fr-003: FR-003

System MUST stop serving a share link as soon as its owner revokes it.

#### requirements.functional.fr-004: FR-004

A share link MUST serve the photos of its album and nothing else, as [§ART-privacy-by-default](../../.specify/memory/constitution.md#art-privacy-by-default-iv-privacy-by-default)
requires.

### requirements.entities: Key Entities *(include if feature involves data)*

- **Album**: a named set of the photos [§SPEC-001-photo-upload](../001-photo-upload/spec.md#spec-001-photo-upload-feature-specification-photo-upload) stores, with one owner.
- **Share link**: an unguessable token for one album, with its creation time and whether it is
  revoked.

## success-criteria: Success Criteria *(mandatory)*

### success-criteria.outcomes: Measurable Outcomes

#### success-criteria.outcomes.sc-001: SC-001

A revoked or expired link serves nothing on its very next request.

## assumptions: Assumptions

- Whoever holds a live link may see the album; recipients have no accounts.
