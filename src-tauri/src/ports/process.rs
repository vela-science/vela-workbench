use std::{
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use thiserror::Error;
use wait_timeout::ChildExt;

const READ_CHUNK: usize = 8 * 1024;

#[derive(Debug, Error)]
pub(crate) enum PortError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Process(String),
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    Unsupported(String),
}

impl PortError {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "invalid_input",
            Self::Unavailable(_) => "unavailable",
            Self::Process(_) => "process",
            Self::Parse(_) => "parse",
            Self::Unsupported(_) => "unsupported",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProcessSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub max_stdout: usize,
    pub max_stderr: usize,
}

impl ProcessSpec {
    pub(crate) fn new(program: impl Into<PathBuf>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: cwd.into(),
            timeout: Duration::from_secs(8),
            max_stdout: 2 * 1024 * 1024,
            max_stderr: 256 * 1024,
        }
    }

    pub(crate) fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args = args
            .into_iter()
            .map(|arg| arg.as_ref().to_os_string())
            .collect();
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProcessOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

fn explicit_environment(command: &mut Command) {
    command.env_clear();
    for key in ["HOME", "TMPDIR", "LANG", "LC_ALL"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    #[cfg(target_os = "macos")]
    command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    #[cfg(not(target_os = "macos"))]
    if let Some(value) = std::env::var_os("PATH") {
        command.env("PATH", value);
    }
    command.env("GIT_OPTIONAL_LOCKS", "0");
    command.env("GIT_TERMINAL_PROMPT", "0");
}

fn drain_bounded<R: Read>(mut reader: R, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut captured = Vec::with_capacity(limit.min(READ_CHUNK));
    let mut chunk = [0_u8; READ_CHUNK];
    let mut truncated = false;
    loop {
        let count = reader.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.len());
        if remaining > 0 {
            captured.extend_from_slice(&chunk[..count.min(remaining)]);
        }
        if count > remaining {
            truncated = true;
        }
    }
    Ok((captured, truncated))
}

#[cfg(unix)]
fn terminate_process_group(pid: u32) {
    // SAFETY: the child was placed in a new process group whose id is its pid.
    // SIGKILL has no borrowed memory or pointer preconditions.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

pub(crate) fn run_bounded(spec: ProcessSpec) -> Result<ProcessOutput, PortError> {
    let cwd = std::fs::canonicalize(&spec.cwd).map_err(|error| {
        PortError::InvalidInput(format!(
            "resolve working directory {}: {error}",
            spec.cwd.display()
        ))
    })?;
    if !cwd.is_dir() {
        return Err(PortError::InvalidInput(format!(
            "working directory is not a directory: {}",
            cwd.display()
        )));
    }

    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Isolate every bounded invocation so timeout termination includes
        // descendants that inherited the captured pipes.
        command.process_group(0);
    }
    explicit_environment(&mut command);

    let mut child = command.spawn().map_err(|error| {
        PortError::Unavailable(format!("start {}: {error}", spec.program.display()))
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PortError::Process("child stdout was unavailable".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| PortError::Process("child stderr was unavailable".into()))?;
    let stdout_limit = spec.max_stdout;
    let stderr_limit = spec.max_stderr;
    let stdout_reader = thread::spawn(move || drain_bounded(stdout, stdout_limit));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, stderr_limit));

    let status = match child
        .wait_timeout(spec.timeout)
        .map_err(|error| PortError::Process(format!("wait for child: {error}")))?
    {
        Some(status) => status,
        None => {
            #[cfg(unix)]
            terminate_process_group(child.id());
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(PortError::Process(format!(
                "child exceeded {} ms and was terminated",
                spec.timeout.as_millis()
            )));
        }
    };

    // A well-behaved bounded read command leaves no background process. Kill
    // any descendant that inherited our pipes before joining the drainers.
    #[cfg(unix)]
    terminate_process_group(child.id());

    let (stdout, stdout_truncated) = stdout_reader
        .join()
        .map_err(|_| PortError::Process("stdout reader panicked".into()))?
        .map_err(|error| PortError::Process(format!("read child stdout: {error}")))?;
    let (stderr, stderr_truncated) = stderr_reader
        .join()
        .map_err(|_| PortError::Process("stderr reader panicked".into()))?
        .map_err(|error| PortError::Process(format!("read child stderr: {error}")))?;

    Ok(ProcessOutput {
        success: status.success(),
        exit_code: status.code(),
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

pub(crate) fn utf8(bytes: &[u8], label: &str) -> Result<String, PortError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| PortError::Parse(format!("{label} was not valid UTF-8")))
}

pub(crate) fn ensure_not_truncated(output: &ProcessOutput, command: &str) -> Result<(), PortError> {
    if output.stdout_truncated || output.stderr_truncated {
        return Err(PortError::Process(format!(
            "{command} exceeded the bounded output limit"
        )));
    }
    Ok(())
}

pub(crate) fn canonical_directory(path: &Path) -> Result<PathBuf, PortError> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| PortError::InvalidInput(format!("resolve {}: {error}", path.display())))?;
    if !canonical.is_dir() {
        return Err(PortError::InvalidInput(format!(
            "not a directory: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{ProcessSpec, run_bounded};

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_descendants_that_hold_output_pipes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut spec = ProcessSpec::new("/bin/sh", temp.path()).args(["-c", "sleep 30 & wait"]);
        spec.timeout = Duration::from_millis(100);
        let started = Instant::now();
        let error = run_bounded(spec).expect_err("process group must time out");
        assert!(error.to_string().contains("terminated"));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[cfg(unix)]
    #[test]
    fn captured_output_is_bounded_while_the_pipe_is_drained() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut spec = ProcessSpec::new("/bin/sh", temp.path()).args([
            "-c",
            "i=0; while [ $i -lt 4096 ]; do printf x; i=$((i+1)); done",
        ]);
        spec.max_stdout = 128;
        let output = run_bounded(spec).expect("bounded process completes");
        assert_eq!(output.stdout.len(), 128);
        assert!(output.stdout_truncated);
    }

    #[cfg(unix)]
    #[test]
    fn successful_parent_cannot_leave_a_pipe_holding_descendant() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut spec = ProcessSpec::new("/bin/sh", temp.path()).args(["-c", "sleep 30 & exit 0"]);
        spec.timeout = Duration::from_secs(2);
        let started = Instant::now();
        let output = run_bounded(spec).expect("parent exits successfully");
        assert!(output.success);
        assert!(started.elapsed() < Duration::from_secs(3));
    }
}
