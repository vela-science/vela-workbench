use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use crate::contracts::{
    EntireCheckpointDto, GitRemoteDto, GitSnapshotDto, GitWorktreeDto, WorktreePreviewDto,
};
use url::Url;

use super::{
    PortError, ProcessSpec, ensure_not_truncated, process::canonical_directory, process::utf8,
    run_bounded,
};

const LOG_FORMAT: &str =
    "%H%x00%T%x00%(trailers:key=Entire-Checkpoint,valueonly,separator=%x1f)%x00";
const CHECK_ATTR_ARG_BYTES: usize = 128 * 1024;
const CHECK_ATTR_PATHS: usize = 16 * 1024;
static TEMP_INDEX_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "macos")]
const GIT_PROGRAM: &str = "/usr/bin/git";
#[cfg(not(target_os = "macos"))]
const GIT_PROGRAM: &str = "git";

fn run_git(root: &Path, args: &[&str]) -> Result<String, PortError> {
    let root = canonical_directory(root)?;
    let mut argv = vec![
        OsString::from("--no-optional-locks"),
        OsString::from("-c"),
        OsString::from("core.fsmonitor=false"),
        OsString::from("-c"),
        OsString::from("core.hooksPath=/dev/null"),
        OsString::from("-C"),
        root.as_os_str().to_os_string(),
    ];
    argv.extend(args.iter().map(OsString::from));
    let mut spec = ProcessSpec::new(GIT_PROGRAM, &root).args(argv);
    spec.timeout = Duration::from_secs(6);
    let output = run_bounded(spec)?;
    ensure_not_truncated(&output, "git")?;
    if !output.success {
        let stderr = utf8(&output.stderr, "git stderr")?;
        return Err(PortError::Process(format!(
            "git read failed with {:?}: {}",
            output.exit_code,
            stderr.trim()
        )));
    }
    utf8(&output.stdout, "git stdout")
}

fn run_git_os(root: &Path, args: Vec<OsString>, timeout: Duration) -> Result<String, PortError> {
    run_git_os_with_index(root, args, timeout, None)
}

fn run_git_os_with_index(
    root: &Path,
    args: Vec<OsString>,
    timeout: Duration,
    index: Option<&Path>,
) -> Result<String, PortError> {
    let root = canonical_directory(root)?;
    let mut argv = vec![
        OsString::from("--no-optional-locks"),
        OsString::from("-c"),
        OsString::from("core.fsmonitor=false"),
        OsString::from("-c"),
        OsString::from("core.hooksPath=/dev/null"),
    ];
    if index.is_some() {
        argv.extend([
            OsString::from("-c"),
            OsString::from("core.splitIndex=false"),
            OsString::from("-c"),
            OsString::from("core.sparseCheckout=false"),
        ]);
    }
    argv.extend([OsString::from("-C"), root.as_os_str().to_os_string()]);
    argv.extend(args);
    let mut spec = ProcessSpec::new(GIT_PROGRAM, &root).args(argv);
    if let Some(index) = index {
        spec = spec.env("GIT_INDEX_FILE", index.as_os_str());
    }
    spec.timeout = timeout;
    let output = run_bounded(spec)?;
    ensure_not_truncated(&output, "git")?;
    if !output.success {
        let stderr = utf8(&output.stderr, "git stderr")?;
        return Err(PortError::Process(format!(
            "git operation failed with {:?}: {}",
            output.exit_code,
            stderr.trim()
        )));
    }
    utf8(&output.stdout, "git stdout")
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new().mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

struct TemporaryGitIndex {
    directory: Option<PathBuf>,
    index: PathBuf,
}

impl TemporaryGitIndex {
    fn new(parent: &Path) -> Result<Self, PortError> {
        let parent = canonical_directory(parent)?;
        for _ in 0..1024 {
            let sequence = TEMP_INDEX_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = parent.join(format!(
                ".vela-workbench-git-index-{}-{sequence}",
                std::process::id()
            ));
            match create_private_directory(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        index: directory.join("index"),
                        directory: Some(directory),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(PortError::Process(format!(
                        "create temporary Git index directory: {error}"
                    )));
                }
            }
        }
        Err(PortError::Process(
            "could not allocate a unique temporary Git index directory".into(),
        ))
    }

    fn close(mut self) -> Result<(), PortError> {
        let Some(directory) = self.directory.take() else {
            return Ok(());
        };
        fs::remove_dir_all(&directory).map_err(|error| {
            PortError::Process(format!(
                "remove temporary Git index directory {}: {error}",
                directory.display()
            ))
        })
    }
}

