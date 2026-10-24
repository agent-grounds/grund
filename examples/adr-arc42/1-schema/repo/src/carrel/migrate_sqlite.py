"""Move the pilot's bookings from SQLite into PostgreSQL, once.

The pilot kept them in one SQLite file (§ADR-0002); they now live in
PostgreSQL (§ADR-0004). Run it once, before the second instance starts.
"""

import sqlite3


def migrate(sqlite_path, conn):
    source = sqlite3.connect(sqlite_path)
    rows = source.execute("SELECT seat, slot, student FROM booking").fetchall()
    with conn.transaction():
        for seat, slot, student in rows:
            conn.execute(
                "INSERT INTO booking (seat, slot, student) VALUES (%s, %s, %s)",
                (seat, slot, student),
            )
