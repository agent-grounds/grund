/// Session wiring, tied to §api/FS-session.
fn is_quote(s: &'static str) -> bool { s.contains('"') }
// §api/FS-gone-after-char-quote
fn main() { println!("x"); }
