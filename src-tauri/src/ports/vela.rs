use std::{
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::contracts::{
    ClaimDto, ClaimsV1Wire, ErrorEnvelopeWire, IntegrationCheckV1Wire, IntegrationDto,
    IntegrationInspectionV1Wire, IntegrationItemDto, PublicationWire, RefusalDto,
    RepositoryClassificationDto, StatusCountsDto, StatusV4Wire, SubmissionArtifactDraftDto,
    SubmissionDraftDto, SubmissionImportPreviewDto, SubmissionPreviewDto, SubmissionResultDto,
    SubmitResultV1Wire, VelaBinaryDto, VelaBinaryStateDto, VelaInspectionDto, VelaStatusDto,
};

use super::{
    PortError, ProcessOutput, ProcessSpec, ensure_not_truncated, process::utf8, run_bounded,
};

pub(crate) const INTERFACE_COMMIT: &str = "c1a34373c2cdd937ed34fd128174a66fa12be71a";
pub(crate) const INTERFACE_TREE: &str = "b9188626039cfc1a4d7d4098d1b7fc6a4a92ad55";
pub(crate) const RUNTIME_VERSION: &str = "vela 0.977.2";
pub(crate) const RUNTIME_COMMIT: &str = "c1a34373c2cdd937ed34fd128174a66fa12be71a";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) const PLATFORM_RUNTIME_SHA256: &str =
    "286ed839ea81b7ed283e04ea1823c1515ad242dcee02b424787b8daa667625e2";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) const PLATFORM_RUNTIME_SHA256: &str =
    "3e2e12ac3410aa4a62013d3d7e2ceb828504c7beaff09cf1d126bc2d7ba077cd";
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
pub(crate) const PLATFORM_RUNTIME_SHA256: &str = "";

#[derive(Debug)]
enum Envelope<T> {
    Success(T),
    Failure(ErrorEnvelopeWire),
}

fn executable(path: &Path) -> Result<PathBuf, PortError> {
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        PortError::InvalidInput(format!(
            "resolve Vela executable {}: {error}",
            path.display()
        ))
    })?;
    let metadata = std::fs::metadata(&canonical)
        .map_err(|error| PortError::InvalidInput(format!("inspect Vela executable: {error}")))?;
    if !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "selected Vela path is not a file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(PortError::InvalidInput(
                "selected Vela file is not executable".into(),
            ));
        }
    }
    Ok(canonical)
}

fn sha256(path: &Path) -> Result<String, PortError> {
    let mut file = File::open(path)
        .map_err(|error| PortError::InvalidInput(format!("open Vela executable: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| PortError::Process(format!("hash Vela executable: {error}")))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn accepted_runtime_sha256(digest: &str) -> bool {
    !PLATFORM_RUNTIME_SHA256.is_empty() && digest == PLATFORM_RUNTIME_SHA256
}

pub(crate) fn inspect_binary(path: &Path) -> Result<VelaBinaryDto, PortError> {
    let path = executable(path)?;
    let digest = sha256(&path)?;
    if !accepted_runtime_sha256(&digest) {
        return Ok(VelaBinaryDto {
            path: path.display().to_string(),
            version: "not executed (unrecognized binary hash)".into(),
            sha256: digest,
            state: VelaBinaryStateDto::Unsupported,
        });
    }
    let cwd = path
        .parent()
        .ok_or_else(|| PortError::InvalidInput("Vela executable has no parent directory".into()))?;
    let mut spec = ProcessSpec::new(&path, cwd).args(["--version"]);
    spec.timeout = Duration::from_secs(4);
    spec.max_stdout = 16 * 1024;
    spec.max_stderr = 16 * 1024;
    let output = run_bounded(spec)?;
    ensure_not_truncated(&output, "vela --version")?;
    if !output.success {
        return Err(PortError::Process(format!(
            "vela --version failed with {:?}: {}",
            output.exit_code,
            utf8(&output.stderr, "Vela version stderr")?.trim()
        )));
    }
    let version = utf8(&output.stdout, "Vela version stdout")?
        .trim()
        .to_string();
    let post_digest = sha256(&path)?;
    if post_digest != digest {
        return Err(PortError::Unsupported(
            "selected Vela executable changed during identity verification".into(),
        ));
    }
    let state = if version == RUNTIME_VERSION {
        VelaBinaryStateDto::SignedRuntimeBaseline
    } else {
        VelaBinaryStateDto::Unsupported
    };
    Ok(VelaBinaryDto {
        path: path.display().to_string(),
        version,
        sha256: post_digest,
        state,
    })
}

fn run_json(binary: &Path, repository: &Path, args: &[&str]) -> Result<ProcessOutput, PortError> {
    if !accepted_runtime_sha256(&sha256(binary)?) {
        return Err(PortError::Unsupported(
            "selected Vela executable changed after identity verification".into(),
        ));
    }
    let mut argv: Vec<OsString> = args.iter().map(OsString::from).collect();
    let placeholder = argv
        .iter()
        .position(|arg| arg == "<repo>")
        .ok_or_else(|| PortError::InvalidInput("Vela argv omitted <repo> placeholder".into()))?;
    argv[placeholder] = repository.as_os_str().to_os_string();
    let mut spec = ProcessSpec::new(binary, repository).args(argv);
    spec.timeout = Duration::from_secs(10);
    let output = run_bounded(spec)?;
    if !accepted_runtime_sha256(&sha256(binary)?) {
        return Err(PortError::Unsupported(
            "selected Vela executable changed during bounded inspection".into(),
        ));
    }
    Ok(output)
}

fn parse_envelope<T: DeserializeOwned>(
    output: &ProcessOutput,
    expected_schema: &str,
    expected_command: &str,
) -> Result<Envelope<T>, PortError> {
    ensure_not_truncated(output, expected_command)?;
    let raw = utf8(&output.stdout, "Vela JSON stdout")?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        PortError::Parse(format!(
            "{expected_command} returned malformed JSON: {error}"
        ))
    })?;
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| PortError::Parse(format!("{expected_command} JSON omitted schema")))?;
    if schema == "vela.error.v1" {
        let envelope: ErrorEnvelopeWire = serde_json::from_value(value).map_err(|error| {
            PortError::Parse(format!(
                "{expected_command} error envelope is invalid: {error}"
            ))
        })?;
        if envelope.schema != "vela.error.v1"
            || envelope.ok
            || envelope.command != expected_command
            || output.success
        {
            return Err(PortError::Parse(format!(
                "{expected_command} error envelope tags disagree with process status"
            )));
        }
        return Ok(Envelope::Failure(envelope));
    }
    if schema != expected_schema {
        return Err(PortError::Unsupported(format!(
            "{expected_command} returned unsupported schema {schema}; expected {expected_schema}"
        )));
    }
    if !output.success {
        return Err(PortError::Parse(format!(
            "{expected_command} returned success schema with exit {:?}",
            output.exit_code
        )));
    }
    let envelope: T = serde_json::from_value(value).map_err(|error| {
        PortError::Parse(format!("{expected_command} JSON shape is invalid: {error}"))
    })?;
    Ok(Envelope::Success(envelope))
}

