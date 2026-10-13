/// Session wiring, tied to §api/FS-session.
fn broken() { let s = r#"this raw string never closes; }
// §api/FS-gone-after-unclosed-raw-string
fn after() {}
