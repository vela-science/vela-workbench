use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::contracts::{
    EnvironmentEntryDto, GitSnapshotDto, OpenGaussGitIdentityDto, OpenGaussHandoffPreviewDto,
    OpenGaussHandoffReceiptDto, OpenGaussProjectDto, OpenGaussToolDto,
};

use super::{PortError, ProcessSpec, ensure_not_truncated, environment_summary, git, run_bounded};

pub(crate) const UPSTREAM_SOURCE_COMMIT: &str = "f87633900ae185b8037bf451a914fe7eeae1eb08";
pub(crate) const UPSTREAM_SOURCE_TREE: &str = "aa3768f7cf5dd06d01a972bc8ed789f7b43246fb";
pub(crate) const SUPPORTED_VERSION_PREFIX: &str = "Gauss v0.2.2 (";
pub(crate) const TRUST_WARNING: &str = "The selected OpenGauss executable runs with your current local user privileges. Even the fixed version probe may read local OpenGauss configuration or authentication and perform OpenGauss-owned network/update checks. Workbench bounds environment, output, lifetime, and process-tree capture only; this is not a sandbox or security isolation.";

const MAX_EXECUTABLE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_TEXT: usize = 1024;
const MAX_MARKERS: usize = 32;
const MAX_PROBE_STDOUT: usize = 256 * 1024;
const MAX_PROBE_STDERR: usize = 128 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenGaussCandidate {
    pub path: PathBuf,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectManifest {
    schema_version: u64,
    name: String,
    #[serde(default)]
    kind: Option<String>,
    lean_root: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    source: Option<ProjectSource>,
    #[serde(default)]
    blueprint: Option<ProjectBlueprint>,
    #[serde(default)]
    paths: Option<ProjectPaths>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ProjectSource {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    template_source: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ProjectBlueprint {
    #[serde(default)]
    markers: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ProjectPaths {
    #[serde(default)]
    runtime: Option<String>,
    #[serde(default)]
    cache: Option<String>,
    #[serde(default)]
    workflows: Option<String>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn hash_file(path: &Path, label: &str, max_size: u64) -> Result<(String, u64), PortError> {
    let mut file = File::open(path)
        .map_err(|error| PortError::InvalidInput(format!("open {label}: {error}")))?;
    let size = file
        .metadata()
        .map_err(|error| PortError::InvalidInput(format!("inspect {label}: {error}")))?
        .len();
    if size > max_size {
        return Err(PortError::Unsupported(format!(
            "{label} exceeds the {max_size} byte pilot limit"
        )));
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| PortError::Process(format!("hash {label}: {error}")))?;
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

fn read_manifest(path: &Path) -> Result<Vec<u8>, PortError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        PortError::InvalidInput(format!("inspect OpenGauss project manifest: {error}"))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "OpenGauss project manifest must be one regular non-symlink file".into(),
        ));
    }
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(PortError::Unsupported(format!(
            "OpenGauss project manifest exceeds {MAX_MANIFEST_BYTES} bytes"
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)
        .map_err(|error| PortError::InvalidInput(format!("open project manifest: {error}")))?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| PortError::Process(format!("read project manifest: {error}")))?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(PortError::Unsupported(
            "OpenGauss project manifest grew beyond its read bound".into(),
        ));
    }
    Ok(bytes)
}

fn bounded_text(value: &str, label: &str, allow_empty: bool) -> Result<String, PortError> {
    let value = value.trim();
    if (!allow_empty && value.is_empty())
        || value.len() > MAX_TEXT
        || value.chars().any(char::is_control)
    {
        return Err(PortError::InvalidInput(format!(
            "OpenGauss project {label} is not bounded control-free text"
        )));
    }
    Ok(value.into())
}

fn contained_declared_path(
    root: &Path,
    raw: &str,
    label: &str,
    require_directory: bool,
) -> Result<PathBuf, PortError> {
    let raw = bounded_text(raw, label, false)?;
    if raw == "." {
        return Ok(root.to_path_buf());
    }
    let relative = Path::new(&raw);
    if relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(PortError::InvalidInput(format!(
            "OpenGauss project {label} must be one normalized relative path"
        )));
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(PortError::InvalidInput(format!(
                "OpenGauss project {label} contains an unsupported path component"
            )));
        };
        current.push(name);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(PortError::InvalidInput(format!(
                        "OpenGauss project {label} crosses a symlink"
                    )));
                }
                let canonical = std::fs::canonicalize(&current).map_err(|error| {
                    PortError::InvalidInput(format!("resolve OpenGauss {label}: {error}"))
                })?;
                if !canonical.starts_with(root) {
                    return Err(PortError::InvalidInput(format!(
                        "OpenGauss project {label} escapes the selected Repository"
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(PortError::InvalidInput(format!(
                    "inspect OpenGauss project {label}: {error}"
                )));
            }
        }
    }
    if current.exists() && require_directory && !current.is_dir() {
        return Err(PortError::InvalidInput(format!(
            "OpenGauss project {label} must resolve to a directory"
        )));
    }
    Ok(current)
}

pub(crate) fn inspect_candidate(selected: &Path) -> Result<OpenGaussCandidate, PortError> {
    let selected_name = selected.file_name().and_then(|value| value.to_str());
    if selected_name != Some("gauss") {
        return Err(PortError::InvalidInput(
            "OpenGauss selection requires an executable named gauss".into(),
        ));
    }
    let canonical = std::fs::canonicalize(selected).map_err(|error| {
        PortError::InvalidInput(format!("resolve selected OpenGauss executable: {error}"))
    })?;
    if canonical.file_name().and_then(|value| value.to_str()) != Some("gauss") {
        return Err(PortError::InvalidInput(
            "OpenGauss selection requires a canonical executable named gauss".into(),
        ));
    }
    let metadata = std::fs::metadata(&canonical).map_err(|error| {
        PortError::InvalidInput(format!("inspect OpenGauss executable: {error}"))
    })?;
    if !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "selected OpenGauss executable is not a regular file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(PortError::InvalidInput(
                "selected OpenGauss file is not executable".into(),
            ));
        }
    }
    let (sha256, size) = hash_file(&canonical, "OpenGauss executable", MAX_EXECUTABLE_BYTES)?;
    Ok(OpenGaussCandidate {
        path: canonical,
        sha256,
        size,
    })
}

