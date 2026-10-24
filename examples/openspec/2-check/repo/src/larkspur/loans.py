"""Loans and holds at the circulation desk, as §SPEC-loans specifies."""

from dataclasses import dataclass, field
from datetime import date, timedelta

LOAN_DAYS = 21
MAX_LOANS = 5
MAX_RENEWALS = 2


@dataclass
class Copy:
    barcode: str
    on_loan: bool = False
    holds: list[str] = field(default_factory=list)


@dataclass
class Loan:
    member: str
    copy: Copy
    due: date
    renewals: int = 0


class Refused(Exception):
    """The desk's answer when a request breaks a requirement."""


def borrow(member: str, copy: Copy, loans: list[Loan], today: date) -> Loan:
    """Record a loan, per §SPEC-loans.requirements.borrow-a-book.

    The cap is §SPEC-loans.requirements.loan-limit.
    """
    if copy.on_loan:
        raise Refused("copy is on loan")
    if sum(1 for loan in loans if loan.member == member) >= MAX_LOANS:
        raise Refused("loan limit reached")
    copy.on_loan = True
    return Loan(member, copy, today + timedelta(days=LOAN_DAYS))


def renew(loan: Loan) -> None:
    """Move the due date, per §DELTA-add-loan-holds-loans.modified.renew-a-loan."""
    if loan.renewals >= MAX_RENEWALS:
        raise Refused("renewal limit reached")
    if any(member != loan.member for member in loan.copy.holds):
        raise Refused("copy is on hold")
    loan.renewals += 1
    loan.due += timedelta(days=LOAN_DAYS)


def place_hold(member: str, copy: Copy) -> None:
    """Queue a member for a copy, per §DELTA-add-loan-holds-loans.added.place-a-hold."""
    if not copy.on_loan:
        raise Refused("copy is on the shelf")
    copy.holds.append(member)
