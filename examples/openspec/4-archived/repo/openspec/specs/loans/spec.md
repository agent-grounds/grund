# SPEC-loans: Loans Specification

## purpose: Purpose

How a member checks a book out of the branch, how many books they may hold at
once, and how long they may keep each one.

## requirements: Requirements

### .check-out-a-book: Requirement: Check Out a Book

The system SHALL record a loan due 21 days after checkout when a member borrows
a copy that is on the shelf.

#### .copy-on-shelf: Scenario: Copy on the shelf

- **GIVEN** a copy that is on the shelf
- **WHEN** a member borrows it on 1 March
- **THEN** a loan is recorded, due on 22 March
- **AND** the copy is marked as on loan

#### .copy-on-loan: Scenario: Copy already on loan

- **GIVEN** a copy that is on loan to another member
- **WHEN** a member tries to borrow it
- **THEN** the system refuses with "copy is on loan"

### .loan-limit: Requirement: Loan Limit

The system MUST refuse a loan that would give a member more than five loans at
once.

#### .sixth-loan: Scenario: Sixth loan refused

- **GIVEN** a member with five loans
- **WHEN** they try to borrow another copy
- **THEN** the system refuses with "loan limit reached"

### .renew-a-loan: Requirement: Renew a Loan

The system SHALL move a loan's due date 21 days later when the member renews
it, SHALL refuse a third renewal of the same loan, and SHALL refuse a renewal
while another member holds the copy.

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
