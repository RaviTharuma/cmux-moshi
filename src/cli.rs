//! Command-line interface.

use crate::cleanup;
use crate::dashboard;
use crate::doctor;
use crate::error::Error;
use crate::host::{self, Host, RealHost};
use crate::launchagent;
use crate::mapping;
use crate::shell;
use crate::sync::{self, SyncOptions};
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

/// Moshi host integration: friendly workspace titles on tmux sessions.
#[derive(Debug, Parser)]
#[command(
    name = "cmux-moshi",
    version,
    about = "Moshi host integration: friendly workspace titles on tmux sessions",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check cmux, tmux, optional mosh, RPC reachability, and MOSHI_CLIENT.
    Doctor,
    /// List live tmux session ↔ workspace id ↔ title mapping.
    List {
        /// Emit JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
    /// Rename default ttys* sessions to friendly cmux titles.
    #[command(alias = "rename")]
    Sync {
        /// Also rename already-named sessions.
        #[arg(long)]
        force: bool,
        /// Print the plan only.
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactive menu for Moshi login shells.
    Dashboard {
        /// Run even when MOSHI_CLIENT is unset.
        #[arg(long)]
        force: bool,
    },
    /// Kill idle ttys* sessions; keep panes running claude/node/python/etc.
    Cleanup {
        /// Print the plan only.
        #[arg(long)]
        dry_run: bool,
    },
    /// Install a ~/.zshrc snippet that execs the dashboard when MOSHI_CLIENT=1.
    #[command(name = "install-shell")]
    InstallShell {
        /// rc file to edit (tests and custom ZDOTDIR). Defaults to $ZDOTDIR/.zshrc or $HOME/.zshrc.
        #[arg(long)]
        rc_file: Option<PathBuf>,
    },
    /// Remove the login-shell snippet.
    #[command(name = "uninstall-shell")]
    UninstallShell {
        /// rc file to edit.
        #[arg(long)]
        rc_file: Option<PathBuf>,
    },
    /// Install the macOS LaunchAgent that runs `sync` every five minutes.
    #[command(name = "install-launchagent")]
    InstallLaunchAgent {
        /// LaunchAgents directory (tests / custom HOME). Defaults to $HOME/Library/LaunchAgents.
        #[arg(long)]
        agents_dir: Option<PathBuf>,
    },
    /// Unload and remove the macOS LaunchAgent plist.
    #[command(name = "uninstall-launchagent")]
    UninstallLaunchAgent {
        /// LaunchAgents directory.
        #[arg(long)]
        agents_dir: Option<PathBuf>,
    },
}

/// Parses argv and runs the selected command against `host`.
pub fn run_with_host<I, T>(args: I, host: &dyn Host) -> Result<i32, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    match cli.command {
        None => {
            let mut cmd = Cli::command_help();
            let _ = cmd.print_help();
            Ok(0)
        }
        Some(Command::Doctor) => {
            let checks = doctor::run(host);
            let (report, code) = doctor::format_report(&checks);
            print!("{report}");
            Ok(code)
        }
        Some(Command::List { json }) => {
            let snapshot = mapping::collect(host)?;
            if json {
                println!("{}", mapping::format_json(&snapshot.sessions)?);
            } else {
                println!("{}", mapping::format_table(&snapshot.sessions));
            }
            Ok(0)
        }
        Some(Command::Sync { force, dry_run }) => {
            let plans = sync::run(host, SyncOptions { force, dry_run })?;
            println!("{}", sync::format_report(&plans, dry_run));
            Ok(0)
        }
        Some(Command::Dashboard { force }) => {
            let stdin = io::stdin();
            let mut input = stdin.lock();
            let mut output = io::stdout();
            dashboard::run(host, force, &mut input, &mut output)?;
            Ok(0)
        }
        Some(Command::Cleanup { dry_run }) => {
            let plans = cleanup::run(host, dry_run)?;
            println!("{}", cleanup::format_report(&plans, dry_run));
            Ok(0)
        }
        Some(Command::InstallShell { rc_file }) => {
            let path = resolve_rc(host, rc_file)?;
            let change = shell::install(&path, SystemTime::now())?;
            print_shell_change(&change)?;
            Ok(0)
        }
        Some(Command::UninstallShell { rc_file }) => {
            let path = resolve_rc(host, rc_file)?;
            let change = shell::uninstall(&path, SystemTime::now())?;
            print_shell_change(&change)?;
            Ok(0)
        }
        Some(Command::InstallLaunchAgent { agents_dir }) => {
            let path = resolve_agents_dir(host, agents_dir)?;
            let change = launchagent::install(host, &path)?;
            println!("{}", change.message);
            io::stdout().flush()?;
            Ok(0)
        }
        Some(Command::UninstallLaunchAgent { agents_dir }) => {
            let path = resolve_agents_dir(host, agents_dir)?;
            let change = launchagent::uninstall(host, &path)?;
            println!("{}", change.message);
            io::stdout().flush()?;
            Ok(0)
        }
    }
}

/// Entry used by the binary (real host).
pub fn run<I, T>(args: I) -> Result<i32, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run_with_host(args, &RealHost)
}

fn resolve_rc(host: &dyn Host, rc_file: Option<PathBuf>) -> Result<PathBuf, Error> {
    if let Some(path) = rc_file {
        return Ok(path);
    }
    Ok(host::default_zshrc_path(&|key| host.env(key)))
}

fn resolve_agents_dir(host: &dyn Host, agents_dir: Option<PathBuf>) -> Result<PathBuf, Error> {
    if let Some(path) = agents_dir {
        return Ok(path);
    }
    Ok(host::default_launch_agents_dir(&|key| host.env(key)))
}

fn print_shell_change(change: &shell::ShellChange) -> Result<(), Error> {
    println!("{}", change.message);
    if let Some(backup) = &change.backup_path {
        println!("backup: {}", backup.display());
    }
    io::stdout().flush()?;
    Ok(())
}

impl Cli {
    fn command_help() -> clap::Command {
        <Self as clap::CommandFactory>::command()
    }
}
