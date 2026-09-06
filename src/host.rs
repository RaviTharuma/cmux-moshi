//! Host command and environment access.

use crate::error::Error;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Captured subprocess output.
#[derive(Clone, Debug, Default)]
pub struct CommandOutput {
    /// Process exit status.
    pub status: i32,
    /// UTF-8 stdout (lossy).
    pub stdout: String,
    /// UTF-8 stderr (lossy).
    pub stderr: String,
}

impl CommandOutput {
    /// True when the process exited 0.
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

/// Abstraction over PATH lookups, subprocesses, and process inspection.
pub trait Host {
    /// Looks up a binary on PATH.
    fn which(&self, name: &str) -> bool;

    /// Reads an environment variable.
    fn env(&self, key: &str) -> Option<String>;

    /// Runs `program` with `args` and captures output.
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, Error>;

    /// Runs `program` inheriting stdin/stdout/stderr (attach / interactive shell).
    fn run_inherit(&self, program: &str, args: &[&str]) -> Result<i32, Error>;

    /// Reads the environment of `pid`.
    fn read_environ(&self, pid: u32) -> Result<HashMap<String, String>, Error>;

    /// Returns the parent pid of `pid`, if known.
    fn parent_pid(&self, pid: u32) -> Result<Option<u32>, Error>;

    /// Returns child pids of `pid`.
    fn child_pids(&self, pid: u32) -> Result<Vec<u32>, Error>;

    /// Returns the command name (comm) of `pid`.
    fn process_comm(&self, pid: u32) -> Result<String, Error>;
}

/// Live host that shells out to local binaries.
#[derive(Debug, Default, Clone)]
pub struct RealHost;

impl Host for RealHost {
    fn which(&self, name: &str) -> bool {
        which_on_path(name, std::env::var_os("PATH").as_deref())
    }

    fn env(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, Error> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|err| map_spawn_error(program, err))?;
        Ok(CommandOutput {
            status: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    fn run_inherit(&self, program: &str, args: &[&str]) -> Result<i32, Error> {
        let status = Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|err| map_spawn_error(program, err))?;
        Ok(status.code().unwrap_or(1))
    }

    fn read_environ(&self, pid: u32) -> Result<HashMap<String, String>, Error> {
        if let Some(map) = read_linux_environ(pid) {
            return Ok(map);
        }
        let output = self.run("ps", &["eww", "-p", &pid.to_string(), "-o", "command="])?;
        Ok(crate::procenv::parse_ps_eww_env(&output.stdout))
    }

    fn parent_pid(&self, pid: u32) -> Result<Option<u32>, Error> {
        if let Some(ppid) = read_linux_ppid(pid) {
            return Ok(ppid);
        }
        let output = self.run("ps", &["-o", "ppid=", "-p", &pid.to_string()])?;
        Ok(parse_single_pid(&output.stdout))
    }

    fn child_pids(&self, pid: u32) -> Result<Vec<u32>, Error> {
        if let Some(children) = read_linux_children(pid) {
            return Ok(children);
        }
        let output = self.run("pgrep", &["-P", &pid.to_string()])?;
        Ok(output
            .stdout
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect())
    }

    fn process_comm(&self, pid: u32) -> Result<String, Error> {
        if let Some(comm) = read_linux_comm(pid) {
            return Ok(comm);
        }
        let output = self.run("ps", &["-o", "comm=", "-p", &pid.to_string()])?;
        Ok(output.stdout.trim().to_string())
    }
}

fn map_spawn_error(program: &str, err: io::Error) -> Error {
    if err.kind() == io::ErrorKind::NotFound {
        Error::command(program, "not found on PATH")
    } else {
        Error::command(program, err.to_string())
    }
}

/// PATH lookup used by [`RealHost::which`] and tests.
pub fn which_on_path(name: &str, path: Option<&std::ffi::OsStr>) -> bool {
    let Some(path) = path else {
        return false;
    };
    for dir in std::env::split_paths(path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return true;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&candidate) {
                if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
                    return true;
                }
            }
        }
    }
    false
}

fn read_linux_environ(pid: u32) -> Option<HashMap<String, String>> {
    let raw = std::fs::read(format!("/proc/{pid}/environ")).ok()?;
    Some(crate::procenv::parse_nul_environ(&raw))
}

fn read_linux_ppid(pid: u32) -> Option<Option<u32>> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("PPid:") {
            let parsed = rest.trim().parse::<u32>().ok();
            return Some(parsed.filter(|&p| p != 0));
        }
    }
    None
}

fn read_linux_children(pid: u32) -> Option<Vec<u32>> {
    let task_dir = format!("/proc/{pid}/task");
    let mut children = Vec::new();
    for entry in std::fs::read_dir(task_dir).ok()? {
        let entry = entry.ok()?;
        let children_path = entry.path().join("children");
        if let Ok(text) = std::fs::read_to_string(children_path) {
            children.extend(
                text.split_whitespace()
                    .filter_map(|s| s.parse::<u32>().ok()),
            );
        }
    }
    Some(children)
}

fn read_linux_comm(pid: u32) -> Option<String> {
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    Some(comm.trim().to_string())
}

