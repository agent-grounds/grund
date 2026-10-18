# FS-login: Login

Users sign in with a username and a password.

## requirements: Requirements

### requirements.lockout: Five failures lock the account

Five failed attempts in a row lock the account for fifteen minutes.

### requirements.reset: A reset link expires

A password reset link expires after one hour.

### requirements.audit: Every attempt is recorded

Each attempt, failed or not, is written to the audit log.
