//! Feature-gated failures immediately after actual owned writes (§FS-distribution.3.2.3.1).
use std::cell::Cell;
thread_local! {static REMAINING:Cell<Option<usize>>=const {Cell::new(None)};}
pub struct BindingWriteFaultGuard(Option<usize>);
pub fn binding_write_fault(count: Option<usize>) -> BindingWriteFaultGuard {
    BindingWriteFaultGuard(REMAINING.with(|v| v.replace(count)))
}
impl Drop for BindingWriteFaultGuard {
    fn drop(&mut self) {
        REMAINING.with(|v| v.set(self.0));
    }
}
pub(crate) fn after_write() -> std::io::Result<()> {
    let fail = REMAINING.with(|v| match v.get() {
        Some(0 | 1) => {
            v.set(None);
            true
        }
        Some(n) => {
            v.set(Some(n - 1));
            false
        }
        None => false,
    });
    if fail {
        Err(std::io::Error::other("injected failure after owned write"))
    } else {
        Ok(())
    }
}