impl Drop for TemporaryGitIndex {
    fn drop(&mut self) {
        if let Some(directory) = self.directory.take() {
            let _ = fs::remove_dir_all(directory);
        }
    }
}

fn validated_ref(value: &str) -> Result<&str, PortError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 512
        || value.starts_with('-')
        || value.chars().any(char::is_control)
    {
        return Err(PortError::InvalidInput(
            "target ref must be a non-empty bounded Git ref without flags or control characters"
                .into(),
        ));
    }
    Ok(value)
}

pub(crate) fn require_tracked_clean_at_head(root: &Path, relative: &str) -> Result<(), PortError> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.len() > 4096
        || path.is_absolute()
        || !path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(PortError::InvalidInput(
            "retained evidence path must be one normalized repository-relative path".into(),
        ));
    }
    run_git_os(
        root,
        vec![
            OsString::from("ls-files"),
            OsString::from("--error-unmatch"),
            OsString::from("--"),
            OsString::from(relative),
        ],
        Duration::from_secs(6),
    )?;
    run_git_os(
        root,
        vec![
            OsString::from("diff"),
            OsString::from("--quiet"),
            OsString::from("--no-ext-diff"),
            OsString::from("--no-textconv"),
            OsString::from("HEAD"),
            OsString::from("--"),
            OsString::from(relative),
        ],
        Duration::from_secs(6),
    )?;
    Ok(())
}

fn empty_destination(path: &Path) -> Result<PathBuf, PortError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        PortError::InvalidInput(format!("inspect worktree destination: {error}"))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PortError::InvalidInput(
            "worktree destination must be one existing non-symlink directory".into(),
        ));
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        PortError::InvalidInput(format!("resolve worktree destination: {error}"))
    })?;
    if std::fs::read_dir(&canonical)
        .map_err(|error| PortError::InvalidInput(format!("read worktree destination: {error}")))?
        .next()
        .is_some()
    {
        return Err(PortError::InvalidInput(
            "worktree destination must be empty".into(),
        ));
    }
    Ok(canonical)
}

fn inspect_checkout_filters(
    root: &Path,
    target_commit: &str,
    index: &Path,
) -> Result<(), PortError> {
    run_git_os_with_index(
        root,
        vec![OsString::from("read-tree"), OsString::from(target_commit)],
        Duration::from_secs(15),
        Some(index),
    )?;
    let names = run_git_os_with_index(
        root,
        vec![OsString::from("ls-files"), OsString::from("-z")],
        Duration::from_secs(15),
        Some(index),
    )?;
    let paths: Vec<&str> = names
        .split('\0')
        .filter(|value| !value.is_empty())
        .collect();
    let argv_bytes = paths.iter().map(|value| value.len() + 1).sum::<usize>();
    if paths.len() > 100_000 || argv_bytes > 512 * 1024 {
        return Err(PortError::Unsupported(
            "target tree is too large for bounded checkout-filter validation".into(),
        ));
    }

    let mut offset = 0;
    while offset < paths.len() {
        let mut end = offset;
        let mut bytes = 0;
        while end < paths.len()
            && end - offset < CHECK_ATTR_PATHS
            && (end == offset || bytes + paths[end].len() < CHECK_ATTR_ARG_BYTES)
        {
            bytes += paths[end].len() + 1;
            end += 1;
        }
        let mut args = vec![
            OsString::from("check-attr"),
            OsString::from("--cached"),
            OsString::from("-z"),
            OsString::from("--all"),
            OsString::from("--"),
        ];
        args.extend(paths[offset..end].iter().map(OsString::from));
        let attributes = run_git_os_with_index(root, args, Duration::from_secs(15), Some(index))?;
        let fields: Vec<&str> = attributes.split('\0').collect();
        for row in fields.chunks(3) {
            if row.len() == 3 && !row[0].is_empty() && row[1] == "filter" {
                return Err(PortError::Unsupported(format!(
                    "target tree assigns checkout filter attribute {:?}; Workbench will not execute repository-configured smudge/process filters",
                    row[2]
                )));
            }
        }
        offset = end;
    }
    Ok(())
}

