// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // If launched as the elevated effect broker (`--broker <req> <resp>`), run it and exit before
    // any GUI initialization.
    if let Some(code) = app_lib::run_broker_if_requested() {
        std::process::exit(code);
    }
    app_lib::run();
}
