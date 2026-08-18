use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

use crate::contracts::{
    EnvironmentEntryDto, GitSnapshotDto, NativeExecPreviewDto, NativeExecProfileDto,
    NativeExecResultDto, NativeExecStateDto, NativeOutputDto, NativeToolDto,
};

use super::{
    CancellableProcessOutput, PortError, ProcessOutput, ProcessSpec, environment_summary,
    run_cancellable,
};

pub(crate) const TRUST_WARNING: &str = "This profile can execute repository-controlled build scripts or plugins with your current local user privileges. Output, environment, lifetime, and process-tree controls are bounds only; this is not a sandbox or security isolation.";

#[cfg(target_os = "macos")]
const SYSTEM_GIT: &str = "/usr/bin/git";
#[cfg(not(target_os = "macos"))]
const SYSTEM_GIT: &str = "git";

const MAX_STDOUT: usize = 2 * 1024 * 1024;
const MAX_STDERR: usize = 1024 * 1024;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn digest_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn digest_file(path: &Path) -> Result<(String, u64), PortError> {
    let mut file = File::open(path)
        .map_err(|error| PortError::InvalidInput(format!("open native tool: {error}")))?;
    let size = file
        .metadata()
        .map_err(|error| PortError::InvalidInput(format!("inspect native tool: {error}")))?
        .len();
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| PortError::Process(format!("hash native tool: {error}")))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    Ok((
        format!(
            "sha256:{}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ),
        size,
    ))
}

fn expected_basename(profile: NativeExecProfileDto) -> &'static str {
    match profile {
        NativeExecProfileDto::GitDiffCheck => "git",
        NativeExecProfileDto::LeanBuild => "lake",
        NativeExecProfileDto::CargoTest => "cargo",
        NativeExecProfileDto::BunTest => "bun",
    }
}

pub(crate) fn inspect_tool(
    profile: NativeExecProfileDto,
    selected: Option<&Path>,
) -> Result<NativeToolDto, PortError> {
    let requested = match profile {
        NativeExecProfileDto::GitDiffCheck => Path::new(SYSTEM_GIT),
        _ => selected.ok_or_else(|| {
            PortError::InvalidInput("this profile requires an explicitly selected tool".into())
        })?,
    };
    let canonical = std::fs::canonicalize(requested).map_err(|error| {
        PortError::InvalidInput(format!(
            "resolve native tool {}: {error}",
            requested.display()
        ))
    })?;
    let metadata = std::fs::metadata(&canonical)
        .map_err(|error| PortError::InvalidInput(format!("inspect native tool: {error}")))?;
    if !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "selected native tool is not a regular file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(PortError::InvalidInput(
                "selected native tool is not executable".into(),
            ));
        }
    }
    let basename = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| PortError::InvalidInput("native tool name is not UTF-8".into()))?;
    if basename != expected_basename(profile) {
        return Err(PortError::InvalidInput(format!(
            "{} profile requires an executable named {}",
            label(profile),
            expected_basename(profile)
        )));
    }
    let (sha256, size) = digest_file(&canonical)?;
    Ok(NativeToolDto {
        profile,
        path: canonical.display().to_string(),
        sha256,
        size,
    })
}

fn label(profile: NativeExecProfileDto) -> &'static str {
    match profile {
        NativeExecProfileDto::GitDiffCheck => "Git diff check",
        NativeExecProfileDto::LeanBuild => "Lean lake build",
        NativeExecProfileDto::CargoTest => "Cargo test locked",
        NativeExecProfileDto::BunTest => "Bun test",
    }
}

fn args(profile: NativeExecProfileDto) -> Vec<String> {
    match profile {
        NativeExecProfileDto::GitDiffCheck => vec![
            "diff".into(),
            "--no-ext-diff".into(),
            "--no-textconv".into(),
            "--check".into(),
        ],
        NativeExecProfileDto::LeanBuild => vec!["build".into()],
        NativeExecProfileDto::CargoTest => vec!["test".into(), "--locked".into()],
        NativeExecProfileDto::BunTest => vec!["test".into()],
    }
}

fn timeout(profile: NativeExecProfileDto) -> Duration {
    match profile {
        NativeExecProfileDto::GitDiffCheck => Duration::from_secs(30),
        _ => Duration::from_secs(15 * 60),
    }
}