fn refusal(area: &str, envelope: ErrorEnvelopeWire) -> RefusalDto {
    RefusalDto {
        area: area.into(),
        kind: envelope.error.kind,
        code: envelope.error.code,
        message: envelope.error.message,
        hint: envelope.error.hint,
        command: envelope.command,
    }
}

fn validate_status(status: StatusV4Wire) -> Result<VelaStatusDto, PortError> {
    if status.schema != "vela.status.v4"
        || !status.ok
        || status.command != "status"
        || status.git.role != "repository_head"
    {
        return Err(PortError::Parse("status envelope tags are invalid".into()));
    }
    if status.counts.claims != status.counts.accepted_claims + status.counts.pending_claims {
        return Err(PortError::Parse(
            "status Claim counts do not form the required partition".into(),
        ));
    }
    if status.integrity.blocker_count != status.integrity.blockers_by_code.values().sum::<u64>() {
        return Err(PortError::Parse(
            "status blocker count disagrees with blockers_by_code".into(),
        ));
    }
    if status.decision_inbox.pending_count
        != status.decision_inbox.protocol_ready_count + status.decision_inbox.protocol_blocked_count
    {
        return Err(PortError::Parse(
            "status Decision Inbox counts do not form the required partition".into(),
        ));
    }
    if status.decision_inbox.pending_count == 0
        && (status.decision_inbox.first_entry_root.is_some() || status.actions.review.is_some())
    {
        return Err(PortError::Parse(
            "empty Decision Inbox reported a first entry or review action".into(),
        ));
    }
    if let Some(review) = &status.actions.review
        && (review.pending_count != status.decision_inbox.pending_count
            || review.command.is_empty())
    {
        return Err(PortError::Parse(
            "status review action disagrees with Decision Inbox".into(),
        ));
    }
    let (work_mode, work_command, work_note) = status.actions.work.parts();
    Ok(VelaStatusDto {
        repository_id: status.repository.id,
        repository_name: status.repository.name,
        repository_profile_root: status.repository.profile_root,
        repository_root: status.roots.repository,
        origin_root: status.roots.origin,
        authority_keyset_root: status.roots.authority_keyset,
        authority_policy_root: status.roots.authority_policy,
        repository_commit: status.git.commit,
        repository_tree: status.git.tree,
        replay: status.integrity.replay,
        strict: status.integrity.strict,
        blocker_count: status.integrity.blocker_count,
        counts: StatusCountsDto {
            claims: status.counts.claims,
            accepted_claims: status.counts.accepted_claims,
            pending_claims: status.counts.pending_claims,
            pending_review: status.counts.pending_review,
            accepted_review: status.counts.accepted_review,
            rejected_review: status.counts.rejected_review,
            withdrawn_review: status.counts.withdrawn_review,
            submissions: status.counts.submissions,
            verifications: status.counts.verifications,
            artifacts: status.counts.artifacts,
        },
        inbox_pending: status.decision_inbox.pending_count,
        inbox_projection_root: status.decision_inbox.projection_root,
        work_mode: work_mode.into(),
        work_command: work_command.into(),
        work_note: work_note.into(),
    })
}

fn validate_claims(
    claims: ClaimsV1Wire,
    status: &VelaStatusDto,
) -> Result<Vec<ClaimDto>, PortError> {
    if claims.schema != "vela.claims.v1"
        || !claims.ok
        || claims.command != "claims"
        || claims.status != "all"
        || claims.order != "claim_id_asc"
        || claims.repository_id != status.repository_id
        || status.repository_root.as_deref() != Some(claims.repository_root.as_str())
        || claims.returned as usize != claims.items.len()
        || claims.total < claims.returned
        || claims.unreadable_returned
            != claims.items.iter().filter(|item| !item.readable).count() as u64
    {
        return Err(PortError::Parse(
            "claims envelope invariants are invalid".into(),
        ));
    }
    if claims.next_cursor.is_none() && claims.total != claims.returned {
        return Err(PortError::Parse(
            "claims pagination ended before all rows were returned".into(),
        ));
    }
    claims
        .items
        .into_iter()
        .map(|item| {
            if !matches!(item.standing.as_str(), "accepted" | "unassessed") {
                return Err(PortError::Parse(format!(
                    "claims row has unsupported standing {}",
                    item.standing
                )));
            }
            if item.readable != item.assertion.is_some() {
                return Err(PortError::Parse(
                    "claims row readable flag disagrees with assertion presence".into(),
                ));
            }
            Ok(ClaimDto {
                claim_id: item.claim_id,
                claim_root: item.claim_root,
                standing: item.standing,
                origin_era: item.origin_era,
                readable: item.readable,
                assertion_kind: item.assertion_kind,
                assertion: item.assertion,
                unreadable_reason: item.unreadable_reason,
                created_at: item.created_at,
                revision: item.revision,
            })
        })
        .collect()
}