fn project(repository: &Path) -> Result<OpenGaussProjectDto, PortError> {
    let repository = std::fs::canonicalize(repository)
        .map_err(|error| PortError::InvalidInput(format!("resolve Repository: {error}")))?;
    let manifest_path = repository.join(".gauss/project.yaml");
    let canonical_manifest = std::fs::canonicalize(&manifest_path).map_err(|error| {
        PortError::Unavailable(format!(
            "selected Repository has no readable .gauss/project.yaml: {error}"
        ))
    })?;
    if !canonical_manifest.starts_with(&repository) {
        return Err(PortError::InvalidInput(
            "OpenGauss project manifest escapes the selected Repository".into(),
        ));
    }
    let bytes = read_manifest(&manifest_path)?;
    let manifest: ProjectManifest = yaml_serde::from_slice(&bytes)
        .map_err(|error| PortError::Parse(format!("parse OpenGauss project manifest: {error}")))?;
    if manifest.schema_version != 1 {
        return Err(PortError::Unsupported(format!(
            "OpenGauss project schema_version {} is unsupported; expected 1",
            manifest.schema_version
        )));
    }
    let name = bounded_text(&manifest.name, "name", false)?;
    let kind = bounded_text(manifest.kind.as_deref().unwrap_or("lean4"), "kind", false)?;
    if kind != "lean4" {
        return Err(PortError::Unsupported(
            "OpenGauss pilot supports only kind lean4".into(),
        ));
    }
    if let Some(created_at) = &manifest.created_at {
        bounded_text(created_at, "created_at", true)?;
    }
    let lean_root = contained_declared_path(&repository, &manifest.lean_root, "lean_root", true)?;
    let has_lean_marker = ["lakefile.lean", "lakefile.toml"].iter().any(|name| {
        let path = lean_root.join(name);
        std::fs::symlink_metadata(path)
            .is_ok_and(|metadata| !metadata.file_type().is_symlink() && metadata.is_file())
    });
    if !has_lean_marker {
        return Err(PortError::InvalidInput(
            "OpenGauss project lean_root has no regular lakefile.lean or lakefile.toml".into(),
        ));
    }
    let source = manifest.source.unwrap_or_default();
    let source_mode = bounded_text(
        source.mode.as_deref().unwrap_or("init"),
        "source.mode",
        false,
    )?;
    let template_source_declared = source
        .template_source
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if let Some(template_source) = &source.template_source {
        bounded_text(template_source, "source.template_source", true)?;
    }
    let blueprint = manifest.blueprint.unwrap_or_default();
    if blueprint.markers.len() > MAX_MARKERS {
        return Err(PortError::Unsupported(
            "OpenGauss project blueprint marker list exceeds 32 entries".into(),
        ));
    }
    let blueprint_markers = blueprint
        .markers
        .iter()
        .map(|marker| bounded_text(marker, "blueprint marker", false))
        .collect::<Result<Vec<_>, _>>()?;
    let paths = manifest.paths.unwrap_or_default();
    for (value, default, label) in [
        (paths.runtime.as_deref(), ".gauss/runtime", "paths.runtime"),
        (paths.cache.as_deref(), ".gauss/cache", "paths.cache"),
        (
            paths.workflows.as_deref(),
            ".gauss/workflows",
            "paths.workflows",
        ),
    ] {
        contained_declared_path(&repository, value.unwrap_or(default), label, true)?;
    }
    Ok(OpenGaussProjectDto {
        manifest_path: canonical_manifest.display().to_string(),
        manifest_sha256: sha256(&bytes),
        manifest_size: bytes.len() as u64,
        schema_version: manifest.schema_version,
        name,
        kind,
        project_root: repository.display().to_string(),
        lean_root: lean_root.display().to_string(),
        source_mode,
        template_source_declared,
        blueprint_markers,
        configured_paths_validated: true,
    })
}