fn refuse_checkout_filters_in(
    root: &Path,
    target_commit: &str,
    temporary_parent: &Path,
) -> Result<(), PortError> {
    let temporary = TemporaryGitIndex::new(temporary_parent)?;
    let result = inspect_checkout_filters(root, target_commit, &temporary.index);
    let cleanup = temporary.close();
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(PortError::Process(format!(
            "{error}; temporary Git index cleanup also failed: {cleanup}"
        ))),
    }
}

fn refuse_checkout_filters(root: &Path, target_commit: &str) -> Result<(), PortError> {
    refuse_checkout_filters_in(root, target_commit, &std::env::temp_dir())
}

pub(crate) fn preview_worktree(
    root: &Path,
    target_ref: &str,
    destination: &Path,
) -> Result<WorktreePreviewDto, PortError> {
    let snapshot = inspect(root)?;
    let target_ref = validated_ref(target_ref)?;
    let destination = empty_destination(destination)?;
    if snapshot
        .worktrees
        .iter()
        .any(|worktree| destination.starts_with(Path::new(&worktree.path)))
    {
        return Err(PortError::InvalidInput(
            "worktree destination must be outside every existing worktree checkout".into(),
        ));
    }
    let revision = format!("{target_ref}^{{commit}}");
    let resolved = run_git_os(
        Path::new(&snapshot.root),
        vec![
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("--end-of-options"),
            OsString::from(revision),
        ],
        Duration::from_secs(6),
    )?;
    let target_commit = resolved.trim().to_ascii_lowercase();
    if target_commit.len() != 40 || !target_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(PortError::Parse(
            "Git ref did not resolve to one full commit id".into(),
        ));
    }
    refuse_checkout_filters(Path::new(&snapshot.root), &target_commit)?;
    Ok(WorktreePreviewDto {
        repository_path: snapshot.root.clone(),
        source_head: snapshot.head_commit,
        source_tree: snapshot.head_tree,
        target_ref: target_ref.into(),
        target_commit: target_commit.clone(),
        destination: destination.display().to_string(),
        command: vec![
            GIT_PROGRAM.into(),
            "--no-optional-locks".into(),
            "-c".into(),
            "core.fsmonitor=false".into(),
            "-c".into(),
            "core.hooksPath=/dev/null".into(),
            "-C".into(),
            snapshot.root.clone(),
            "worktree".into(),
            "add".into(),
            "--detach".into(),
            destination.display().to_string(),
            target_commit.clone(),
        ],
        rollback: vec![
            GIT_PROGRAM.into(),
            "--no-optional-locks".into(),
            "-c".into(),
            "core.fsmonitor=false".into(),
            "-c".into(),
            "core.hooksPath=/dev/null".into(),
            "-C".into(),
            snapshot.root.clone(),
            "worktree".into(),
            "remove".into(),
            destination.display().to_string(),
        ],
        warning: "Creates one detached checkout at the exact resolved commit. It does not switch, reset, or modify the selected checkout.".into(),
    })
}

