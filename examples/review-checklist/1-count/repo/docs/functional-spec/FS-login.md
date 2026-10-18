# FS-login: Login

Users sign in with a username and a password, and every attempt is kept in §FS-audit.

## requirements: Requirements

### requirements.lockout: Five failures lock the account

Five failed attempts in a row lock the account for fifteen minutes; §FS-audit.entries records it.

### requirements.reset: A reset link expires

A password reset link expires after one hour.

### requirements.audit: Every attempt is recorded

Each attempt, failed or not, is written to the audit log as §FS-audit.entries says.
