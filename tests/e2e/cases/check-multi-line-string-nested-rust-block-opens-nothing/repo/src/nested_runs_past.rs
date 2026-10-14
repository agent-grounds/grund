/// Session wiring, tied to §api/FS-session.
fn f() {} /* outer comment
  see g() /* nested */ and the
  "quoted word */
// §api/FS-gone-after-nested-runs-past
fn main() { println!("x"); }
