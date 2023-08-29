#[cfg(feature = "log")]
fn main() {
    atomic_dbg::log::init();
    log::trace!("This is an TRACE log message!");
    log::debug!("This is an DEBUG log message!");
    log::info!("This is an INFO log message!");
    log::warn!("This is an WARN log message!");
    log::error!("This is an ERROR log message!");
}

#[cfg(not(feature = "log"))]
fn main() -> Result<(), &'static str> {
    Err("This example requires --features=log.")
}
