//! The application entry point.

/// Logs a user in (§FS-login).
pub fn login(email: &str, password: &str) -> bool {
    !email.is_empty() && !password.is_empty()
}