fn probe_version(repository: &Path, candidate: &OpenGaussCandidate) -> Result<String, PortError> {
    let mut spec = ProcessSpec::new(&candidate.path, repository).args(["--version"]);
    spec.timeout = Duration::from_secs(20);
    spec.max_stdout = MAX_PROBE_STDOUT;
    spec.max_stderr = MAX_PROBE_STDERR;
    spec.path_prefix = candidate.path.parent().map(Path::to_path_buf);
    let output = run_bounded(spec)?;
    ensure_not_truncated(&output, "OpenGauss version probe")?;
    if !output.success {
        return Err(PortError::Process(format!(
            "OpenGauss --version failed with exit {:?}",
            output.exit_code
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| PortError::Parse("OpenGauss --version stdout was not UTF-8".into()))?;
    let first = stdout.lines().next().unwrap_or("").trim();
    if first.len() > 160
        || first.chars().any(char::is_control)
        || !first.starts_with(SUPPORTED_VERSION_PREFIX)
        || !first.ends_with(')')
    {
        return Err(PortError::Unsupported(
            "OpenGauss pilot requires exact Gauss v0.2.2 version output".into(),
        ));
    }
    Ok(first.into())
}

fn environment(path_prefix: Option<&Path>) -> Vec<EnvironmentEntryDto> {
    environment_summary(path_prefix)
        .into_iter()
        .map(|(name, value)| EnvironmentEntryDto { name, value })
        .collect()
}

pub(crate) fn git_identity(snapshot: &GitSnapshotDto) -> OpenGaussGitIdentityDto {
    OpenGaussGitIdentityDto {
        branch: snapshot.branch.clone(),
        commit: snapshot.head_commit.clone(),
        tree: snapshot.head_tree.clone(),
        dirty: snapshot.dirty,
        changed_paths: snapshot.changed_paths,
    }
}

pub(crate) fn preview(
    repository: &Path,
    expected_git: &GitSnapshotDto,
    candidate: &OpenGaussCandidate,
) -> Result<OpenGaussHandoffPreviewDto, PortError> {
    let canonical_repository = std::fs::canonicalize(repository)
        .map_err(|error| PortError::InvalidInput(format!("resolve Repository: {error}")))?;
    if expected_git.root != canonical_repository.display().to_string() {
        return Err(PortError::InvalidInput(
            "OpenGauss Repository disagrees with the exact Git snapshot".into(),
        ));
    }
    let current_git = git::inspect(&canonical_repository)?;
    if &current_git != expected_git {
        return Err(PortError::Process(
            "Repository changed before OpenGauss inspection".into(),
        ));
    }
    let project_before = project(&canonical_repository)?;
    let before = inspect_candidate(&candidate.path)?;
    if &before != candidate {
        return Err(PortError::InvalidInput(
            "selected OpenGauss executable changed before version inspection".into(),
        ));
    }
    let version = probe_version(&canonical_repository, candidate)?;
    let after = inspect_candidate(&candidate.path)?;
    if &after != candidate {
        return Err(PortError::Process(
            "selected OpenGauss executable changed during version inspection".into(),
        ));
    }
    let project_after = project(&canonical_repository)?;
    let confirmed_git = git::inspect(&canonical_repository)?;
    if project_after != project_before || confirmed_git != current_git {
        return Err(PortError::Process(
            "Repository or OpenGauss project changed during inspection; select again".into(),
        ));
    }
    Ok(OpenGaussHandoffPreviewDto {
        repository_path: canonical_repository.display().to_string(),
        tool: OpenGaussToolDto {
            path: candidate.path.display().to_string(),
            version,
            sha256: candidate.sha256.clone(),
            size: candidate.size,
            probe_argv: vec!["--version".into()],
            probe_environment: environment(candidate.path.parent()),
            trust_warning: TRUST_WARNING.into(),
        },
        project: project_before,
        git_before: git_identity(&current_git),
        cwd: canonical_repository.display().to_string(),
        interactive_argv: vec![candidate.path.display().to_string()],
        launcher_environment: environment(None),
        documented_workflows: [
            "/prove",
            "/draft",
            "/review",
            "/checkpoint",
            "/refactor",
            "/golf",
            "/autoprove",
            "/formalize",
            "/autoformalize",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        documented_entrypoint: "Interactive OpenGauss slash command selected by the user after handoff".into(),
        backend_identity: "Not exposed by project.yaml or the Workbench handoff; OpenGauss owns backend selection".into(),
        hidden_transport_visible: false,
        upstream_source_commit: UPSTREAM_SOURCE_COMMIT.into(),
        upstream_source_tree: UPSTREAM_SOURCE_TREE.into(),
        authority_effect: "none".into(),
        boundary: "Workbench opens Terminal at the exact project root. It does not start OpenGauss, type a slash command, observe hidden model transport, or ingest OpenGauss session/swarm/trajectory state.".into(),
    })
}

pub(crate) fn revalidate_preview(
    expected: &OpenGaussHandoffPreviewDto,
) -> Result<OpenGaussHandoffPreviewDto, PortError> {
    let repository = Path::new(&expected.repository_path);
    let git = git::inspect(repository)?;
    let candidate = inspect_candidate(Path::new(&expected.tool.path))?;
    preview(repository, &git, &candidate)
}

pub(crate) fn validate_static(expected: &OpenGaussHandoffPreviewDto) -> Result<(), PortError> {
    let repository = Path::new(&expected.repository_path);
    let candidate = inspect_candidate(Path::new(&expected.tool.path))?;
    if candidate.path.display().to_string() != expected.tool.path
        || candidate.sha256 != expected.tool.sha256
        || candidate.size != expected.tool.size
    {
        return Err(PortError::Process(
            "selected OpenGauss executable changed after inspection".into(),
        ));
    }
    if project(repository)? != expected.project {
        return Err(PortError::Process(
            "OpenGauss project manifest changed after inspection; select again".into(),
        ));
    }
    let git_before = git::inspect(repository)?;
    if git_identity(&git_before) != expected.git_before || git::inspect(repository)? != git_before {
        return Err(PortError::Process(
            "Repository changed after OpenGauss inspection; select again".into(),
        ));
    }
    Ok(())
}

pub(crate) fn launched_receipt(
    preview: OpenGaussHandoffPreviewDto,
    terminal_owner: String,
) -> OpenGaussHandoffReceiptDto {
    OpenGaussHandoffReceiptDto {
        preview,
        terminal_owner,
        launched_at_unix_ms: now_ms(),
        git_after: None,
        selected_evidence: Vec::new(),
        selected_checks: Vec::new(),
        result_boundary: "No external result is inferred. After OpenGauss work, explicitly capture exact files and run reviewed checks; only selected bytes may feed an ordinary Submission v3.".into(),
    }
}

pub(crate) fn refresh_receipt(
    receipt: &OpenGaussHandoffReceiptDto,
) -> Result<OpenGaussHandoffReceiptDto, PortError> {
    let repository = Path::new(&receipt.preview.repository_path);
    let candidate = inspect_candidate(Path::new(&receipt.preview.tool.path))?;
    if candidate.path.display().to_string() != receipt.preview.tool.path
        || candidate.sha256 != receipt.preview.tool.sha256
        || candidate.size != receipt.preview.tool.size
    {
        return Err(PortError::Process(
            "selected OpenGauss executable changed after handoff".into(),
        ));
    }
    if project(repository)? != receipt.preview.project {
        return Err(PortError::Process(
            "OpenGauss project manifest changed after handoff; capture it explicitly if scientifically relevant and start a fresh handoff".into(),
        ));
    }
    let after = git::inspect(repository)?;
    if git::inspect(repository)? != after {
        return Err(PortError::Process(
            "Repository changed while refreshing the OpenGauss receipt".into(),
        ));
    }
    let mut refreshed = receipt.clone();
    refreshed.git_after = Some(git_identity(&after));
    refreshed.selected_evidence.clear();
    refreshed.selected_checks.clear();
    Ok(refreshed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    fn git(root: &Path, args: &[&str]) {
        let status = Command::new("/usr/bin/git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("run Git");
        assert!(status.success(), "Git failed: {args:?}");
    }

    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        git(root, &["init", "-q"]);
        git(root, &["config", "user.name", "Workbench test"]);
        git(root, &["config", "user.email", "workbench@example.invalid"]);
        fs::write(root.join("lakefile.lean"), "-- disposable Lean project\n").expect("lakefile");
        fs::create_dir(root.join(".gauss")).expect(".gauss");
        fs::write(
            root.join(".gauss/project.yaml"),
            "schema_version: 1\nname: Disposable pilot\nkind: lean4\nlean_root: .\nsource:\n  mode: init\n  template_source: ''\nblueprint:\n  markers: []\npaths:\n  runtime: .gauss/runtime\n  cache: .gauss/cache\n  workflows: .gauss/workflows\n",
        )
        .expect("manifest");
        let gauss = root.join("gauss");
        fs::write(
            &gauss,
            "#!/bin/sh\n[ \"$#\" -eq 1 ] && [ \"$1\" = \"--version\" ] || exit 91\nprintf 'Gauss v0.2.2 (2026.4.5)\\nProject: fixture\\n'\n",
        )
        .expect("gauss fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gauss).expect("gauss metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gauss, permissions).expect("chmod gauss");
        }
        git(root, &["add", "lakefile.lean", ".gauss/project.yaml"]);
        git(root, &["commit", "-q", "-m", "disposable project"]);
        (temp, gauss)
    }

    #[test]
    fn preview_binds_exact_tool_project_and_git_without_creating_gauss_state() {
        let (temp, gauss) = fixture();
        let before = super::git::inspect(temp.path()).expect("Git before");
        let candidate = inspect_candidate(&gauss).expect("candidate");
        let preview = preview(temp.path(), &before, &candidate).expect("preview");
        assert_eq!(preview.tool.version, "Gauss v0.2.2 (2026.4.5)");
        assert_eq!(preview.project.schema_version, 1);
        assert_eq!(preview.authority_effect, "none");
        assert!(!preview.hidden_transport_visible);
        assert_eq!(
            preview.interactive_argv,
            vec![gauss.canonicalize().unwrap().display().to_string()]
        );
        assert!(preview.tool.trust_warning.contains("not a sandbox"));
        assert!(preview.documented_workflows.contains(&"/prove".into()));
        assert!(preview.boundary.contains("does not start OpenGauss"));
        assert!(!temp.path().join(".gauss/runtime").exists());
        assert_eq!(super::git::inspect(temp.path()).unwrap(), before);
        validate_static(&preview).expect("unchanged static handoff");
        let receipt = launched_receipt(preview, "Terminal".into());
        let refreshed = refresh_receipt(&receipt).expect("refresh receipt");
        assert!(refreshed.git_after.is_some());
        assert!(refreshed.selected_evidence.is_empty());
        assert!(refreshed.selected_checks.is_empty());
    }

    #[test]
    fn manifest_paths_and_symlinks_fail_closed() {
        let (temp, _gauss) = fixture();
        fs::write(
            temp.path().join(".gauss/project.yaml"),
            "schema_version: 1\nname: Escape\nkind: lean4\nlean_root: ../outside\n",
        )
        .expect("hostile manifest");
        assert!(project(temp.path()).is_err());
    }

    #[test]
    fn unsupported_version_and_executable_change_are_refused() {
        let (temp, gauss) = fixture();
        let candidate = inspect_candidate(&gauss).expect("candidate");
        fs::write(&gauss, "#!/bin/sh\necho 'Gauss v9.9.9 (future)'\n").expect("changed gauss");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gauss).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gauss, permissions).unwrap();
        }
        let git = super::git::inspect(temp.path()).expect("Git");
        assert!(preview(temp.path(), &git, &candidate).is_err());
        let future = inspect_candidate(&gauss).expect("future candidate");
        assert!(preview(temp.path(), &git, &future).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn executable_alias_and_changed_project_are_refused() {
        use std::os::unix::fs::symlink;

        let (temp, gauss) = fixture();
        let alias = temp.path().join("gauss-alias");
        symlink(&gauss, &alias).expect("alias");
        assert!(inspect_candidate(&alias).is_err());

        let candidate = inspect_candidate(&gauss).expect("candidate");
        let git = super::git::inspect(temp.path()).expect("Git");
        let exact = preview(temp.path(), &git, &candidate).expect("preview");
        fs::write(
            temp.path().join(".gauss/project.yaml"),
            "schema_version: 1\nname: Changed\nkind: lean4\nlean_root: .\n",
        )
        .expect("change manifest");
        assert!(validate_static(&exact).is_err());
    }
}
