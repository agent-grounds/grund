# Normalizes sign-in input (§FS-login)


def normalize_email(raw: str) -> str:
    # Trim, then lower-case (§FS-login.format.1)
    return raw.strip().lower()


def normalize_password(raw: str) -> str:
    # Never trimmed (§FS-login.format.2)
    return raw
