# SPEC-loans: Loans

## requirements: Requirements

### .check-out-a-book: Requirement: Check Out a Book

The system SHALL record a loan due 21 days after checkout when a member borrows
a copy that is on the shelf.

#### .copy-on-shelf: Scenario: Copy on the shelf

- **GIVEN** a copy that is on the shelf
- **WHEN** a member borrows it on 1 March
- **THEN** a loan is recorded, due on 22 March

### .loan-limit: Requirement: Loan Limit

The system MUST refuse a loan that would give a member more than five loans at
once.
