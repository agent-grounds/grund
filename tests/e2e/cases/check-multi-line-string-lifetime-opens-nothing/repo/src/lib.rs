/// Session wiring, tied to §api/FS-session.
fn first<'a>(x: &'a str) -> &'a str { x }
// §api/FS-gone-after-lifetime
fn quote() -> char { 'x' }
