"""Release the seats nobody claimed in time.

A booking with no claim 15 minutes into its slot is cancelled (§ADR-0003). The
cancellation and a claim lock the same row, so a claim that arrives first
always wins (§QS-3).
"""

CLAIM_WINDOW_MINUTES = 15

RELEASE = """
    DELETE FROM booking
    WHERE claimed_at IS NULL
      AND slot_start < now() - make_interval(mins => %(window)s)
    RETURNING seat, slot, student
"""


def release_unclaimed(conn):
    with conn.transaction():
        return conn.execute(RELEASE, {"window": CLAIM_WINDOW_MINUTES}).fetchall()
