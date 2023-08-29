//! An extremely minimal log implementation that prints everything to `stderr`
//! and has no configuration.

struct Log;

impl log::Log for Log {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        crate::eprintln!(
            "{}: {} - {}",
            record.metadata().target(),
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&Log).unwrap();
}
