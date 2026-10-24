# Tasks

## 1. Hold queue

- [x] 1.1 Keep a hold queue per copy and let a member join it while the copy
  is on loan ([§DELTA-add-loan-holds-loans.added.place-a-hold](specs/loans/spec.md#place-a-hold-requirement-place-a-hold))
- [ ] 1.2 Keep a returned copy for the first member in its queue for 3 days
  ([§DELTA-add-loan-holds-loans.added.place-a-hold.copy-returned](specs/loans/spec.md#copy-returned-scenario-held-copy-returned))
- [x] 1.3 Test joining a hold queue in `tests/test_loans.py`

## 2. Renewals

- [x] 2.1 Refuse a renewal while another member holds the copy
  ([§DELTA-add-loan-holds-loans.modified.renew-a-loan](specs/loans/spec.md#renew-a-loan-requirement-renew-a-loan))
- [x] 2.2 Test the refusal in `tests/test_loans.py`

## 3. Rename

- [ ] 3.1 Rename `borrow` to `check_out` in `src/larkspur/loans.py`
  ([§SPEC-loans.requirements.borrow-a-book](../../specs/loans/spec.md#borrow-a-book-requirement-borrow-a-book))