pub(crate) fn create_worktree(expected: &WorktreePreviewDto) -> Result<PathBuf, PortError> {
    let rebuilt = preview_worktree(
        Path::new(&expected.repository_path),
        &expected.target_ref,
        Path::new(&expected.destination),
    )?;
    if &rebuilt != expected {
        return Err(PortError::Unsupported(
            "worktree preview is stale; preview again".into(),
        ));
    }
    run_git_os(
        Path::new(&expected.repository_path),
        vec![
            OsString::from("worktree"),
            OsString::from("add"),
            OsString::from("--detach"),
            OsString::from(&expected.destination),
            OsString::from(&expected.target_commit),
        ],
        Duration::from_secs(60),
    )?;
    let destination = std::fs::canonicalize(&expected.destination).map_err(|error| {
        PortError::Process(format!("resolve created worktree destination: {error}"))
    })?;
    let snapshot = inspect(&destination)?;
    if snapshot.head_commit != expected.target_commit || !snapshot.detached {
        return Err(PortError::Process(
            "created worktree does not match the reviewed detached commit".into(),
        ));
    }
    Ok(destination)
}

pub(crate) fn inspect(root: &Path) -> Result<GitSnapshotDto, PortError> {
    let selected = canonical_directory(root)?;
    let reported_root = run_git(&selected, &["rev-parse", "--show-toplevel"])?;
    let root = canonical_directory(Path::new(reported_root.trim()))?;
    if !selected.starts_with(&root) {
        return Err(PortError::InvalidInput(format!(
            "Git reported a worktree root outside the selected directory: {}",
            root.display()
        )));
    }
    let status = run_git(&root, &["status", "--porcelain=v2", "--branch", "-z"])?;
    let worktrees = run_git(&root, &["worktree", "list", "--porcelain"])?;
    let remotes = run_git(&root, &["remote", "-v"])?;
    let log = run_git(
        &root,
        &["log", "-n", "100", &format!("--format={LOG_FORMAT}")],
    )?;

    let parsed_status = parse_status(&status)?;
    let (head_commit, head_tree, checkpoints) = parse_log(&log)?;
    if parsed_status.head_oid != head_commit {
        return Err(PortError::Parse(format!(
            "Git status HEAD {} disagrees with log HEAD {head_commit}",
            parsed_status.head_oid
        )));
    }

    Ok(GitSnapshotDto {
        root: root.display().to_string(),
        branch: parsed_status.branch,
        detached: parsed_status.detached,
        head_commit,
        head_tree,
        upstream: parsed_status.upstream,
        ahead: parsed_status.ahead,
        behind: parsed_status.behind,
        dirty: parsed_status.changed_paths > 0,
        conflicted: parsed_status.conflicted,
        changed_paths: parsed_status.changed_paths,
        worktrees: parse_worktrees(&worktrees)?,
        remotes: parse_remotes(&remotes)?,
        entire_checkpoints: checkpoints,
    })
}

#[derive(Debug, Default)]
struct ParsedStatus {
    head_oid: String,
    branch: Option<String>,
    detached: bool,
    upstream: Option<String>,
    ahead: u64,
    behind: u64,
    changed_paths: u64,
    conflicted: bool,
}

fn parse_status(raw: &str) -> Result<ParsedStatus, PortError> {
    let normalized = raw.replace('\0', "\n");
    let mut parsed = ParsedStatus::default();
    for line in normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some(value) = line.strip_prefix("# branch.oid ") {
            parsed.head_oid = value.to_string();
        } else if let Some(value) = line.strip_prefix("# branch.head ") {
            parsed.detached = value == "(detached)";
            parsed.branch = (!parsed.detached).then(|| value.to_string());
        } else if let Some(value) = line.strip_prefix("# branch.upstream ") {
            parsed.upstream = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("# branch.ab ") {
            let mut fields = value.split_whitespace();
            parsed.ahead = fields
                .next()
                .and_then(|item| item.strip_prefix('+'))
                .and_then(|item| item.parse().ok())
                .unwrap_or_default();
            parsed.behind = fields
                .next()
                .and_then(|item| item.strip_prefix('-'))
                .and_then(|item| item.parse().ok())
                .unwrap_or_default();
        } else if line.starts_with("1 ") || line.starts_with("2 ") || line.starts_with("? ") {
            parsed.changed_paths += 1;
        } else if line.starts_with("u ") {
            parsed.changed_paths += 1;
            parsed.conflicted = true;
        }
    }
    if parsed.head_oid.len() != 40 || !parsed.head_oid.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(PortError::Parse(
            "Git status did not report a full HEAD object id".into(),
        ));
    }
    Ok(parsed)
}

