# DELTA-add-loan-holds-loans: Loans delta for add-loan-holds

## added: ADDED Requirements

### .place-a-hold: Requirement: Place a Hold

The system SHALL let a member place a hold on a copy that is on loan, and SHALL
keep a returned copy for the first member in its hold queue for 3 days.

#### .copy-on-loan: Scenario: Hold on a copy that is on loan

- **GIVEN** a copy that is on loan to another member
- **WHEN** a member places a hold on it
- **THEN** the member joins the end of the copy's hold queue

#### .copy-returned: Scenario: Held copy returned

- **GIVEN** a copy whose hold queue is not empty
- **WHEN** the copy is returned on 1 March
- **THEN** it is kept for the first member in the queue until 4 March

## modified: MODIFIED Requirements

### .renew-a-loan: Requirement: Renew a Loan

The system SHALL move a loan's due date 21 days later when the member renews
it, SHALL refuse a third renewal of the same loan, and SHALL refuse a renewal
while another member holds the copy. (Previously: only the third renewal was
refused, [§SPEC-loans.requirements.renew-a-loan](../../../../specs/loans/spec.md#renew-a-loan-requirement-renew-a-loan).)

#### .first-renewal: Scenario: First renewal

- **GIVEN** a loan due on 22 March that has never been renewed
- **WHEN** the member renews it
- **THEN** the loan is due on 12 April

#### .third-renewal: Scenario: Third renewal refused

- **GIVEN** a loan that has been renewed twice
- **WHEN** the member renews it again
- **THEN** the system refuses with "renewal limit reached"

#### .held-copy: Scenario: Renewal of a held copy refused

- **GIVEN** a loan whose copy another member holds
- **WHEN** the member renews it
- **THEN** the system refuses with "copy is on hold"

## renamed: RENAMED Requirements

- FROM: `### .borrow-a-book: Requirement: Borrow a Book`
- TO: `### .check-out-a-book: Requirement: Check Out a Book`

The desk calls it a check-out, and with holds "borrow" no longer names one
action. This renames [§SPEC-loans.requirements.borrow-a-book](../../../../specs/loans/spec.md#borrow-a-book-requirement-borrow-a-book).
