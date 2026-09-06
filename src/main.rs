//! Binary entry for the Moshi host CLI (`cmux-moshi`).
//!
//! Phone clients never receive a mux sidebar PTY. This binary is the
//! integration: doctor, list, sync, dashboard, cleanup, install-shell.

fn main() {
    match cmux_moshi::cli::run(std::env::args_os()) {
        Ok(code) => {
            if code != 0 {
                std::process::exit(code);
            }
        }
        Err(err) => {
            eprintln!("cmux-moshi: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