fn parse_log(raw: &str) -> Result<(String, String, Vec<EntireCheckpointDto>), PortError> {
    // Empty trailer fields are meaningful record separators. Preserve them so
    // a commit without an Entire trailer cannot shift the next commit's fields.
    let fields: Vec<&str> = raw.split('\0').collect();
    if fields.len() < 2 {
        return Err(PortError::Parse(
            "Git log did not report HEAD commit and tree".into(),
        ));
    }
    let head_commit = fields[0].to_string();
    let head_tree = fields[1].to_string();
    if head_commit.len() != 40 || head_tree.len() != 40 {
        return Err(PortError::Parse(
            "Git log returned a non-full commit or tree id".into(),
        ));
    }

    let mut checkpoints = Vec::new();
    for chunk in fields.chunks(3) {
        if chunk.len() < 2 {
            break;
        }
        let commit = chunk[0].trim().to_string();
        if let Some(trailers) = chunk.get(2) {
            for checkpoint_id in trailers
                .split(['\u{1f}', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                checkpoints.push(EntireCheckpointDto {
                    commit: commit.clone(),
                    checkpoint_id: checkpoint_id.to_string(),
                });
            }
        }
    }
    Ok((head_commit, head_tree, checkpoints))
}

fn parse_worktrees(raw: &str) -> Result<Vec<GitWorktreeDto>, PortError> {
    let mut result = Vec::new();
    for block in raw.split("\n\n").filter(|block| !block.trim().is_empty()) {
        let mut item = GitWorktreeDto {
            path: String::new(),
            head: None,
            branch: None,
            detached: false,
            locked: false,
            prunable: false,
        };
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("worktree ") {
                item.path = value.to_string();
            } else if let Some(value) = line.strip_prefix("HEAD ") {
                item.head = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("branch ") {
                item.branch = Some(
                    value
                        .strip_prefix("refs/heads/")
                        .unwrap_or(value)
                        .to_string(),
                );
            } else if line == "detached" {
                item.detached = true;
            } else if line.starts_with("locked") {
                item.locked = true;
            } else if line.starts_with("prunable") {
                item.prunable = true;
            }
        }
        if item.path.is_empty() {
            return Err(PortError::Parse(
                "Git worktree entry omitted its path".into(),
            ));
        }
        result.push(item);
    }
    Ok(result)
}

fn parse_remotes(raw: &str) -> Result<Vec<GitRemoteDto>, PortError> {
    let mut result = Vec::new();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let (name, rest) = line
            .split_once('\t')
            .ok_or_else(|| PortError::Parse("Git remote row omitted its tab separator".into()))?;
        let (url, operation) = rest
            .rsplit_once(' ')
            .ok_or_else(|| PortError::Parse("Git remote row omitted its operation".into()))?;
        result.push(GitRemoteDto {
            name: name.to_string(),
            url: redact_remote_url(url),
            operation: operation
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string(),
        });
    }
    Ok(result)
}