fn validate_integration(
    check: IntegrationCheckV1Wire,
    inspection: IntegrationInspectionV1Wire,
) -> Result<IntegrationDto, PortError> {
    if check.schema != "vela.cli.integration-check.v1"
        || inspection.schema != "vela.cli.integration-inspection.v1"
        || !check.ok
        || !inspection.ok
        || check.command != "integration check"
        || inspection.command != "integration inspect"
        || check.authority_effect != "none"
        || inspection.authority_effect != "none"
        || check.manifest_root != inspection.manifest_root
        || check.does_not_establish != inspection.does_not_establish
        || !check
            .does_not_establish
            .iter()
            .any(|value| value == "Standing")
    {
        return Err(PortError::Parse(
            "native integration envelopes do not preserve their non-authority contract".into(),
        ));
    }
    let observed_documents =
        1 + inspection.profiles.len() + inspection.bindings.len() + inspection.methods.len();
    if check.documents_checked != observed_documents as u64 {
        return Err(PortError::Parse(
            "native integration document count disagrees with inspection".into(),
        ));
    }
    let map_items = |items: Vec<crate::contracts::IntegrationItemWire>| {
        items
            .into_iter()
            .map(|item| IntegrationItemDto {
                kind: item.kind,
                id: item.id,
                path: item.path,
                root: item.root,
            })
            .collect()
    };
    Ok(IntegrationDto {
        authority_effect: check.authority_effect,
        manifest_root: check.manifest_root,
        documents_checked: check.documents_checked,
        repository: inspection.repository,
        revision: inspection.revision,
        profiles: map_items(inspection.profiles),
        bindings: map_items(inspection.bindings),
        methods: map_items(inspection.methods),
        does_not_establish: check.does_not_establish,
    })
}

pub(crate) fn inspect_repository(
    repository: &Path,
    binary_path: Option<&Path>,
) -> Result<(RepositoryClassificationDto, String, VelaInspectionDto), PortError> {
    let Some(binary_path) = binary_path else {
        return Ok((
            RepositoryClassificationDto::GitOnly,
            "Vela inspection unavailable: no signed runtime selected.".into(),
            VelaInspectionDto {
                binary: None,
                status: None,
                claims: Vec::new(),
                integration: None,
                refusal: Some(RefusalDto {
                    area: "vela_binary".into(),
                    kind: "unavailable".into(),
                    code: None,
                    message: "Select the installed signed Vela v0.977.2 executable to classify this repository.".into(),
                    hint: None,
                    command: "vela --version".into(),
                }),
            },
        ));
    };
    let binary = inspect_binary(binary_path)?;
    if binary.state != VelaBinaryStateDto::SignedRuntimeBaseline {
        return Ok((
            RepositoryClassificationDto::GitOnly,
            "Vela inspection refused: selected executable is not the pinned signed runtime baseline.".into(),
            VelaInspectionDto {
                binary: Some(binary),
                status: None,
                claims: Vec::new(),
                integration: None,
                refusal: Some(RefusalDto {
                    area: "vela_binary".into(),
                    kind: "unsupported".into(),
                    code: None,
                    message: "Runtime execution is pinned to signed Vela v0.977.2 with the reviewed platform hash.".into(),
                    hint: Some("Choose the installed signed v0.977.2 release binary for this platform.".into()),
                    command: "vela --version".into(),
                }),
            },
        ));
    }
    let binary_path = Path::new(&binary.path);

    let status_output = run_json(binary_path, repository, &["status", "<repo>", "--json"])?;
    match parse_envelope::<StatusV4Wire>(&status_output, "vela.status.v4", "status")? {
        Envelope::Success(status) => {
            let status = validate_status(status)?;
            let claims_output = run_json(
                binary_path,
                repository,
                &["claims", "<repo>", "--status", "all", "--json"],
            )?;
            let (claims, refusal) =
                match parse_envelope::<ClaimsV1Wire>(&claims_output, "vela.claims.v1", "claims")? {
                    Envelope::Success(claims) => (validate_claims(claims, &status)?, None),
                    Envelope::Failure(error) => (Vec::new(), Some(refusal("claims", error))),
                };
            Ok((
                RepositoryClassificationDto::VelaRepository,
                "Validated by signed Vela status schema vela.status.v4.".into(),
                VelaInspectionDto {
                    binary: Some(binary),
                    status: Some(status),
                    claims,
                    integration: None,
                    refusal,
                },
            ))
        }
        Envelope::Failure(status_error) => {
            let check_output = run_json(
                binary_path,
                repository,
                &["integration", "check", "<repo>", "--json"],
            )?;
            match parse_envelope::<IntegrationCheckV1Wire>(
                &check_output,
                "vela.cli.integration-check.v1",
                "integration check",
            )? {
                Envelope::Success(check) => {
                    let inspect_output = run_json(
                        binary_path,
                        repository,
                        &["integration", "inspect", "<repo>", "--json"],
                    )?;
                    let inspection = match parse_envelope::<IntegrationInspectionV1Wire>(
                        &inspect_output,
                        "vela.cli.integration-inspection.v1",
                        "integration inspect",
                    )? {
                        Envelope::Success(inspection) => inspection,
                        Envelope::Failure(error) => {
                            return Err(PortError::Parse(format!(
                                "integration check succeeded but inspect refused: {}",
                                error.error.message
                            )));
                        }
                    };
                    Ok((
                        RepositoryClassificationDto::NativeIntegration,
                        "Validated by signed Vela native integration schemas; authority_effect is none.".into(),
                        VelaInspectionDto {
                            binary: Some(binary),
                            status: None,
                            claims: Vec::new(),
                            integration: Some(validate_integration(check, inspection)?),
                            refusal: None,
                        },
                    ))
                }
                Envelope::Failure(integration_error) => {
                    let authoritative = integration_error.error.code.as_deref()
                        == Some("native_integration_manifest_required");
                    let classification = if authoritative {
                        RepositoryClassificationDto::VelaRepository
                    } else {
                        RepositoryClassificationDto::GitOnly
                    };
                    let basis = if authoritative {
                        "Repository authority was recognized, but status inspection refused."
                    } else {
                        "Signed Vela found neither a readable authority Repository nor a valid native integration."
                    };
                    Ok((
                        classification,
                        basis.into(),
                        VelaInspectionDto {
                            binary: Some(binary),
                            status: None,
                            claims: Vec::new(),
                            integration: None,
                            refusal: Some(refusal("status", status_error)),
                        },
                    ))
                }
            }
        }
    }
}

