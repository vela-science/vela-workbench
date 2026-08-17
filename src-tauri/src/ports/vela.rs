use std::{
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::contracts::{
    ClaimDto, ClaimsV1Wire, ErrorEnvelopeWire, IntegrationCheckV1Wire, IntegrationDto,
    IntegrationInspectionV1Wire, IntegrationItemDto, RefusalDto, RepositoryClassificationDto,
    StatusCountsDto, StatusV4Wire, VelaBinaryDto, VelaBinaryStateDto, VelaInspectionDto,
    VelaStatusDto,
};

use super::{
    PortError, ProcessOutput, ProcessSpec, ensure_not_truncated, process::utf8, run_bounded,
};

pub(crate) const INTERFACE_COMMIT: &str = "3bfcf23f12fb6a38a924a257ba25ad3d8594dc78";
pub(crate) const INTERFACE_TREE: &str = "ab85ef6ec7f6cd7c49fc4664bbbbd4f597e71816";
pub(crate) const RUNTIME_VERSION: &str = "vela 0.977.0";
pub(crate) const RUNTIME_COMMIT: &str = "00d567c879138733ba22949efc985b54578c148b";
pub(crate) const MACOS_ARM64_RUNTIME_SHA256: &str =
    "4332427789bf3dac83ebad9843670047b448f6ba370661f48a0100cbb61bc00c";

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

pub(crate) fn inspect_binary(path: &Path) -> Result<VelaBinaryDto, PortError> {
    let path = executable(path)?;
    let digest = sha256(&path)?;
    if digest != MACOS_ARM64_RUNTIME_SHA256 {
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
    let state = if version != RUNTIME_VERSION {
        VelaBinaryStateDto::Unsupported
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        VelaBinaryStateDto::SignedRuntimeBaseline
    } else {
        VelaBinaryStateDto::VersionMatchHashUnrecognized
    };
    Ok(VelaBinaryDto {
        path: path.display().to_string(),
        version,
        sha256: post_digest,
        state,
    })
}

fn run_json(binary: &Path, repository: &Path, args: &[&str]) -> Result<ProcessOutput, PortError> {
    if sha256(binary)? != MACOS_ARM64_RUNTIME_SHA256 {
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
    if sha256(binary)? != MACOS_ARM64_RUNTIME_SHA256 {
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
                    message: "Select the installed signed Vela v0.977.0 executable to classify this repository.".into(),
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
                    message: "Runtime execution is pinned to signed Vela v0.977.0 with the reviewed platform hash.".into(),
                    hint: Some("Choose the installed signed release binary. Merged Core main is an interface target, not a runtime.".into()),
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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{
        ClaimsV1Wire, Envelope, IntegrationCheckV1Wire, IntegrationInspectionV1Wire, StatusV4Wire,
        VelaBinaryStateDto, inspect_binary, parse_envelope, validate_claims, validate_integration,
        validate_status,
    };
    use crate::ports::ProcessOutput;

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

    #[test]
    fn frozen_status_and_claims_validate_together() {
        let status = match parse_envelope::<StatusV4Wire>(
            &fixture("../fixtures/core/3bfcf23f/status-math-v0977.json"),
            "vela.status.v4",
            "status",
        )
        .expect("status envelope")
        {
            Envelope::Success(value) => validate_status(value).expect("status invariants"),
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        let claims = match parse_envelope::<ClaimsV1Wire>(
            &fixture("../fixtures/core/3bfcf23f/claims-math-v0977.json"),
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
            &fixture("../fixtures/core/3bfcf23f/integration-check-lean-proofs-v0977.json"),
            "vela.cli.integration-check.v1",
            "integration check",
        )
        .expect("check envelope")
        {
            Envelope::Success(value) => value,
            Envelope::Failure(_) => panic!("fixture unexpectedly refused"),
        };
        let inspection = match parse_envelope::<IntegrationInspectionV1Wire>(
            &fixture("../fixtures/core/3bfcf23f/integration-inspect-lean-proofs-v0977.json"),
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
    fn unsupported_schema_fails_closed() {
        let output = fixture("../tests/fixtures/hostile/unsupported-status.json");
        let error = parse_envelope::<StatusV4Wire>(&output, "vela.status.v4", "status")
            .expect_err("unsupported schema must refuse");
        assert_eq!(error.kind(), "unsupported");
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
                "#!/bin/sh\ntouch '{}'\necho vela 0.977.0\n",
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
