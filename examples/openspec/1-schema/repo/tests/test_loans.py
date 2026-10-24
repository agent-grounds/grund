"""Each test proves the scenario its docstring cites."""

from datetime import date

import pytest

from larkspur.loans import Copy, Loan, Refused, borrow, place_hold, renew


def test_copy_on_shelf():
    """§SPEC-loans.requirements.borrow-a-book.copy-on-shelf"""
    copy = Copy("31842")
    loan = borrow("ana", copy, [], date(2026, 3, 1))
    assert loan.due == date(2026, 3, 22)
    assert copy.on_loan


def test_copy_on_loan():
    """§SPEC-loans.requirements.borrow-a-book.copy-on-loan"""
    with pytest.raises(Refused, match="copy is on loan"):
        borrow("ana", Copy("31842", on_loan=True), [], date(2026, 3, 1))


def test_hold_on_a_copy_that_is_on_loan():
    """§DELTA-add-loan-holds-loans.added.place-a-hold.copy-on-loan"""
    copy = Copy("31842", on_loan=True, holds=["ben"])
    place_hold("ana", copy)
    assert copy.holds == ["ben", "ana"]


def test_renewal_of_a_held_copy_refused():
    """§DELTA-add-loan-holds-loans.modified.renew-a-loan.held-copy"""
    loan = Loan("ana", Copy("31842", on_loan=True, holds=["ben"]), date(2026, 3, 22))
    with pytest.raises(Refused, match="copy is on hold"):
        renew(loan)