const SUBMISSION_AUTHORITY_BOUNDARY: &str = "This producer-authenticated Submission creates a pending Proposal only. Repository authority is separate; no Verification, Decision, Event, Standing, or acceptance action is available here.";

fn bounded_text(value: &str, label: &str) -> Result<String, PortError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 16_384 || trimmed.chars().any(|value| value == '\0') {
        return Err(PortError::InvalidInput(format!(
            "{label} must be non-empty, trimmed, and at most 16384 bytes"
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_digest(value: &str, label: &str) -> Result<(), PortError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(PortError::InvalidInput(format!(
            "{label} must be a full sha256 digest"
        )));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(PortError::InvalidInput(format!(
            "{label} must be a lowercase full sha256 digest"
        )));
    }
    Ok(())
}

fn validate_relative_artifact(path: &str) -> Result<(), PortError> {
    let path = Path::new(path);
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || path.as_os_str().len() > 4096
        || path.to_string_lossy().contains(':')
        || !path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(PortError::InvalidInput(
            "Submission Artifact paths must be normalized repository-relative files".into(),
        ));
    }
    Ok(())
}

fn read_submission_artifact(
    repository: &Path,
    path: &str,
    expected_digest: &str,
) -> Result<(u64, String), PortError> {
    validate_relative_artifact(path)?;
    validate_digest(expected_digest, "Artifact digest")?;
    let source = repository.join(path);
    let metadata = std::fs::symlink_metadata(&source).map_err(|error| {
        PortError::InvalidInput(format!("inspect Submission Artifact {path}: {error}"))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(format!(
            "Submission Artifact {path} must be a regular non-symlink file"
        )));
    }
    if metadata.len() > super::evidence::MAX_EVIDENCE_BYTES {
        return Err(PortError::Unsupported(format!(
            "Submission Artifact {path} exceeds the Workbench exact-display limit"
        )));
    }
    let observed = format!("sha256:{}", sha256(&source)?);
    if observed != expected_digest {
        return Err(PortError::Unsupported(format!(
            "Submission Artifact {path} changed after capture"
        )));
    }
    Ok((metadata.len(), observed))
}

fn validate_draft(repository: &Path, draft: &SubmissionDraftDto) -> Result<u64, PortError> {
    bounded_text(&draft.assertion, "Result assertion")?;
    if draft.conditions.len() > 32
        || draft.verification_requirements.len() > 32
        || draft.producer_check_run_ids.len() > 16
    {
        return Err(PortError::InvalidInput(
            "Submission draft exceeds bounded condition, verification-requirement, or producer-check counts"
                .into(),
        ));
    }
    if !matches!(
        draft.claim_type.as_str(),
        "computational" | "theoretical" | "empirical" | "negative" | "contradiction"
    ) {
        return Err(PortError::InvalidInput("unsupported Result type".into()));
    }
    if !matches!(
        draft.replayability.as_str(),
        "exact" | "bounded" | "approximate" | "unavailable" | "unknown"
    ) {
        return Err(PortError::InvalidInput("unsupported replayability".into()));
    }
    if draft.artifacts.is_empty() || draft.artifacts.len() > 32 {
        return Err(PortError::InvalidInput(
            "Submission draft must bind between 1 and 32 explicit Artifacts".into(),
        ));
    }
    if draft.caveats.is_empty() || draft.caveats.len() > 32 {
        return Err(PortError::InvalidInput(
            "Submission draft must state between 1 and 32 caveats".into(),
        ));
    }
    for value in draft
        .conditions
        .iter()
        .chain(&draft.caveats)
        .chain(&draft.verification_requirements)
    {
        bounded_text(value, "Submission text field")?;
    }
    if !draft.producer.starts_with("agent:")
        || draft.producer.len() > 16_384
        || draft
            .producer
            .chars()
            .any(|value| value.is_whitespace() || value.is_control())
    {
        return Err(PortError::InvalidInput(
            "direct authoring producer must be one explicit agent:<id> identity".into(),
        ));
    }
    if let Some(source_run) = &draft.source_run {
        bounded_text(source_run, "source run")?;
    }
    let mut total = 0_u64;
    for artifact in &draft.artifacts {
        bounded_text(&artifact.kind, "Artifact kind")?;
        if artifact.kind.len() > 128
            || !artifact
                .kind
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(PortError::InvalidInput(
                "Artifact kind must use at most 128 ASCII letters, digits, dot, underscore, or dash"
                    .into(),
            ));
        }
        let (size, _) = read_submission_artifact(repository, &artifact.path, &artifact.sha256)?;
        if size != artifact.size {
            return Err(PortError::Unsupported(format!(
                "Submission Artifact {} size changed after capture",
                artifact.path
            )));
        }
        total = total
            .checked_add(size)
            .ok_or_else(|| PortError::Unsupported("Artifact size total overflowed".into()))?;
    }
    Ok(total)
}