fn require_markers(root: &Path, profile: NativeExecProfileDto) -> Result<(), PortError> {
    let exists = |name: &str| root.join(name).is_file();
    let valid = match profile {
        NativeExecProfileDto::GitDiffCheck => true,
        NativeExecProfileDto::LeanBuild => {
            exists("lean-toolchain") && (exists("lakefile.toml") || exists("lakefile.lean"))
        }
        NativeExecProfileDto::CargoTest => exists("Cargo.toml") && exists("Cargo.lock"),
        NativeExecProfileDto::BunTest => exists("package.json") && exists("bun.lock"),
    };
    if !valid {
        return Err(PortError::Unsupported(format!(
            "selected repository does not contain the reviewed markers for {}",
            label(profile)
        )));
    }
    Ok(())
}

pub(crate) fn preview(
    root: &Path,
    git: &GitSnapshotDto,
    profile: NativeExecProfileDto,
    tool: &NativeToolDto,
) -> Result<NativeExecPreviewDto, PortError> {
    let canonical_root = std::fs::canonicalize(root).map_err(|error| {
        PortError::InvalidInput(format!("resolve native execution root: {error}"))
    })?;
    if canonical_root.display().to_string() != git.root {
        return Err(PortError::InvalidInput(
            "native execution root disagrees with the exact Git snapshot".into(),
        ));
    }
    if tool.profile != profile {
        return Err(PortError::InvalidInput(
            "selected tool belongs to a different execution profile".into(),
        ));
    }
    let rechecked = inspect_tool(profile, Some(Path::new(&tool.path)))?;
    if &rechecked != tool {
        return Err(PortError::Unsupported(
            "selected native tool changed after selection".into(),
        ));
    }
    require_markers(&canonical_root, profile)?;
    let prefix = Path::new(&tool.path).parent();
    let environment = environment_summary(prefix)
        .into_iter()
        .map(|(name, value)| EnvironmentEntryDto { name, value })
        .collect();
    Ok(NativeExecPreviewDto {
        profile,
        label: label(profile).into(),
        repository_path: git.root.clone(),
        source_commit: git.head_commit.clone(),
        source_tree: git.head_tree.clone(),
        executable: tool.clone(),
        argv: args(profile),
        working_directory: git.root.clone(),
        environment,
        timeout_ms: timeout(profile).as_millis().try_into().unwrap_or(u64::MAX),
        max_stdout_bytes: MAX_STDOUT as u64,
        max_stderr_bytes: MAX_STDERR as u64,
        trust_warning: TRUST_WARNING.into(),
        sandboxed: false,
    })
}

fn output(stream: &str, bytes: Vec<u8>, truncated: bool) -> NativeOutputDto {
    NativeOutputDto {
        stream: stream.into(),
        sha256: digest_bytes(&bytes),
        size: bytes.len() as u64,
        content_base64: STANDARD.encode(&bytes),
        content_utf8: String::from_utf8(bytes).ok(),
        truncated,
    }
}

pub(crate) fn run(
    run_id: String,
    current_git: &GitSnapshotDto,
    expected: &NativeExecPreviewDto,
    cancel: Arc<AtomicBool>,
) -> Result<NativeExecResultDto, PortError> {
    let rebuilt = preview(
        Path::new(&expected.repository_path),
        current_git,
        expected.profile,
        &expected.executable,
    )?;
    if &rebuilt != expected {
        return Err(PortError::Unsupported(
            "native execution preview is stale; preview again".into(),
        ));
    }
    let started_at_unix_ms = now_ms();
    let mut spec = ProcessSpec::new(
        PathBuf::from(&expected.executable.path),
        PathBuf::from(&expected.working_directory),
    )
    .args(&expected.argv);
    spec.timeout = timeout(expected.profile);
    spec.max_stdout = MAX_STDOUT;
    spec.max_stderr = MAX_STDERR;
    spec.path_prefix = Path::new(&expected.executable.path)
        .parent()
        .map(Path::to_path_buf);
    let result = run_cancellable(spec, cancel)?;
    let (state, process): (NativeExecStateDto, ProcessOutput) = match result {
        CancellableProcessOutput::Completed(output) if output.success => {
            (NativeExecStateDto::Completed, output)
        }
        CancellableProcessOutput::Completed(output) => (NativeExecStateDto::Failed, output),
        CancellableProcessOutput::Cancelled(output) => (NativeExecStateDto::Cancelled, output),
        CancellableProcessOutput::TimedOut(output) => (NativeExecStateDto::TimedOut, output),
    };
    let after = inspect_tool(expected.profile, Some(Path::new(&expected.executable.path)))?;
    if after.sha256 != expected.executable.sha256 || after.size != expected.executable.size {
        return Err(PortError::Unsupported(
            "selected native tool changed during execution".into(),
        ));
    }
    let producer_check_outcome = match state {
        NativeExecStateDto::Completed => "pass",
        NativeExecStateDto::Failed => "fail",
        NativeExecStateDto::Cancelled => "skipped",
        NativeExecStateDto::TimedOut => "error",
    };
    Ok(NativeExecResultDto {
        run_id,
        profile: expected.profile,
        state,
        exit_code: process.exit_code,
        started_at_unix_ms,
        completed_at_unix_ms: now_ms(),
        source_commit: expected.source_commit.clone(),
        source_tree: expected.source_tree.clone(),
        executable_sha256: expected.executable.sha256.clone(),
        stdout: output("stdout", process.stdout, process.stdout_truncated),
        stderr: output("stderr", process.stderr, process.stderr_truncated),
        producer_check_method: format!("vela-workbench-{}", profile_key(expected.profile)),
        producer_check_outcome: producer_check_outcome.into(),
    })
}

