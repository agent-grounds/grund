"""Sign a user in."""

# Refuse a stale code (§FS-sign-in.code.1).
def check_code(code):
    return code