pub(crate) fn preview_submission_draft(
    repository: &Path,
    binary: &Path,
    git: &crate::contracts::GitSnapshotDto,
    draft: SubmissionDraftDto,
    producer_checks: Vec<String>,
) -> Result<SubmissionPreviewDto, PortError> {
    let identity = inspect_binary(binary)?;
    if identity.state != VelaBinaryStateDto::SignedRuntimeBaseline {
        return Err(PortError::Unsupported(
            "Submission intake requires the pinned signed Vela runtime".into(),
        ));
    }
    let artifact_total_bytes = validate_draft(repository, &draft)?;
    if producer_checks.len() != draft.producer_check_run_ids.len()
        || producer_checks.len() > 16
        || producer_checks.iter().any(|value| {
            value.len() > 16_384
                || value.trim() != value
                || !value.contains(':')
                || value.chars().any(|character| character == '\0')
        })
    {
        return Err(PortError::InvalidInput(
            "producer checks must resolve exactly from the selected completed runs".into(),
        ));
    }
    let mut argv = vec![
        identity.path.clone(),
        "submit".into(),
        "--repo".into(),
        git.root.clone(),
    ];
    argv.extend(["--claim".into(), draft.assertion.clone()]);
    argv.extend(["--type".into(), draft.claim_type.clone()]);
    for condition in &draft.conditions {
        argv.extend(["--condition".into(), condition.clone()]);
    }
    argv.extend(["--replayability".into(), draft.replayability.clone()]);
    for artifact in &draft.artifacts {
        argv.extend([
            "--artifact".into(),
            format!("{}:{}", artifact.path, artifact.kind),
        ]);
    }
    for caveat in &draft.caveats {
        argv.extend(["--caveat".into(), caveat.clone()]);
    }
    for check in &producer_checks {
        argv.extend(["--check".into(), check.clone()]);
    }
    for requirement in &draft.verification_requirements {
        argv.extend(["--requires-verification".into(), requirement.clone()]);
    }
    if let Some(source_run) = &draft.source_run {
        argv.extend(["--source-run".into(), source_run.clone()]);
    }
    argv.extend(["--as".into(), draft.producer.clone(), "--json".into()]);
    Ok(SubmissionPreviewDto {
        draft,
        repository_path: git.root.clone(),
        source_commit: git.head_commit.clone(),
        source_tree: git.head_tree.clone(),
        vela_binary_sha256: format!("sha256:{}", identity.sha256),
        argv,
        artifact_total_bytes,
        producer_checks,
        authority_boundary: SUBMISSION_AUTHORITY_BOUNDARY.into(),
        warning: "Vela will sign as the displayed producer and make one ordinary local Git commit if repository preconditions pass. No network publication occurs.".into(),
    })
}

fn run_submit(
    binary: &Path,
    repository: &Path,
    args: &[String],
) -> Result<SubmissionResultDto, PortError> {
    if !accepted_runtime_sha256(&sha256(binary)?) {
        return Err(PortError::Unsupported(
            "selected Vela executable changed before Submission intake".into(),
        ));
    }
    let mut spec = ProcessSpec::new(binary, repository).args(args);
    spec.timeout = Duration::from_secs(120);
    spec.max_stdout = 512 * 1024;
    spec.max_stderr = 256 * 1024;
    let output = run_bounded(spec)?;
    if !accepted_runtime_sha256(&sha256(binary)?) {
        return Err(PortError::Unsupported(
            "selected Vela executable changed during Submission intake".into(),
        ));
    }
    match parse_envelope::<SubmitResultV1Wire>(&output, "vela.submit-result.v1", "submit")? {
        Envelope::Failure(error) => Err(PortError::Process(format!(
            "Vela refused Submission intake: {}",
            error.error.message
        ))),
        Envelope::Success(result) => validate_submit_result(result),
    }
}

fn validate_submit_result(result: SubmitResultV1Wire) -> Result<SubmissionResultDto, PortError> {
    if result.schema != "vela.submit-result.v1"
        || result.route != "pending_review"
        || result.accepted_event_count_before != result.accepted_event_count_after
        || result.accepted_event_delta != 0
        || result.accepted_state_changed
    {
        return Err(PortError::Parse(
            "Vela submit result crossed the bounded producer-only authority contract".into(),
        ));
    }
    let (publication_state, publication_commit) = match result.publication {
        PublicationWire::Unchanged { commit } => ("unchanged".into(), Some(commit)),
        PublicationWire::Uncommitted { candidate, reason } => {
            if reason.trim().is_empty() {
                return Err(PortError::Parse(
                    "uncommitted publication omitted its reason".into(),
                ));
            }
            ("uncommitted".into(), candidate)
        }
        PublicationWire::CommittedLocal { commit } => ("committed_local".into(), Some(commit)),
    };
    Ok(SubmissionResultDto {
        operation_id: result.operation_id,
        submission_id: result.submission_id,
        submission_root: result.submission_root,
        proposal_id: result.proposal_id,
        proposal_root: result.proposal_root,
        claim_id: result.claim_id,
        route: result.route,
        accepted_event_delta: result.accepted_event_delta,
        accepted_state_changed: result.accepted_state_changed,
        publication_state,
        publication_commit,
    })
}

