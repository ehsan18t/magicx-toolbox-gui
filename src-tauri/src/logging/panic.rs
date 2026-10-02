use std::panic::PanicHookInfo;

/// Main process only. Release builds abort right after the hook, so the ring and the session file
/// are the only record of a panic there.
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Skipped when the panic is inside the logger on this thread: its locks are held here.
        if !super::pipeline::IN_LOGGER.get() {
            super::record_panic(&describe(info));
        }
        if cfg!(debug_assertions) {
            previous(info);
        }
    }));
}

fn describe(info: &PanicHookInfo<'_>) -> String {
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unnamed");
    let msg = info.payload_as_str().unwrap_or("(no message)");
    match info.location() {
        Some(at) => format!(
            "panic in thread '{name}' at {}:{}:{}: {msg}",
            at.file(),
            at.line(),
            at.column()
        ),
        None => format!("panic in thread '{name}': {msg}"),
    }
}