fn redact_remote_url(raw: &str) -> String {
    if let Ok(mut url) = Url::parse(raw) {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string();
    }
    if let Some((host_part, path)) = raw.split_once(':')
        && !host_part.starts_with('/')
        && let Some((_, host)) = host_part.rsplit_once('@')
    {
        return format!("ssh://{host}/{}", path.trim_start_matches('/'));
    }
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::Path, process::Command, thread};

    use super::{
        GIT_PROGRAM, create_worktree, inspect, parse_log, parse_remotes, parse_status,
        parse_worktrees, preview_worktree, refuse_checkout_filters_in,
        require_tracked_clean_at_head,
    };

    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new(GIT_PROGRAM)
            .args(args)
            .current_dir(root)
            .output()
            .expect("run git fixture command");
        assert!(
            output.status.success(),
            "git fixture command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("git fixture output is UTF-8")
            .trim()
            .to_string()
    }

    fn commit_all(root: &Path, message: &str) -> String {
        git(root, &["add", "-A"]);
        git(
            root,
            &[
                "-c",
                "user.name=Vela Test",
                "-c",
                "user.email=test@invalid.example",
                "commit",
                "-q",
                "-m",
                message,
            ],
        );
        git(root, &["rev-parse", "HEAD"])
    }

    fn bytes_under(path: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries: Vec<_> = fs::read_dir(path)
            .expect("read directory")
            .map(|entry| entry.expect("directory entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                bytes_under(&entry, root, out);
            } else if entry.is_file() {
                out.insert(
                    entry
                        .strip_prefix(root)
                        .expect("relative path")
                        .display()
                        .to_string(),
                    fs::read(&entry).expect("read file"),
                );
            }
        }
    }

    fn snapshot_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut result = BTreeMap::new();
        bytes_under(root, root, &mut result);
        result
    }

    #[test]
    fn status_parser_keeps_branch_and_dirty_facts_separate() {
        let raw = "# branch.oid 0123456789012345678901234567890123456789\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +2 -1\n? new.txt\0u UU N... conflict.txt\0";
        let parsed = parse_status(raw).expect("status parses");
        assert_eq!(parsed.branch.as_deref(), Some("main"));
        assert_eq!((parsed.ahead, parsed.behind), (2, 1));
        assert_eq!(parsed.changed_paths, 2);
        assert!(parsed.conflicted);
    }

    #[test]
    fn log_parser_extracts_only_checkpoint_trailers() {
        let raw = "0123456789012345678901234567890123456789\0abcdefabcdefabcdefabcdefabcdefabcdefabcd\0checkpoint-1\u{1f}checkpoint-2\0";
        let (_, _, checkpoints) = parse_log(raw).expect("log parses");
        assert_eq!(checkpoints.len(), 2);
        assert_eq!(checkpoints[0].checkpoint_id, "checkpoint-1");
    }

    #[test]
    fn worktree_parser_preserves_detached_state() {
        let raw = "worktree /tmp/repo\nHEAD 0123456789012345678901234567890123456789\ndetached\n\n";
        let rows = parse_worktrees(raw).expect("worktrees parse");
        assert!(rows[0].detached);
        assert_eq!(rows[0].path, "/tmp/repo");
    }

    #[test]
    fn inspection_changes_no_repository_bytes_or_refs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .expect("git init")
                .success()
        );
        fs::write(root.join("result.txt"), "bounded evidence\n").expect("write fixture");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["add", "result.txt"])
                .current_dir(root)
                .status()
                .expect("git add")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
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
                .expect("git commit")
                .success()
        );
        let before = snapshot_bytes(root);
        let snapshot = inspect(root).expect("read-only inspection");
        let after = snapshot_bytes(root);
        assert_eq!(before, after);
        assert!(!snapshot.dirty);
    }

    #[test]
    fn retained_verification_inputs_must_be_tracked_and_unchanged_at_head() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .expect("init")
                .success()
        );
        fs::write(root.join("method.json"), "{}\n").expect("method");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["add", "method.json"])
                .current_dir(root)
                .status()
                .expect("add")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
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
        require_tracked_clean_at_head(root, "method.json").expect("tracked current method");
        fs::write(root.join("method.json"), "{\"changed\":true}\n").expect("dirty method");
        assert!(require_tracked_clean_at_head(root, "method.json").is_err());
        fs::write(root.join("untracked.json"), "{}\n").expect("untracked");
        assert!(require_tracked_clean_at_head(root, "untracked.json").is_err());
        assert!(require_tracked_clean_at_head(root, "../escape").is_err());
    }

    #[test]
    fn detached_worktree_creation_preserves_selected_checkout_and_binds_exact_commit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("source");
        let destination = temp.path().join("detached");
        fs::create_dir(&root).expect("source");
        fs::create_dir(&destination).expect("destination");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["init", "-q"])
                .current_dir(&root)
                .status()
                .expect("init")
                .success()
        );
        fs::write(root.join("result.txt"), "bounded evidence\n").expect("fixture");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["add", "result.txt"])
                .current_dir(&root)
                .status()
                .expect("add")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
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
                .current_dir(&root)
                .status()
                .expect("commit")
                .success()
        );
        let before = inspect(&root).expect("before");
        let source_bytes = fs::read(root.join("result.txt")).expect("source bytes");
        let preview = preview_worktree(&root, "HEAD", &destination).expect("preview");
        assert_eq!(preview.target_commit, before.head_commit);
        assert_eq!(preview.command[0], GIT_PROGRAM);
        assert_eq!(preview.command[1], "--no-optional-locks");
        assert!(
            preview
                .command
                .windows(2)
                .any(|pair| pair == ["-C", before.root.as_str()])
        );
        let created = create_worktree(&preview).expect("create");
        let created_snapshot = inspect(&created).expect("created inspect");
        let after = inspect(&root).expect("after");
        assert!(created_snapshot.detached);
        assert_eq!(created_snapshot.head_commit, before.head_commit);
        assert_eq!(after.head_commit, before.head_commit);
        assert_eq!(
            fs::read(root.join("result.txt")).expect("source bytes"),
            source_bytes
        );
        assert!(preview_worktree(&root, "--help", temp.path()).is_err());
        let nested = root.join("nested-worktree");
        fs::create_dir(&nested).expect("nested destination");
        let error = preview_worktree(&root, "HEAD", &nested)
            .expect_err("nested worktree must not dirty the selected checkout");
        assert!(
            error
                .to_string()
                .contains("outside every existing worktree")
        );

        let filtered_destination = temp.path().join("filtered-worktree");
        fs::create_dir(&filtered_destination).expect("filtered destination");
        let marker = temp.path().join("smudge-executed");
        fs::write(root.join(".gitattributes"), "*.txt filter=unspecified\n").expect("attributes");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["add", ".gitattributes"])
                .current_dir(&root)
                .status()
                .expect("add attributes")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
                .args([
                    "-c",
                    "user.name=Vela Test",
                    "-c",
                    "user.email=test@invalid.example",
                    "commit",
                    "-q",
                    "-m",
                    "hostile filter",
                ])
                .current_dir(&root)
                .status()
                .expect("commit attributes")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
                .args([
                    "config",
                    "filter.unspecified.smudge",
                    &format!("/bin/sh -c 'touch {}; cat'", marker.display()),
                ])
                .current_dir(&root)
                .status()
                .expect("configure hostile filter")
                .success()
        );
        let error = preview_worktree(&root, "HEAD", &filtered_destination)
            .expect_err("repository checkout filter must be refused before Git mutation");
        assert!(error.to_string().contains("checkout filter attribute"));
        assert!(!marker.exists());
    }

    #[test]
    fn checkout_filter_validation_is_exact_clean_bounded_and_concurrent() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("source");
        let indexes = temp.path().join("indexes");
        fs::create_dir(&root).expect("source");
        fs::create_dir(&indexes).expect("indexes");
        git(&root, &["init", "-q"]);
        fs::write(root.join("result.txt"), "bounded evidence\n").expect("fixture");
        let clean_commit = commit_all(&root, "clean target");

        let marker = temp.path().join("smudge-executed");
        git(
            &root,
            &[
                "config",
                "filter.unspecified.smudge",
                &format!("/bin/sh -c 'touch {}; cat'", marker.display()),
            ],
        );
        for (assignment, message) in [
            ("*.txt filter=unspecified\n", "valued filter"),
            ("*.txt filter\n", "set filter"),
            ("*.txt -filter\n", "unset filter"),
        ] {
            fs::write(root.join(".gitattributes"), assignment).expect("attributes");
            let hostile_commit = commit_all(&root, message);
            let error = refuse_checkout_filters_in(&root, &hostile_commit, &indexes)
                .expect_err("all filter assignments must fail closed");
            assert!(error.to_string().contains("checkout filter attribute"));
            assert!(!marker.exists());
            assert_eq!(fs::read_dir(&indexes).expect("read indexes").count(), 0);
        }

        fs::write(root.join(".gitattributes"), "*.txt filter=working-tree\n")
            .expect("hostile checked-out tree");
        refuse_checkout_filters_in(&root, &clean_commit, &indexes)
            .expect("clean exact target must ignore hostile selected index and checkout");
        assert_eq!(fs::read_dir(&indexes).expect("read indexes").count(), 0);

        let mut workers = Vec::new();
        for _ in 0..8 {
            let root = root.clone();
            let indexes = indexes.clone();
            let clean_commit = clean_commit.clone();
            workers.push(thread::spawn(move || {
                refuse_checkout_filters_in(&root, &clean_commit, &indexes)
                    .expect("concurrent exact-tree validation");
            }));
        }
        for worker in workers {
            worker.join().expect("validation worker");
        }
        assert_eq!(fs::read_dir(&indexes).expect("read indexes").count(), 0);

        for invalid in ["not-a-commit", "0000000000000000000000000000000000000000"] {
            refuse_checkout_filters_in(&root, invalid, &indexes)
                .expect_err("invalid target must fail closed");
            assert_eq!(fs::read_dir(&indexes).expect("read indexes").count(), 0);
        }
        assert!(!marker.exists());
    }

    #[test]
    fn remotes_drop_credentials_before_renderer_transport() {
        let rows = parse_remotes(
            "origin\thttps://token:secret@example.com/owner/repo.git?key=no#fragment (fetch)\n",
        )
        .expect("remote parses");
        assert_eq!(rows[0].url, "https://example.com/owner/repo.git");
        assert!(!rows[0].url.contains("token"));
        assert!(!rows[0].url.contains("secret"));
    }

    #[test]
    fn selected_symlink_is_resolved_to_its_canonical_worktree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("actual");
        fs::create_dir(&root).expect("actual directory");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["init", "-q"])
                .current_dir(&root)
                .status()
                .expect("git init")
                .success()
        );
        fs::write(root.join("result.txt"), "evidence\n").expect("fixture");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["add", "result.txt"])
                .current_dir(&root)
                .status()
                .expect("git add")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
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
                .current_dir(&root)
                .status()
                .expect("git commit")
                .success()
        );
        #[cfg(unix)]
        std::os::unix::fs::symlink(&root, temp.path().join("selected-link")).expect("symlink");
        let snapshot = inspect(&temp.path().join("selected-link")).expect("canonical selection");
        assert_eq!(
            snapshot.root,
            root.canonicalize()
                .expect("canonical root")
                .display()
                .to_string()
        );
    }

    #[test]
    fn separate_git_dir_cannot_redirect_selection_to_an_unrelated_worktree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let actual = temp.path().join("actual");
        let decoy = temp.path().join("decoy");
        fs::create_dir(&actual).expect("actual");
        fs::create_dir(&decoy).expect("decoy");
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["init", "-q"])
                .current_dir(&actual)
                .status()
                .expect("git init")
                .success()
        );
        assert!(
            Command::new(GIT_PROGRAM)
                .args(["config", "core.worktree", actual.to_str().expect("utf8")])
                .current_dir(&actual)
                .status()
                .expect("set worktree")
                .success()
        );
        fs::write(
            decoy.join(".git"),
            format!("gitdir: {}\n", actual.join(".git").display()),
        )
        .expect("gitdir file");
        let error = inspect(&decoy).expect_err("unrelated worktree must be refused");
        assert!(error.to_string().contains("outside the selected directory"));
    }
}
