use std::{
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
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
    pub path_prefix: Option<PathBuf>,
    environment: Vec<(OsString, OsString)>,
    /// Forward only the standard SSH agent socket path to a command whose
    /// closed contract explicitly requires Repository-authority signing.
    pub include_ssh_auth_sock: bool,
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
            path_prefix: None,
            environment: Vec::new(),
            include_ssh_auth_sock: false,
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

    pub(crate) fn env(mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.environment
            .push((name.as_ref().to_os_string(), value.as_ref().to_os_string()));
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

fn environment_values(
    path_prefix: Option<&Path>,
    include_ssh_auth_sock: bool,
) -> Vec<(OsString, OsString)> {
    let mut values = Vec::new();
    for key in ["HOME", "TMPDIR", "LANG", "LC_ALL"] {
        if let Some(value) = std::env::var_os(key) {
            values.push((OsString::from(key), value));
        }
    }
    #[cfg(target_os = "macos")]
    let base_path = OsString::from("/usr/bin:/bin:/usr/sbin:/sbin");
    #[cfg(not(target_os = "macos"))]
    let base_path = std::env::var_os("PATH").unwrap_or_default();
    let path = if let Some(prefix) = path_prefix {
        let mut value = prefix.as_os_str().to_os_string();
        value.push(":");
        value.push(base_path);
        value
    } else {
        base_path
    };
    values.push((OsString::from("PATH"), path));
    values.push((OsString::from("GIT_OPTIONAL_LOCKS"), OsString::from("0")));
    values.push((OsString::from("GIT_TERMINAL_PROMPT"), OsString::from("0")));
    if include_ssh_auth_sock && let Some(value) = std::env::var_os("SSH_AUTH_SOCK") {
        values.push((OsString::from("SSH_AUTH_SOCK"), value));
    }
    values
}

pub(crate) fn environment_summary(path_prefix: Option<&Path>) -> Vec<(String, String)> {
    environment_values(path_prefix, false)
        .into_iter()
        .map(|(name, value)| {
            (
                name.to_string_lossy().into_owned(),
                value.to_string_lossy().into_owned(),
            )
        })
        .collect()
}

fn explicit_environment(
    command: &mut Command,
    path_prefix: Option<&Path>,
    include_ssh_auth_sock: bool,
    environment: &[(OsString, OsString)],
) {
    command.env_clear();
    for (name, value) in environment_values(path_prefix, include_ssh_auth_sock) {
        command.env(name, value);
    }
    for (name, value) in environment {
        command.env(name, value);
    }
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
    explicit_environment(
        &mut command,
        spec.path_prefix.as_deref(),
        spec.include_ssh_auth_sock,
        &spec.environment,
    );

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

#[derive(Debug, Clone)]
pub(crate) enum CancellableProcessOutput {
    Completed(ProcessOutput),
    Cancelled(ProcessOutput),
    TimedOut(ProcessOutput),
}

fn collect_output(
    status: Option<std::process::ExitStatus>,
    stdout_reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
    stderr_reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<ProcessOutput, PortError> {
    let (stdout, stdout_truncated) = stdout_reader
        .join()
        .map_err(|_| PortError::Process("stdout reader panicked".into()))?
        .map_err(|error| PortError::Process(format!("read child stdout: {error}")))?;
    let (stderr, stderr_truncated) = stderr_reader
        .join()
        .map_err(|_| PortError::Process("stderr reader panicked".into()))?
        .map_err(|error| PortError::Process(format!("read child stderr: {error}")))?;
    Ok(ProcessOutput {
        success: status
            .as_ref()
            .is_some_and(std::process::ExitStatus::success),
        exit_code: status.and_then(|value| value.code()),
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

pub(crate) fn run_cancellable(
    spec: ProcessSpec,
    cancel: Arc<AtomicBool>,
) -> Result<CancellableProcessOutput, PortError> {
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
        command.process_group(0);
    }
    explicit_environment(
        &mut command,
        spec.path_prefix.as_deref(),
        spec.include_ssh_auth_sock,
        &spec.environment,
    );
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
    let started = Instant::now();
    let mut termination = None;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| PortError::Process(format!("poll child: {error}")))?
        {
            break Some(status);
        }
        if cancel.load(Ordering::SeqCst) {
            termination = Some("cancelled");
            break None;
        }
        if started.elapsed() >= spec.timeout {
            termination = Some("timed_out");
            break None;
        }
        thread::sleep(Duration::from_millis(25));
    };
    if status.is_none() {
        #[cfg(unix)]
        terminate_process_group(child.id());
        let _ = child.kill();
        let _ = child.wait();
    } else {
        #[cfg(unix)]
        terminate_process_group(child.id());
    }
    let output = collect_output(status, stdout_reader, stderr_reader)?;
    Ok(match termination {
        Some("cancelled") => CancellableProcessOutput::Cancelled(output),
        Some("timed_out") => CancellableProcessOutput::TimedOut(output),
        _ => CancellableProcessOutput::Completed(output),
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
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::{Duration, Instant},
    };

    use super::{CancellableProcessOutput, ProcessSpec, run_bounded, run_cancellable};

    #[cfg(unix)]
    #[test]
    fn explicit_cancellation_terminates_the_process_group_promptly() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut spec = ProcessSpec::new("/bin/sh", temp.path()).args(["-c", "sleep 30 & wait"]);
        spec.timeout = Duration::from_secs(30);
        let cancellation = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancellation);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            signal.store(true, Ordering::SeqCst);
        });
        let started = Instant::now();
        let result = run_cancellable(spec, cancellation).expect("cancellable process");
        assert!(matches!(result, CancellableProcessOutput::Cancelled(_)));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

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