fn profile_key(profile: NativeExecProfileDto) -> &'static str {
    match profile {
        NativeExecProfileDto::GitDiffCheck => "git-diff-check",
        NativeExecProfileDto::LeanBuild => "lean-build",
        NativeExecProfileDto::CargoTest => "cargo-test",
        NativeExecProfileDto::BunTest => "bun-test",
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::Path, process::Command, sync::Arc};

    use super::{SYSTEM_GIT, TRUST_WARNING, args, inspect_tool, preview, run};
    use crate::{contracts::NativeExecProfileDto, ports::git};

    #[test]
    fn profiles_have_fixed_argv_and_explicit_non_sandbox_warning() {
        assert_eq!(args(NativeExecProfileDto::LeanBuild), ["build".to_string()]);
        assert_eq!(
            args(NativeExecProfileDto::CargoTest),
            ["test".to_string(), "--locked".to_string()]
        );
        assert!(TRUST_WARNING.contains("current local user privileges"));
        assert!(TRUST_WARNING.contains("not a sandbox"));
    }

    fn file_bytes(path: &Path, root: &Path, output: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries: Vec<_> = fs::read_dir(path)
            .expect("read directory")
            .map(|entry| entry.expect("entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                file_bytes(&entry, root, output);
            } else if entry.is_file() {
                output.insert(
                    entry
                        .strip_prefix(root)
                        .expect("relative")
                        .display()
                        .to_string(),
                    fs::read(&entry).expect("bytes"),
                );
            }
        }
    }

    #[test]
    fn reviewed_git_profile_is_explicit_bounded_and_byte_preserving() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        assert!(
            Command::new(SYSTEM_GIT)
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .expect("init")
                .success()
        );
        fs::write(root.join("result.txt"), "bounded result\n").expect("fixture");
        assert!(
            Command::new(SYSTEM_GIT)
                .args(["add", "result.txt"])
                .current_dir(root)
                .status()
                .expect("add")
                .success()
        );
        assert!(
            Command::new(SYSTEM_GIT)
                .args([
                    "-c",
                    "user.name=Vela Test",
                    "-c",
                    "user.email=test@invalid.example",
                    "commit",
                    "-q",
                    "-m",
                    "fixture"
                ])
                .current_dir(root)
                .status()
                .expect("commit")
                .success()
        );
        let git = git::inspect(root).expect("snapshot");
        let tool = inspect_tool(NativeExecProfileDto::GitDiffCheck, None).expect("system Git");
        let reviewed =
            preview(root, &git, NativeExecProfileDto::GitDiffCheck, &tool).expect("preview");
        assert_eq!(
            reviewed.argv,
            ["diff", "--no-ext-diff", "--no-textconv", "--check"]
        );
        assert!(!reviewed.sandboxed);
        assert!(reviewed.environment.iter().all(|entry| !matches!(
            entry.name.as_str(),
            "SSH_AUTH_SOCK" | "GITHUB_TOKEN" | "AWS_ACCESS_KEY_ID"
        )));
        let mut before = BTreeMap::new();
        file_bytes(root, root, &mut before);
        let result = run("run-explicit-test".into(), &git, &reviewed, Arc::default()).expect("run");
        let mut after = BTreeMap::new();
        file_bytes(root, root, &mut after);
        assert_eq!(
            result.state,
            crate::contracts::NativeExecStateDto::Completed
        );
        assert_eq!(before, after);
    }
}
