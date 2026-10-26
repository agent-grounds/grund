# holds Specification

## Purpose

Who gets a returned copy when more than one member is waiting for it.

## Requirements

### Requirement: Hold Queue Order

The system SHALL offer a returned copy to the member who placed the oldest hold.

#### Scenario: Two holds

- **GIVEN** holds placed by ana on 1 March and ben on 2 March
- **WHEN** the copy is returned
- **THEN** it is kept for ana
