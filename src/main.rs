//! Binary entry for `cmux-moshi`.

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
