use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

type Handler = Box<dyn FnMut() + Send + 'static>;

static CTRL_C_HANDLERS: Mutex<Vec<Handler>> = Mutex::new(Vec::new());
static CTRL_C_INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn register_ctrlc_handler<F>(handler: F) -> Result<(), ctrlc::Error>
where
    F: FnMut() + Send + 'static,
{
    if let Ok(mut guard) = CTRL_C_HANDLERS.lock() {
        guard.push(Box::new(handler));
    }

    if CTRL_C_INSTALLED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
        && let Err(e) = ctrlc::set_handler(|| {
            if let Ok(mut handlers) = CTRL_C_HANDLERS.lock() {
                for handler in handlers.iter_mut() {
                    handler();
                }
            }
        })
    {
        CTRL_C_INSTALLED.store(false, Ordering::SeqCst);
        return Err(e);
    }

    Ok(())
}
