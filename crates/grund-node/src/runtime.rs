//! Request guards, pool ownership and environment cleanup (§FS-distribution.3.2.3.1).
use napi::Env;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once, Weak};

pub static JOBS: AtomicUsize = AtomicUsize::new(0);
pub static ENVIRONMENTS: AtomicUsize = AtomicUsize::new(0);
pub static WRITER: AtomicBool = AtomicBool::new(false);
static HOOK: Once = Once::new();
static POOL: Mutex<Weak<rayon::ThreadPool>> = Mutex::new(Weak::new());
thread_local! {
    static GUARDED:Cell<bool>=const {Cell::new(false)};
    static CONTEXTS:RefCell<BTreeMap<usize,Arc<Context>>>=const {RefCell::new(BTreeMap::new())};
}

pub struct Guard(bool);
impl Guard {
    pub fn enter() -> Self {
        Self(GUARDED.with(|v| v.replace(true)))
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        GUARDED.with(|v| v.set(self.0));
    }
}
pub fn install_hook() {
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !GUARDED.with(Cell::get) {
                previous(info);
            }
        }));
    });
}
pub struct Context {
    pub pool: Arc<rayon::ThreadPool>,
    pub accepting: AtomicBool,
}
impl Drop for Context {
    fn drop(&mut self) {
        ENVIRONMENTS.fetch_sub(1, Ordering::SeqCst);
    }
}
pub fn context(env: &Env) -> napi::Result<Arc<Context>> {
    let key = env.raw() as usize;
    CONTEXTS.with(|contexts| {
        if let Some(c) = contexts.borrow().get(&key) {
            return Ok(c.clone());
        }
        let pool = {
            let mut weak = POOL.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(pool) = weak.upgrade() {
                pool
            } else {
                let pool = Arc::new(
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(
                            std::thread::available_parallelism()
                                .map_or(2, |n| n.get())
                                .min(8),
                        )
                        .start_handler(|_| GUARDED.with(|v| v.set(true)))
                        .exit_handler(|_| GUARDED.with(|v| v.set(false)))
                        .build()
                        .map_err(|e| napi::Error::from_reason(e.to_string()))?,
                );
                *weak = Arc::downgrade(&pool);
                pool
            }
        };
        let context = Arc::new(Context {
            pool,
            accepting: AtomicBool::new(true),
        });
        ENVIRONMENTS.fetch_add(1, Ordering::SeqCst);
        let weak = Arc::downgrade(&context);
        env.add_env_cleanup_hook(key, move |key| {
            if let Some(c) = weak.upgrade() {
                c.accepting.store(false, Ordering::SeqCst);
            }
            CONTEXTS.with(|contexts| contexts.borrow_mut().remove(&key));
        })?;
        contexts.borrow_mut().insert(key, context.clone());
        Ok(context)
    })
}
pub struct WriterLease;
impl WriterLease {
    pub fn acquire() -> Option<Self> {
        WRITER
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .ok()
            .map(|_| Self)
    }
}
impl Drop for WriterLease {
    fn drop(&mut self) {
        WRITER.store(false, Ordering::SeqCst);
    }
}
