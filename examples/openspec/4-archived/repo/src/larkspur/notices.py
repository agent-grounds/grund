"""The emails a member gets about their loans, as §SPEC-notices specifies."""

from datetime import date, timedelta

REMINDER_DAYS = 2
OVERDUE_EVERY = 7


def due_reminder(due: date, today: date) -> bool:
    """Whether today's job emails a reminder: §SPEC-notices.requirements.due-reminder."""
    return today == due - timedelta(days=REMINDER_DAYS)


def overdue_notice(due: date, today: date) -> bool:
    """Whether today's job emails a notice: §SPEC-notices.requirements.overdue-notice."""
    late = (today - due).days
    return late >= 1 and (late - 1) % OVERDUE_EVERY == 0
