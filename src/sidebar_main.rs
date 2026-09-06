//! `cmux-moshi-sidebar` — optional workspace picker for mux plugin packaging.
//!
//! Official cmux installs git plugins only through the sidebar plugin
//! channel. This binary is the `[run]` executable that channel verifies.
//! It lists friendly workspace titles and selects one via `cmux-client`.
//! It is not a Moshi UI: phone clients never see this PTY.

use anyhow::Result;
use clap::Parser;
use cmux_moshi::sidebar::{probe_report, ui, Picker};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Write};
use std::time::Duration;

const POLL_EVERY: Duration = Duration::from_millis(100);

#[derive(Debug, Parser)]
#[command(
    name = "cmux-moshi-sidebar",
    version,
    about = "Optional cmux workspace picker (mux sidebar plugin packaging; not a Moshi panel)"
)]
struct Args {
    /// Print socket resolution and exit without opening a TUI.
    #[arg(long)]
    probe: bool,
}

fn main() {
    let args = Args::parse();
    if args.probe {
        let report = probe_report(|key| std::env::var_os(key));
        print!("{report}");
        let _ = io::stdout().flush();
        return;
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    if let Err(err) = run_tui() {
        eprintln!("cmux-moshi-sidebar: {err}");
        std::process::exit(1);
    }
}

fn run_tui() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = event_loop(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut picker = Picker::new();
    picker.connect_or_schedule();

    loop {
        terminal.draw(|frame| ui::draw(frame, &picker.view()))?;
        if event::poll(POLL_EVERY)? {
            if let Event::Key(key) = event::read()? {
                if picker.handle_key(key) {
                    break;
                }
            }
        }
        picker.tick();
    }
    Ok(())
}
