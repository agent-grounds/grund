//! A server process finishes before its fixture root is removed, also when a
//! client assertion unwinds (§FS-repository-maintenance.4.1, §AR-ci.10.4).

use std::io;
use std::ops::{Deref, DerefMut};
use std::process::{Child as ProcessChild, Output};

pub struct Child(Option<ProcessChild>);

impl Child {
    pub fn new(child: ProcessChild) -> Self {
        Self(Some(child))
    }

    pub fn wait_with_output(mut self) -> io::Result<Output> {
        self.0.take().expect("live child").wait_with_output()
    }
}

impl Deref for Child {
    type Target = ProcessChild;

    fn deref(&self) -> &ProcessChild {
        self.0.as_ref().expect("live child")
    }
}

impl DerefMut for Child {
    fn deref_mut(&mut self) -> &mut ProcessChild {
        self.0.as_mut().expect("live child")
    }
}

impl Drop for Child {
    /// Reap even an early-exiting test's server before the root owner drops
    /// (§FS-repository-maintenance.4.1).
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