fn parse_single_pid(text: &str) -> Option<u32> {
    let value = text.split_whitespace().next()?.parse::<u32>().ok()?;
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

/// Default login-shell rc path from the environment (never a hardcoded home).
pub fn default_zshrc_path(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    if let Some(zdot) = env("ZDOTDIR").filter(|s| !s.is_empty()) {
        return PathBuf::from(zdot).join(".zshrc");
    }
    let home = env("HOME").unwrap_or_else(|| ".".to_string());
    PathBuf::from(home).join(".zshrc")
}

/// Programmable host for hermetic tests.
#[derive(Clone, Debug, Default)]
pub struct FakeHost {
    /// Binary names treated as present on PATH.
    pub binaries: Vec<String>,
    /// Environment map.
    pub env: HashMap<String, String>,
    /// Scripted command results keyed by `program` + args.
    pub commands: HashMap<(String, Vec<String>), CommandOutput>,
    /// Default result when a command is not scripted.
    pub default_command: Option<CommandOutput>,
    /// Per-pid environments.
    pub environs: HashMap<u32, HashMap<String, String>>,
    /// Per-pid parent.
    pub parents: HashMap<u32, u32>,
    /// Per-pid children.
    pub children: HashMap<u32, Vec<u32>>,
    /// Per-pid comm.
    pub comms: HashMap<u32, String>,
    /// Recorded inherit invocations.
    pub inherited: Vec<(String, Vec<String>)>,
}

impl FakeHost {
    /// Inserts a successful scripted command.
    pub fn ok(&mut self, program: &str, args: &[&str], stdout: &str) {
        self.commands.insert(
            (
                program.to_string(),
                args.iter().map(|s| (*s).to_string()).collect(),
            ),
            CommandOutput {
                status: 0,
                stdout: stdout.to_string(),
                stderr: String::new(),
            },
        );
    }

    /// Inserts a failed scripted command.
    pub fn fail(&mut self, program: &str, args: &[&str], status: i32, stderr: &str) {
        self.commands.insert(
            (
                program.to_string(),
                args.iter().map(|s| (*s).to_string()).collect(),
            ),
            CommandOutput {
                status,
                stdout: String::new(),
                stderr: stderr.to_string(),
            },
        );
    }
}

impl Host for FakeHost {
    fn which(&self, name: &str) -> bool {
        self.binaries.iter().any(|b| b == name)
    }

    fn env(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned()
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, Error> {
        let key = (
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        );
        if let Some(out) = self.commands.get(&key) {
            return Ok(out.clone());
        }
        if let Some(out) = &self.default_command {
            return Ok(out.clone());
        }
        Err(Error::command(program, "not scripted in FakeHost"))
    }

    fn run_inherit(&self, program: &str, args: &[&str]) -> Result<i32, Error> {
        let _ = program;
        let _ = args;
        // FakeHost is used from tests via `&mut` wrappers; inherit is recorded
        // by [`RecordingHost`].
        Ok(0)
    }

    fn read_environ(&self, pid: u32) -> Result<HashMap<String, String>, Error> {
        Ok(self.environs.get(&pid).cloned().unwrap_or_default())
    }

    fn parent_pid(&self, pid: u32) -> Result<Option<u32>, Error> {
        Ok(self.parents.get(&pid).copied())
    }

    fn child_pids(&self, pid: u32) -> Result<Vec<u32>, Error> {
        Ok(self.children.get(&pid).cloned().unwrap_or_default())
    }

    fn process_comm(&self, pid: u32) -> Result<String, Error> {
        Ok(self
            .comms
            .get(&pid)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()))
    }
}

/// Host wrapper that records inherit/run calls for dashboard tests.
pub struct RecordingHost<'a, H: Host> {
    /// Inner host.
    pub inner: &'a H,
    /// Inherited program invocations.
    pub inherited: std::cell::RefCell<Vec<(String, Vec<String>)>>,
}

impl<'a, H: Host> RecordingHost<'a, H> {
    /// Wraps `inner`.
    pub fn new(inner: &'a H) -> Self {
        Self {
            inner,
            inherited: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl<H: Host> Host for RecordingHost<'_, H> {
    fn which(&self, name: &str) -> bool {
        self.inner.which(name)
    }

    fn env(&self, key: &str) -> Option<String> {
        self.inner.env(key)
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, Error> {
        self.inner.run(program, args)
    }

    fn run_inherit(&self, program: &str, args: &[&str]) -> Result<i32, Error> {
        self.inherited.borrow_mut().push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        Ok(0)
    }

    fn read_environ(&self, pid: u32) -> Result<HashMap<String, String>, Error> {
        self.inner.read_environ(pid)
    }

    fn parent_pid(&self, pid: u32) -> Result<Option<u32>, Error> {
        self.inner.parent_pid(pid)
    }

    fn child_pids(&self, pid: u32) -> Result<Vec<u32>, Error> {
        self.inner.child_pids(pid)
    }

    fn process_comm(&self, pid: u32) -> Result<String, Error> {
        self.inner.process_comm(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zshrc_uses_zdotdir_then_home() {
        let env = |key: &str| match key {
            "ZDOTDIR" => Some("/tmp/cmux-moshi-zdot".to_string()),
            "HOME" => Some("/tmp/cmux-moshi-home".to_string()),
            _ => None,
        };
        assert_eq!(
            default_zshrc_path(&env),
            PathBuf::from("/tmp/cmux-moshi-zdot/.zshrc")
        );
        let env = |key: &str| match key {
            "HOME" => Some("/tmp/cmux-moshi-home".to_string()),
            _ => None,
        };
        assert_eq!(
            default_zshrc_path(&env),
            PathBuf::from("/tmp/cmux-moshi-home/.zshrc")
        );
    }
}
