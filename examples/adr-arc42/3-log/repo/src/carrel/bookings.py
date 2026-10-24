"""Book, move and cancel seats.

Bookings live in PostgreSQL, shared by both instances (§ADR-0004). The unique
index on (seat, slot) is what turns a second booking of one seat into an error
the caller can answer with another seat (§QS-1).
"""

BOOK = """
    INSERT INTO booking (seat, slot, student)
    VALUES (%(seat)s, %(slot)s, %(student)s)
"""


class SeatTaken(Exception):
    """Another booking already holds this seat for this slot."""


def book(conn, seat, slot, student):
    try:
        with conn.transaction():
            conn.execute(BOOK, {"seat": seat, "slot": slot, "student": student})
    except conn.UniqueViolation as err:
        raise SeatTaken(seat) from err
