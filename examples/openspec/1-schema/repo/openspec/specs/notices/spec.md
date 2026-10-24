# SPEC-notices: Notices Specification

## purpose: Purpose

Which messages a member receives about their loans, and on which days the
system sends them.

## requirements: Requirements

### .due-reminder: Requirement: Due Reminder

The system SHALL email a member two days before a loan falls due. The due date
is the one checkout set, [§SPEC-loans.requirements.borrow-a-book](../loans/spec.md#borrow-a-book-requirement-borrow-a-book), as moved by
any renewal, [§SPEC-loans.requirements.renew-a-loan](../loans/spec.md#renew-a-loan-requirement-renew-a-loan).

#### .two-days-before: Scenario: Two days before the due date

- **GIVEN** a loan due on 22 March
- **WHEN** the reminder job runs on 20 March
- **THEN** the member is emailed a reminder naming the book and the due date

### .overdue-notice: Requirement: Overdue Notice

The system SHALL email a member on the first day a loan is overdue, and every
seven days after that until the copy is returned.

#### .first-day-overdue: Scenario: First day overdue

- **GIVEN** a loan due on 22 March
- **WHEN** the notice job runs on 23 March
- **THEN** the member is emailed an overdue notice

#### .weekly-after: Scenario: Weekly after that

- **GIVEN** a loan due on 22 March that is still out
- **WHEN** the notice job runs on 30 March
- **THEN** the member is emailed a second overdue notice