pub(crate) fn submit_draft(
    binary: &Path,
    preview: &SubmissionPreviewDto,
) -> Result<SubmissionResultDto, PortError> {
    let [program, args @ ..] = preview.argv.as_slice() else {
        return Err(PortError::InvalidInput(
            "Submission preview omitted argv".into(),
        ));
    };
    if Path::new(program) != binary {
        return Err(PortError::Unsupported(
            "Submission preview Vela path changed".into(),
        ));
    }
    run_submit(binary, Path::new(&preview.repository_path), args)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DsseEnvelopeWire {
    payload_type: String,
    payload: String,
    signatures: Vec<DsseSignatureWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DsseSignatureWire {
    keyid: String,
    sig: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmissionPayloadPreviewWire {
    schema: String,
    identity: SubmissionIdentityPreviewWire,
    claim: SubmissionClaimPreviewWire,
    artifacts: Vec<SubmissionArtifactPreviewWire>,
    caveats: Vec<String>,
    replayability: String,
    producer_checks: Vec<serde_json::Value>,
    verification_requirements: Vec<String>,
    requested_change: serde_json::Value,
    provenance: SubmissionProvenancePreviewWire,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmissionIdentityPreviewWire {
    schema: String,
    actor_id: String,
    actor_class: String,
    public_key_hex: String,
    declared_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmissionClaimPreviewWire {
    assertion: String,
    #[serde(rename = "type")]
    claim_type: String,
    conditions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmissionArtifactPreviewWire {
    kind: String,
    path: String,
    digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmissionProvenancePreviewWire {
    producer: String,
    source_system: String,
    source_run: Option<String>,
    emitted_at: String,
}

fn read_envelope(path: &Path) -> Result<(PathBuf, Vec<u8>), PortError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        PortError::InvalidInput(format!("inspect Submission envelope: {error}"))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "Submission envelope must be one regular non-symlink file".into(),
        ));
    }
    if metadata.len() > super::evidence::MAX_EVIDENCE_BYTES {
        return Err(PortError::Unsupported(
            "Submission envelope exceeds the Workbench exact-display limit".into(),
        ));
    }
    let canonical = std::fs::canonicalize(path).map_err(|error| {
        PortError::InvalidInput(format!("resolve Submission envelope: {error}"))
    })?;
    let bytes = std::fs::read(&canonical)
        .map_err(|error| PortError::InvalidInput(format!("read Submission envelope: {error}")))?;
    Ok((canonical, bytes))
}

pub(crate) fn preview_submission_import(
    repository: &Path,
    binary: &Path,
    git: &crate::contracts::GitSnapshotDto,
    envelope_path: &Path,
) -> Result<SubmissionImportPreviewDto, PortError> {
    let identity = inspect_binary(binary)?;
    if identity.state != VelaBinaryStateDto::SignedRuntimeBaseline {
        return Err(PortError::Unsupported(
            "Submission import requires the pinned signed Vela runtime".into(),
        ));
    }
    let (canonical, envelope_bytes) = read_envelope(envelope_path)?;
    let envelope: DsseEnvelopeWire = serde_json::from_slice(&envelope_bytes).map_err(|error| {
        PortError::Parse(format!("Submission envelope JSON is invalid: {error}"))
    })?;
    if envelope.payload_type != "application/vnd.vela.submission.v3+json"
        || envelope.signatures.is_empty()
        || envelope
            .signatures
            .iter()
            .any(|signature| signature.keyid.trim().is_empty() || signature.sig.trim().is_empty())
    {
        return Err(PortError::Parse(
            "Submission envelope does not have the required v3 DSSE shape".into(),
        ));
    }
    let payload_bytes = STANDARD
        .decode(&envelope.payload)
        .map_err(|_| PortError::Parse("Submission DSSE payload is not valid base64".into()))?;
    let payload: SubmissionPayloadPreviewWire = serde_json::from_slice(&payload_bytes)
        .map_err(|error| PortError::Parse(format!("Submission v3 payload is invalid: {error}")))?;
    if payload.schema != "vela.submission.v3"
        || payload.identity.schema != "vela.signer-identity.v1"
        || payload.identity.actor_id != payload.provenance.producer
        || payload.identity.actor_class.trim().is_empty()
        || payload.identity.public_key_hex.len() != 64
        || payload.identity.declared_at.trim().is_empty()
        || payload.provenance.source_system.trim().is_empty()
        || payload.provenance.emitted_at.trim().is_empty()
        || !matches!(
            payload.replayability.as_str(),
            "exact" | "bounded" | "approximate" | "unavailable" | "unknown"
        )
        || payload.artifacts.is_empty()
        || payload.artifacts.len() > 32
        || payload.claim.conditions.len() > 32
        || payload.caveats.is_empty()
        || payload.producer_checks.len() > 32
        || payload.verification_requirements.len() > 32
        || !payload.requested_change.is_object()
    {
        return Err(PortError::Parse(
            "Submission payload preview invariants are invalid".into(),
        ));
    }
    bounded_text(&payload.identity.actor_id, "signed producer identity")?;
    if payload.identity.actor_id.chars().any(char::is_whitespace) {
        return Err(PortError::Parse(
            "signed producer identity contains whitespace or control characters".into(),
        ));
    }
    bounded_text(&payload.claim.assertion, "signed Claim assertion")?;
    for value in payload
        .claim
        .conditions
        .iter()
        .chain(&payload.caveats)
        .chain(&payload.verification_requirements)
    {
        bounded_text(value, "signed Submission text field")?;
    }
    if let Some(source_run) = &payload.provenance.source_run {
        bounded_text(source_run, "source run")?;
    }
    let mut artifacts = Vec::new();
    for artifact in payload.artifacts {
        bounded_text(&artifact.kind, "signed Artifact kind")?;
        if artifact.kind.len() > 128
            || !artifact
                .kind
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(PortError::Parse(
                "signed Artifact kind is outside the bounded token grammar".into(),
            ));
        }
        validate_relative_artifact(&artifact.path)?;
        validate_digest(&artifact.digest, "Artifact digest")?;
        let repository_source = repository.join(&artifact.path);
        let source = if repository_source.is_file() {
            repository_source
        } else {
            let digest = artifact
                .digest
                .strip_prefix("sha256:")
                .expect("validated digest");
            canonical
                .parent()
                .ok_or_else(|| PortError::InvalidInput("Submission envelope has no parent".into()))?
                .join("artifacts")
                .join("sha256")
                .join(digest)
        };
        let metadata = std::fs::symlink_metadata(&source).map_err(|error| {
            PortError::InvalidInput(format!(
                "Submission transport Artifact {} is unavailable: {error}",
                artifact.path
            ))
        })?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > super::evidence::MAX_EVIDENCE_BYTES
            || format!("sha256:{}", sha256(&source)?) != artifact.digest
        {
            return Err(PortError::Unsupported(format!(
                "Submission transport Artifact {} is not the exact bounded regular file",
                artifact.path
            )));
        }
        artifacts.push(SubmissionArtifactDraftDto {
            path: artifact.path,
            kind: artifact.kind,
            sha256: artifact.digest,
            size: metadata.len(),
        });
    }
    Ok(SubmissionImportPreviewDto {
        envelope_path: canonical.display().to_string(),
        envelope_sha256: format!("sha256:{}", sha256(&canonical)?),
        envelope_size: envelope_bytes.len() as u64,
        envelope_base64: STANDARD.encode(&envelope_bytes),
        payload_type: envelope.payload_type,
        producer: payload.identity.actor_id,
        assertion: payload.claim.assertion,
        claim_type: payload.claim.claim_type,
        artifacts,
        repository_path: git.root.clone(),
        source_commit: git.head_commit.clone(),
        source_tree: git.head_tree.clone(),
        vela_binary_sha256: format!("sha256:{}", identity.sha256),
        authority_boundary: SUBMISSION_AUTHORITY_BOUNDARY.into(),
        warning: "The Workbench preview validates bounded v3 shape and exact transport bytes. The signed Vela CLI independently verifies the producer signature and repository preconditions before any import.".into(),
    })
}

pub(crate) fn import_submission(
    binary: &Path,
    preview: &SubmissionImportPreviewDto,
) -> Result<SubmissionResultDto, PortError> {
    let (_, current) = read_envelope(Path::new(&preview.envelope_path))?;
    let current_digest = Sha256::digest(&current)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if format!("sha256:{current_digest}") != preview.envelope_sha256
        || current.len() as u64 != preview.envelope_size
        || STANDARD.encode(&current) != preview.envelope_base64
    {
        return Err(PortError::Unsupported(
            "signed Submission envelope changed after preview".into(),
        ));
    }
    run_submit(
        binary,
        Path::new(&preview.repository_path),
        &[
            "submit".into(),
            preview.envelope_path.clone(),
            "--repo".into(),
            preview.repository_path.clone(),
            "--json".into(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{
        ClaimsV1Wire, DsseEnvelopeWire, Envelope, IntegrationCheckV1Wire,
        IntegrationInspectionV1Wire, PLATFORM_RUNTIME_SHA256, PublicationWire, RUNTIME_VERSION,
        StatusV4Wire, SubmissionPayloadPreviewWire, SubmitResultV1Wire, VelaBinaryStateDto,
        accepted_runtime_sha256, inspect_binary, parse_envelope, validate_claims,
        validate_integration, validate_status, validate_submit_result,
    };
    use crate::ports::ProcessOutput;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use sha2::{Digest, Sha256};

    fn fixture(path: &str) -> ProcessOutput {
        ProcessOutput {
            success: true,
            exit_code: Some(0),
            stdout: std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
                .expect("fixture exists"),
            stderr: Vec::new(),
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn digest_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn frozen_status_and_claims_validate_together() {
        let status = match parse_envelope::<StatusV4Wire>(
            &fixture("../fixtures/core/v0.977.2/status-math-v09772.json"),
            "vela.status.v4",
            "status",
        )
        .expect("status envelope")
        {
            Envelope::Success(value) => validate_status(value).expect("status invariants"),
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        let claims = match parse_envelope::<ClaimsV1Wire>(
            &fixture("../fixtures/core/v0.977.2/claims-math-v09772.json"),
            "vela.claims.v1",
            "claims",
        )
        .expect("claims envelope")
        {
            Envelope::Success(value) => validate_claims(value, &status).expect("claims invariants"),
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        assert_eq!(claims.len(), 3);
    }

    #[test]
    fn frozen_integration_preserves_non_authority() {
        let check = match parse_envelope::<IntegrationCheckV1Wire>(
            &fixture("../fixtures/core/v0.977.2/integration-check-lean-proofs-v09772.json"),
            "vela.cli.integration-check.v1",
            "integration check",
        )
        .expect("check envelope")
        {
            Envelope::Success(value) => value,
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        let inspection = match parse_envelope::<IntegrationInspectionV1Wire>(
            &fixture("../fixtures/core/v0.977.2/integration-inspect-lean-proofs-v09772.json"),
            "vela.cli.integration-inspection.v1",
            "integration inspect",
        )
        .expect("inspection envelope")
        {
            Envelope::Success(value) => value,
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        let view = validate_integration(check, inspection).expect("integration invariants");
        assert_eq!(view.authority_effect, "none");
        assert!(
            view.does_not_establish
                .iter()
                .any(|value| value == "Standing")
        );
    }

    #[test]
    fn frozen_release_manifests_bind_v09772_source_and_platform_binaries() {
        let manifest = |name: &str| -> serde_json::Value {
            let bytes = std::fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../fixtures/core/v0.977.2/release")
                    .join(name),
            )
            .expect("release manifest fixture");
            serde_json::from_slice(&bytes).expect("release manifest JSON")
        };
        let macos = manifest("vela-macos-aarch64.zip.release-manifest.json");
        let linux = manifest("vela-linux-x86_64.tar.gz.release-manifest.json");
        for value in [&macos, &linux] {
            assert_eq!(value["schema"], "vela.release-bundle-manifest.v1");
            assert_eq!(value["release"]["version"], "0.977.2");
            assert_eq!(
                value["source"]["commit"],
                "c1a34373c2cdd937ed34fd128174a66fa12be71a"
            );
            assert_eq!(
                value["source"]["tree"],
                "b9188626039cfc1a4d7d4098d1b7fc6a4a92ad55"
            );
        }
        assert_eq!(
            macos["binary"]["sha256"],
            "sha256:286ed839ea81b7ed283e04ea1823c1515ad242dcee02b424787b8daa667625e2"
        );
        assert_eq!(
            linux["binary"]["sha256"],
            "sha256:3e2e12ac3410aa4a62013d3d7e2ceb828504c7beaff09cf1d126bc2d7ba077cd"
        );
    }

    #[test]
    fn frozen_signed_submission_v3_binds_exact_transport_artifact() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures/core/v0.977.2/submission-bundle");
        let envelope_bytes = fs::read(root.join("submission.json")).expect("signed envelope");
        assert_eq!(
            digest_hex(&envelope_bytes),
            "f1669cdfa498ff85c162bce6173f04b39cdf7620fb198a19b45f6d932302204a"
        );
        let envelope: DsseEnvelopeWire =
            serde_json::from_slice(&envelope_bytes).expect("DSSE envelope shape");
        assert_eq!(
            envelope.payload_type,
            "application/vnd.vela.submission.v3+json"
        );
        assert!(!envelope.signatures.is_empty());
        let payload: SubmissionPayloadPreviewWire =
            serde_json::from_slice(&STANDARD.decode(&envelope.payload).expect("base64 payload"))
                .expect("Submission v3 payload");
        assert_eq!(payload.schema, "vela.submission.v3");
        assert_eq!(payload.identity.actor_id, payload.provenance.producer);
        let artifact = payload.artifacts.first().expect("bounded Artifact");
        let digest = artifact
            .digest
            .strip_prefix("sha256:")
            .expect("sha256 Artifact");
        let bytes =
            fs::read(root.join("artifacts/sha256").join(digest)).expect("transport Artifact");
        assert_eq!(digest_hex(&bytes), digest);
    }

    #[test]
    fn unsupported_schema_fails_closed() {
        let output = fixture("../tests/fixtures/hostile/unsupported-status.json");
        let error = parse_envelope::<StatusV4Wire>(&output, "vela.status.v4", "status")
            .expect_err("unsupported schema must refuse");
        assert_eq!(error.kind(), "unsupported");
    }

    #[test]
    fn submission_result_cannot_cross_into_acceptance_or_events() {
        let result = SubmitResultV1Wire {
            schema: "vela.submit-result.v1".into(),
            operation_id: "op".into(),
            submission_id: "vsb_test".into(),
            submission_root: format!("sha256:{}", "1".repeat(64)),
            proposal_id: "vpr_test".into(),
            proposal_root: format!("sha256:{}", "2".repeat(64)),
            claim_id: "vcl_test".into(),
            route: "pending_review".into(),
            accepted_event_count_before: 0,
            accepted_event_count_after: 1,
            accepted_event_delta: 1,
            accepted_state_changed: true,
            publication: PublicationWire::CommittedLocal {
                commit: "3".repeat(40),
            },
        };
        let error = validate_submit_result(result)
            .expect_err("Submission intake must refuse authority effects");
        assert!(
            error
                .to_string()
                .contains("producer-only authority contract")
        );
    }

    #[test]
    fn malformed_json_fails_before_semantic_rendering() {
        let output = fixture("../tests/fixtures/hostile/malformed.json");
        let error = parse_envelope::<StatusV4Wire>(&output, "vela.status.v4", "status")
            .expect_err("malformed JSON must refuse");
        assert_eq!(error.kind(), "parse");
    }

    #[test]
    fn prose_that_mimics_a_stable_code_does_not_create_one() {
        let mut output = fixture("../tests/fixtures/hostile/message-mimic-error.json");
        output.success = false;
        output.exit_code = Some(1);
        match parse_envelope::<StatusV4Wire>(&output, "vela.status.v4", "review.accept")
            .expect("valid error envelope")
        {
            Envelope::Failure(error) => assert!(error.error.code.is_none()),
            Envelope::Success(_) => panic!("error fixture became a success"),
        }
    }

    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64")
    ))]
    #[test]
    fn runtime_policy_accepts_only_signed_v09772_for_this_platform() {
        assert_eq!(RUNTIME_VERSION, "vela 0.977.2");
        assert!(accepted_runtime_sha256(PLATFORM_RUNTIME_SHA256));
        assert!(!accepted_runtime_sha256(
            "4332427789bf3dac83ebad9843670047b448f6ba370661f48a0100cbb61bc00c"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn unrecognized_executable_symlink_is_hashed_without_execution() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let temp = tempfile::tempdir().expect("tempdir");
        let executable = temp.path().join("malicious-vela");
        let marker = temp.path().join("executed");
        fs::write(
            &executable,
            format!(
                "#!/bin/sh\ntouch '{}'\necho vela 0.977.2\n",
                marker.display()
            ),
        )
        .expect("script");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).expect("executable");
        let selected = temp.path().join("vela-link");
        symlink(&executable, &selected).expect("symlink");
        let identity = inspect_binary(&selected).expect("identity response");
        assert_eq!(identity.state, VelaBinaryStateDto::Unsupported);
        assert_eq!(
            identity.path,
            executable
                .canonicalize()
                .expect("canonical")
                .display()
                .to_string()
        );
        assert!(!marker.exists(), "unrecognized executable must never run");
    }
}
