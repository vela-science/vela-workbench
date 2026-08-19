use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::contracts::{
    DecisionActionDto, DecisionBlockerDto, DecisionEntryDto, DecisionExecutionDto,
    DecisionInboxDto, DecisionPreviewDto, DecisionReadbackDto, DecisionRequestDto, GitSnapshotDto,
    PublicationWire, RecoveryPreviewDto, RecoveryResultDto, StandingDeltaDto, StandingStateDto,
    StructuredVelaRefusalDto, VerificationDraftDto, VerificationFacetDto,
    VerificationImportPreviewDto, VerificationMethodDto, VerificationPreviewDto,
    VerificationResultDto,
};

use super::{PortError, ProcessSpec, ensure_not_truncated, run_bounded};

const MAX_JSON: usize = 4 * 1024 * 1024;
const MAX_METHOD: u64 = 1024 * 1024;
const MAX_ENVELOPE: u64 = 4 * 1024 * 1024;
const MAX_ITEMS: usize = 32;

enum CliOutcome {
    Success(Value),
    Refusal(StructuredVelaRefusalDto),
}

fn digest(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn exact_binary(binary: &Path) -> Result<crate::contracts::VelaBinaryDto, PortError> {
    let identity = super::vela::inspect_binary(binary)?;
    if identity.state != crate::contracts::VelaBinaryStateDto::SignedRuntimeBaseline {
        return Err(PortError::Unsupported(
            "Vela Repository actions require the exact signed Vela v0.977.3 runtime".into(),
        ));
    }
    Ok(identity)
}

fn run_cli(
    binary: &Path,
    repository: &Path,
    args: &[String],
    expected_command: &str,
    success_schema: &str,
    authority: bool,
) -> Result<CliOutcome, PortError> {
    let before = exact_binary(binary)?;
    let mut spec = ProcessSpec::new(binary, repository).args(args.iter().map(OsString::from));
    spec.timeout = Duration::from_secs(if authority { 90 } else { 30 });
    spec.max_stdout = MAX_JSON;
    spec.max_stderr = 512 * 1024;
    spec.include_ssh_auth_sock = authority;
    let output = run_bounded(spec)?;
    let after = exact_binary(binary)?;
    if before.path != after.path || before.sha256 != after.sha256 {
        return Err(PortError::Unsupported(
            "selected Vela executable changed during a Repository action".into(),
        ));
    }
    ensure_not_truncated(&output, expected_command)?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
        PortError::Parse(format!(
            "{expected_command} returned malformed JSON: {error}"
        ))
    })?;
    let schema = string(&value, "schema")?;
    if schema == "vela.error.v1" {
        if output.success || value.get("ok").and_then(Value::as_bool) != Some(false) {
            return Err(PortError::Parse(format!(
                "{expected_command} error envelope disagrees with process status"
            )));
        }
        let command = string(&value, "command")?;
        if command != expected_command {
            return Err(PortError::Parse(format!(
                "{expected_command} error envelope named {command}"
            )));
        }
        let error = object(&value, "error")?;
        return Ok(CliOutcome::Refusal(StructuredVelaRefusalDto {
            kind: string(error, "kind")?.into(),
            code: optional_string(error, "code")?,
            message: string(error, "message")?.into(),
            hint: optional_string(error, "hint")?,
            command: command.into(),
            operation_id: optional_string(&value, "operation_id")?,
            changed: value.get("changed").and_then(Value::as_bool),
            next: optional_string(&value, "next")?,
        }));
    }
    if !output.success || schema != success_schema {
        return Err(PortError::Unsupported(format!(
            "{expected_command} returned schema {schema}; expected {success_schema}"
        )));
    }
    Ok(CliOutcome::Success(value))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, PortError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| PortError::Parse(format!("Vela JSON omitted string {key}")))
}

fn optional_string(value: &Value, key: &str) -> Result<Option<String>, PortError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        _ => Err(PortError::Parse(format!(
            "Vela JSON field {key} was not a nullable string"
        ))),
    }
}

fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Value, PortError> {
    value
        .get(key)
        .filter(|item| item.is_object())
        .ok_or_else(|| PortError::Parse(format!("Vela JSON omitted object {key}")))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, PortError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| PortError::Parse(format!("Vela JSON omitted array {key}")))
}

fn strings(value: &Value, key: &str) -> Result<Vec<String>, PortError> {
    let items = array(value, key)?;
    if items.len() > MAX_ITEMS {
        return Err(PortError::Parse(format!(
            "Vela JSON {key} exceeded {MAX_ITEMS}"
        )));
    }
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| PortError::Parse(format!("Vela JSON {key} item was not text")))
        })
        .collect()
}

fn bounded_token(value: &str, label: &str) -> Result<(), PortError> {
    if value.is_empty() || value.len() > 1024 || value.chars().any(|ch| ch.is_control()) {
        return Err(PortError::InvalidInput(format!(
            "{label} is not bounded text"
        )));
    }
    Ok(())
}

pub(crate) fn validate_operation_id(value: &str) -> Result<(), PortError> {
    if !super::valid_recovery_operation_id(value) {
        return Err(PortError::InvalidInput(
            "recovery operation id must use vop_ with one lowercase 64-character digest".into(),
        ));
    }
    Ok(())
}

fn relative_path(
    repository: &Path,
    selected: &Path,
    label: &str,
) -> Result<(PathBuf, String), PortError> {
    let repository = std::fs::canonicalize(repository)
        .map_err(|error| PortError::InvalidInput(format!("resolve repository: {error}")))?;
    let selected = std::fs::canonicalize(selected)
        .map_err(|error| PortError::InvalidInput(format!("resolve {label}: {error}")))?;
    if !selected.starts_with(&repository) || selected == repository {
        return Err(PortError::InvalidInput(format!(
            "{label} must be inside the selected repository"
        )));
    }
    let metadata = std::fs::symlink_metadata(&selected)
        .map_err(|error| PortError::InvalidInput(format!("inspect {label}: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(format!(
            "{label} must be one regular non-symlink file"
        )));
    }
    let relative = selected.strip_prefix(&repository).expect("contained path");
    if !relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(PortError::InvalidInput(format!(
            "{label} path is not normalized"
        )));
    }
    let text = relative
        .to_str()
        .ok_or_else(|| PortError::InvalidInput(format!("{label} path must be UTF-8")))?
        .replace('\\', "/");
    Ok((selected, text))
}

fn read_bounded(path: &Path, limit: u64, label: &str) -> Result<Vec<u8>, PortError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| PortError::InvalidInput(format!("inspect {label}: {error}")))?;
    if metadata.len() == 0 || metadata.len() > limit {
        return Err(PortError::InvalidInput(format!(
            "{label} must contain 1..={limit} bytes"
        )));
    }
    std::fs::read(path).map_err(|error| PortError::InvalidInput(format!("read {label}: {error}")))
}

type ReviewMethodFields = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
);

fn review_method_fields(value: &Value) -> Result<ReviewMethodFields, PortError> {
    if string(value, "schema")? != "vela.review-method.v1" {
        return Err(PortError::Unsupported(
            "Verification Method must use vela.review-method.v1".into(),
        ));
    }
    let reviewer = object(value, "reviewer")?;
    Ok((
        string(value, "profile")?.into(),
        string(value, "property")?.into(),
        Some(string(reviewer, "kind")?.into()),
        Some(string(reviewer, "display_name")?.into()),
        Some(string(reviewer, "identifier")?.into()),
        optional_string(reviewer, "provider")?,
        optional_string(reviewer, "version")?,
        strings(value, "procedure")?,
        strings(value, "required_output")?,
        strings(value, "does_not_establish")?,
    ))
}

pub(crate) fn inspect_verification_method(
    repository: &Path,
    selected: &Path,
) -> Result<VerificationMethodDto, PortError> {
    let (path, relative) = relative_path(repository, selected, "Verification Method")?;
    let bytes = read_bounded(&path, MAX_METHOD, "Verification Method")?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PortError::Parse(format!("parse Verification Method: {error}")))?;
    let (
        profile,
        property,
        reviewer_kind,
        reviewer_display_name,
        reviewer_identifier,
        provider,
        version,
        procedure,
        required_output,
        does_not_establish,
    ) = review_method_fields(&value)?;
    Ok(VerificationMethodDto {
        path: path.display().to_string(),
        repository_relative_path: relative,
        sha256: digest(&bytes),
        size: bytes.len() as u64,
        profile,
        property,
        reviewer_kind,
        reviewer_display_name,
        reviewer_identifier,
        provider,
        version,
        attested_by_actor_id: optional_string(&value, "attested_by_actor_id")?,
        procedure,
        required_output,
        does_not_establish,
    })
}

fn standing_state(value: &Value) -> Result<StandingStateDto, PortError> {
    let accepted = array(value, "accepted")?;
    if accepted.len() > MAX_ITEMS {
        return Err(PortError::Parse(
            "Standing delta exceeded bounded Claim count".into(),
        ));
    }
    Ok(StandingStateDto {
        repository_root: string(value, "repository_root")?.into(),
        accepted_claim_ids: accepted
            .iter()
            .map(|item| string(item, "claim_id").map(str::to_owned))
            .collect::<Result<_, _>>()?,
    })
}

fn basic_verification(value: &Value) -> Result<VerificationFacetDto, PortError> {
    Ok(VerificationFacetDto {
        verification_record_id: string(value, "verification_record_id")?.into(),
        verification_record_root: string(value, "verification_record_root")?.into(),
        verifier: string(value, "verifier")?.into(),
        performer_kind: None,
        performer_identifier: None,
        provider: None,
        version: None,
        method_metadata_status: "not_loaded_from_signed_show".into(),
        method_profile: String::new(),
        method_path: String::new(),
        environment_root: String::new(),
        property: string(value, "property")?.into(),
        outcome: string(value, "outcome")?.into(),
        declared_independent_of: Vec::new(),
        shared_dependencies: Vec::new(),
        evidence_artifact_ids: Vec::new(),
        output_artifact_ids: Vec::new(),
        does_not_establish: strings(value, "does_not_establish")?,
        protocol_evidence_role: optional_string(value, "protocol_evidence_role")?,
        satisfies_requirements: strings(value, "satisfies_requirements")?,
    })
}

fn entry(value: &Value) -> Result<DecisionEntryDto, PortError> {
    if string(value, "schema")? != "vela.decision-inbox-entry.v3" {
        return Err(PortError::Unsupported(
            "Decision Inbox entry schema is unsupported".into(),
        ));
    }
    let inputs = object(value, "inputs")?;
    let readiness = object(value, "readiness")?;
    let delta = object(value, "standing_delta")?;
    let scope = object(delta, "scope")?;
    let counts = object(object(delta, "counts")?, "global_accepted_claims")?;
    let heads = object(value, "authority_heads")?;
    let verifications = array(value, "verification_records")?;
    if verifications.len() > MAX_ITEMS {
        return Err(PortError::Parse(
            "Decision Inbox Verification list exceeded bound".into(),
        ));
    }
    Ok(DecisionEntryDto {
        proposal_id: string(value, "proposal_id")?.into(),
        proposal_root: string(inputs, "proposal_root")?.into(),
        submission_root: string(inputs, "submission_root")?.into(),
        claim_id: string(value, "claim_id")?.into(),
        claim_root: string(inputs, "claim_root")?.into(),
        repository_root: string(inputs, "repository_root")?.into(),
        verification_set_root: string(inputs, "verification_set_root")?.into(),
        entry_root: string(value, "entry_root")?.into(),
        assertion: string(value, "assertion")?.into(),
        proposal_actor: string(value, "proposal_actor")?.into(),
        proposal_action: string(value, "proposal_action")?.into(),
        proposal_reason: string(value, "proposal_reason")?.into(),
        created_at: string(value, "created_at")?.into(),
        protocol_gate: string(readiness, "protocol_gate")?.into(),
        blockers: array(readiness, "blockers")?
            .iter()
            .map(|blocker| {
                Ok(DecisionBlockerDto {
                    code: string(blocker, "code")?.into(),
                    detail: string(blocker, "detail")?.into(),
                    subject: optional_string(blocker, "subject")?,
                })
            })
            .collect::<Result<Vec<_>, PortError>>()?,
        rejection_available: readiness
            .get("rejection_available")
            .and_then(Value::as_bool)
            .ok_or_else(|| PortError::Parse("Decision Inbox omitted rejection_available".into()))?,
        verification_requirements: strings(value, "verification_requirements")?,
        verifications: verifications
            .iter()
            .map(basic_verification)
            .collect::<Result<_, _>>()?,
        limits: strings(value, "limits")?,
        standing_delta: StandingDeltaDto {
            transition: string(delta, "transition")?.into(),
            affected_claim_ids: strings(scope, "affected_claim_ids")?,
            before: standing_state(object(delta, "before")?)?,
            if_accept: standing_state(object(delta, "if_accept")?)?,
            if_reject: standing_state(object(delta, "if_reject")?)?,
            global_accepted_before: counts
                .get("before")
                .and_then(Value::as_u64)
                .ok_or_else(|| PortError::Parse("Standing counts omitted before".into()))?,
            global_accepted_if_accept: counts
                .get("if_accept")
                .and_then(Value::as_u64)
                .ok_or_else(|| PortError::Parse("Standing counts omitted if_accept".into()))?,
            global_accepted_if_reject: counts
                .get("if_reject")
                .and_then(Value::as_u64)
                .ok_or_else(|| PortError::Parse("Standing counts omitted if_reject".into()))?,
        },
        authority_keyset_root: string(heads, "authority_keyset_root")?.into(),
        policy_bundle_root: string(heads, "policy_bundle_root")?.into(),
        authority_record_root: string(heads, "authority_record_root")?.into(),
        authority_event_log_root: string(heads, "authority_event_log_root")?.into(),
    })
}

pub(crate) fn decision_inbox(
    repository: &Path,
    binary: &Path,
) -> Result<DecisionInboxDto, PortError> {
    let args = vec![
        "review".into(),
        "inbox".into(),
        "--repo".into(),
        repository.display().to_string(),
        "--json".into(),
    ];
    match run_cli(binary, repository, &args, "review.inbox", "vela.decision-inbox.v3", false)? {
        CliOutcome::Refusal(refusal) => Ok(DecisionInboxDto {
            repository_id: String::new(), repository_root: String::new(), projection_root: String::new(), entries: Vec::new(), observed_at_unix_ms: now_ms(),
            task: "Inspect the current Decision Inbox".into(),
            included_records: Vec::new(),
            omissions: vec!["Inbox unavailable; no Proposal, Verification, Decision, objection, or Standing fact was summarized.".into()],
            stale: true,
            refusal: Some(refusal),
        }),
        CliOutcome::Success(value) => parse_decision_inbox_value(&value),
    }
}

fn parse_decision_inbox_value(value: &Value) -> Result<DecisionInboxDto, PortError> {
    if value.get("ok").and_then(Value::as_bool) != Some(true)
        || string(value, "command")? != "review.inbox"
        || string(value, "schema")? != "vela.decision-inbox.v3"
    {
        return Err(PortError::Parse("Decision Inbox tags are invalid".into()));
    }
    let raw = array(value, "entries")?;
    if raw.len() > MAX_ITEMS {
        return Err(PortError::Parse(
            "Decision Inbox exceeded 32 entries".into(),
        ));
    }
    let entries = raw.iter().map(entry).collect::<Result<Vec<_>, _>>()?;
    Ok(DecisionInboxDto {
        repository_id: string(value, "repository_id")?.into(),
        repository_root: string(value, "repository_root")?.into(),
        projection_root: string(value, "projection_root")?.into(),
        included_records: entries
            .iter()
            .flat_map(|item| {
                std::iter::once(item.proposal_id.clone()).chain(
                    item.verifications
                        .iter()
                        .map(|verification| verification.verification_record_id.clone()),
                )
            })
            .collect(),
        entries,
        observed_at_unix_ms: now_ms(),
        task: "Inspect one exact Proposal for attributed accept or reject".into(),
        omissions: vec!["No source interpretation, hidden activity trajectory, unresolved external objection, or evidence outside the current signed Vela read surface is inferred.".into()],
        stale: false,
        refusal: None,
    })
}

fn verified_current_method(
    repository: &Path,
    method_path: &str,
    expected_environment_root: &str,
    expected_verifier: &str,
) -> Option<VerificationMethodDto> {
    super::git::require_tracked_clean_at_head(repository, method_path).ok()?;
    let inspected = inspect_verification_method(repository, &repository.join(method_path)).ok()?;
    (inspected.sha256 == expected_environment_root
        && inspected.attested_by_actor_id.as_deref() == Some(expected_verifier))
    .then_some(inspected)
}

fn detailed_verifications(
    repository: &Path,
    show: &Value,
    entry: &DecisionEntryDto,
) -> Result<Vec<VerificationFacetDto>, PortError> {
    let records = array(show, "verification_records")?;
    if records.len() > MAX_ITEMS {
        return Err(PortError::Parse(
            "review.show Verification list exceeded bound".into(),
        ));
    }
    records
        .iter()
        .map(|wrapper| {
            let record = object(wrapper, "record")?;
            let identity = object(record, "identity")?;
            let independence = object(record, "independence")?;
            let method = object(record, "method")?;
            let scope = object(record, "scope")?;
            let subject = object(record, "subject")?;
            let method_path = string(method, "implementation")?;
            let environment_root = string(method, "environment_root")?;
            let verifier = string(identity, "actor_id")?;
            let inspected =
                verified_current_method(repository, method_path, environment_root, verifier);
            let id = string(wrapper, "verification_record_id")?;
            let basic = entry
                .verifications
                .iter()
                .find(|item| item.verification_record_id == id);
            Ok(VerificationFacetDto {
                verification_record_id: id.into(),
                verification_record_root: string(wrapper, "verification_record_root")?.into(),
                verifier: verifier.into(),
                performer_kind: inspected
                    .as_ref()
                    .and_then(|item| item.reviewer_kind.clone())
                    .or_else(|| optional_string(identity, "actor_class").ok().flatten()),
                performer_identifier: inspected
                    .as_ref()
                    .and_then(|item| item.reviewer_identifier.clone()),
                provider: inspected.as_ref().and_then(|item| item.provider.clone()),
                version: inspected.as_ref().and_then(|item| item.version.clone()),
                method_metadata_status: if inspected.is_some() {
                    "verified_current"
                } else {
                    "unavailable_or_mismatched"
                }
                .into(),
                method_profile: string(method, "profile")?.into(),
                method_path: method_path.into(),
                environment_root: environment_root.into(),
                property: string(scope, "property")?.into(),
                outcome: string(record, "outcome")?.into(),
                declared_independent_of: strings(independence, "declared_independent_of")?,
                shared_dependencies: strings(independence, "shared_dependencies")?,
                evidence_artifact_ids: strings(subject, "artifact_ids")?,
                output_artifact_ids: strings(record, "output_artifact_ids")?,
                does_not_establish: strings(scope, "does_not_establish")?,
                protocol_evidence_role: basic.and_then(|item| item.protocol_evidence_role.clone()),
                satisfies_requirements: basic
                    .map(|item| item.satisfies_requirements.clone())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

fn review_show(
    repository: &Path,
    binary: &Path,
    proposal_id: &str,
) -> Result<CliOutcome, PortError> {
    let args = vec![
        "review".into(),
        "show".into(),
        "--repo".into(),
        repository.display().to_string(),
        proposal_id.into(),
        "--json".into(),
    ];
    run_cli(
        binary,
        repository,
        &args,
        "review.show",
        "vela.review.v1",
        false,
    )
}

fn validate_draft(draft: &VerificationDraftDto) -> Result<(), PortError> {
    bounded_token(&draft.proposal_id, "Proposal id")?;
    bounded_token(&draft.profile, "Method profile")?;
    bounded_token(&draft.actor, "Verification actor")?;
    if !matches!(
        draft.outcome.as_str(),
        "pass" | "fail" | "error" | "inconclusive"
    ) || draft.does_not_establish.is_empty()
        || draft.does_not_establish.len() > MAX_ITEMS
        || draft.independent_of.len() > MAX_ITEMS
        || draft.shared_dependencies.len() > MAX_ITEMS
        || draft.output_paths.len() > MAX_ITEMS
    {
        return Err(PortError::InvalidInput(
            "Verification draft exceeds closed values or item bounds".into(),
        ));
    }
    for (label, values) in [
        ("nonclaim", &draft.does_not_establish),
        ("independence", &draft.independent_of),
        ("shared dependency", &draft.shared_dependencies),
    ] {
        for value in values {
            bounded_token(value, label)?;
        }
    }
    Ok(())
}

fn imported_outcome(record: &Value) -> Result<String, PortError> {
    let outcome = string(record, "outcome")?;
    if !matches!(outcome, "pass" | "fail" | "error" | "inconclusive") {
        return Err(PortError::InvalidInput(
            "Verification envelope outcome is not a supported closed value".into(),
        ));
    }
    Ok(outcome.into())
}

fn current_import_subject<'a>(
    inbox: &'a DecisionInboxDto,
    proposal_id: &str,
    proposal_root: &str,
    submission_root: &str,
    claim_id: &str,
) -> Result<&'a DecisionEntryDto, PortError> {
    let current = inbox
        .entries
        .iter()
        .find(|entry| entry.proposal_id == proposal_id)
        .ok_or_else(|| {
            PortError::InvalidInput(
                "Verification envelope Proposal is not current in the Decision Inbox".into(),
            )
        })?;
    if current.proposal_root != proposal_root
        || current.submission_root != submission_root
        || current.claim_id != claim_id
        || current.repository_root != inbox.repository_root
    {
        return Err(PortError::InvalidInput(
            "Verification envelope subject does not match the exact current Proposal, Submission, Claim, and Repository roots".into(),
        ));
    }
    Ok(current)
}

pub(crate) fn preview_verification_record(
    repository: &Path,
    binary: &Path,
    git: &GitSnapshotDto,
    draft: VerificationDraftDto,
) -> Result<VerificationPreviewDto, PortError> {
    validate_draft(&draft)?;
    let method = inspect_verification_method(repository, Path::new(&draft.method.path))?;
    if method != draft.method
        || method.profile != draft.profile
        || method.attested_by_actor_id.as_deref() != Some(draft.actor.as_str())
    {
        return Err(PortError::InvalidInput(
            "Verification Method, profile, or attesting actor changed".into(),
        ));
    }
    super::git::require_tracked_clean_at_head(repository, &method.repository_relative_path)?;
    let inbox = decision_inbox(repository, binary)?;
    let current = inbox
        .entries
        .into_iter()
        .find(|item| item.proposal_id == draft.proposal_id)
        .ok_or_else(|| {
            PortError::InvalidInput("Proposal is not current in the Decision Inbox".into())
        })?;
    let mut argv = vec![
        "verification".into(),
        "record".into(),
        "--repo".into(),
        repository.display().to_string(),
        draft.proposal_id.clone(),
        "--profile".into(),
        draft.profile.clone(),
        "--method".into(),
        draft.method.repository_relative_path.clone(),
        "--outcome".into(),
        draft.outcome.clone(),
    ];
    if let Some(property) = &draft.property {
        bounded_token(property, "Verification property")?;
        argv.extend(["--property".into(), property.clone()]);
    }
    if draft.complementary {
        argv.push("--complementary".into());
    }
    for value in &draft.does_not_establish {
        argv.extend(["--does-not-establish".into(), value.clone()]);
    }
    for value in &draft.independent_of {
        argv.extend(["--independent-of".into(), value.clone()]);
    }
    for value in &draft.shared_dependencies {
        argv.extend(["--shared-dependency".into(), value.clone()]);
    }
    let mut selected_output_roots = Vec::new();
    for path in &draft.output_paths {
        let (absolute, relative) =
            relative_path(repository, &repository.join(path), "Verification output")?;
        super::git::require_tracked_clean_at_head(repository, &relative)?;
        selected_output_roots.push(digest(&read_bounded(
            &absolute,
            16 * 1024 * 1024,
            "Verification output",
        )?));
        argv.extend(["--output".into(), relative]);
    }
    argv.extend(["--as".into(), draft.actor.clone(), "--json".into()]);
    Ok(VerificationPreviewDto {
        draft, repository_path: repository.display().to_string(), source_commit: git.head_commit.clone(), source_tree: git.head_tree.clone(), repository_root: current.repository_root.clone(), proposal_root: current.proposal_root, submission_root: current.submission_root, claim_root: current.claim_root,
        vela_binary_sha256: exact_binary(binary)?.sha256,
        argv, selected_output_roots,
        authority_effect: "none".into(),
        warning: "This records one scoped Verification observation. It does not accept, reject, create an Event, or change Standing; declared independence and shared dependencies remain separate evidence facets.".into(),
    })
}

fn publication(value: &Value) -> Result<(String, Option<String>), PortError> {
    let publication: PublicationWire =
        serde_json::from_value(object(value, "publication")?.clone())
            .map_err(|error| PortError::Parse(format!("publication shape is invalid: {error}")))?;
    Ok(match publication {
        PublicationWire::Unchanged { commit } => ("unchanged".into(), Some(commit)),
        PublicationWire::Uncommitted { .. } => ("uncommitted".into(), None),
        PublicationWire::CommittedLocal { commit } => ("committed_local".into(), Some(commit)),
    })
}

fn verification_result(
    outcome: CliOutcome,
    proposal_id: &str,
) -> Result<VerificationResultDto, PortError> {
    match outcome {
        CliOutcome::Refusal(refusal) => Ok(VerificationResultDto {
            operation_id: refusal.operation_id.clone(),
            verification_record_id: None,
            verification_record_root: None,
            proposal_id: proposal_id.into(),
            claim_id: None,
            outcome: None,
            idempotent: None,
            accepted_event_delta: None,
            publication_state: None,
            publication_commit: None,
            refusal: Some(refusal),
        }),
        CliOutcome::Success(value) => {
            if string(&value, "proposal_id")? != proposal_id
                || value.get("accepted_event_delta").and_then(Value::as_u64) != Some(0)
            {
                return Err(PortError::Parse(
                    "Verification result changed authority or Proposal binding".into(),
                ));
            }
            let (publication_state, publication_commit) = publication(&value)?;
            Ok(VerificationResultDto {
                operation_id: Some(string(&value, "operation_id")?.into()),
                verification_record_id: Some(string(&value, "verification_record_id")?.into()),
                verification_record_root: Some(string(&value, "verification_record_root")?.into()),
                proposal_id: proposal_id.into(),
                claim_id: Some(string(&value, "claim_id")?.into()),
                outcome: Some(string(&value, "outcome")?.into()),
                idempotent: value.get("idempotent").and_then(Value::as_bool),
                accepted_event_delta: value.get("accepted_event_delta").and_then(Value::as_u64),
                publication_state: Some(publication_state),
                publication_commit,
                refusal: None,
            })
        }
    }
}

pub(crate) fn record_verification(
    repository: &Path,
    binary: &Path,
    preview: &VerificationPreviewDto,
) -> Result<VerificationResultDto, PortError> {
    let outcome = run_cli(
        binary,
        repository,
        &preview.argv,
        "verification.record",
        "vela.verification-import-result.v1",
        false,
    )?;
    verification_result(outcome, &preview.draft.proposal_id)
}

pub(crate) fn preview_verification_import(
    repository: &Path,
    binary: &Path,
    git: &GitSnapshotDto,
    selected: &Path,
) -> Result<VerificationImportPreviewDto, PortError> {
    let path = std::fs::canonicalize(selected).map_err(|error| {
        PortError::InvalidInput(format!("resolve Verification envelope: {error}"))
    })?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        PortError::InvalidInput(format!("inspect Verification envelope: {error}"))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "Verification envelope must be one regular non-symlink file".into(),
        ));
    }
    let bytes = read_bounded(&path, MAX_ENVELOPE, "Verification envelope")?;
    let envelope: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PortError::Parse(format!("parse Verification envelope: {error}")))?;
    if string(&envelope, "payloadType")? != "application/vnd.vela.verification-record.v2+json"
        || array(&envelope, "signatures")?.len() != 1
    {
        return Err(PortError::Unsupported(
            "Verification envelope is not signed v2 DSSE".into(),
        ));
    }
    let payload = base64::engine::general_purpose::STANDARD
        .decode(string(&envelope, "payload")?)
        .map_err(|error| PortError::Parse(format!("decode Verification payload: {error}")))?;
    let record: Value = serde_json::from_slice(&payload)
        .map_err(|error| PortError::Parse(format!("parse Verification payload: {error}")))?;
    if string(&record, "schema")? != "vela.verification-record.v2" {
        return Err(PortError::Unsupported(
            "Verification payload schema is unsupported".into(),
        ));
    }
    let identity = object(&record, "identity")?;
    let subject = object(&record, "subject")?;
    let method = object(&record, "method")?;
    let scope = object(&record, "scope")?;
    let independence = object(&record, "independence")?;
    let outcome = imported_outcome(&record)?;
    let proposal_id = string(subject, "proposal_id")?.to_owned();
    let proposal_root = string(subject, "proposal_root")?.to_owned();
    let submission_root = string(subject, "submission_root")?.to_owned();
    let claim_id = string(subject, "claim_id")?.to_owned();
    let inbox = decision_inbox(repository, binary)?;
    if let Some(refusal) = inbox.refusal {
        return Err(PortError::Process(format!(
            "Decision Inbox refused with {:?}: {}",
            refusal.code, refusal.message
        )));
    }
    current_import_subject(
        &inbox,
        &proposal_id,
        &proposal_root,
        &submission_root,
        &claim_id,
    )?;
    let root = digest(&bytes);
    let id = format!("vvr_{}", &root[7..23]);
    let actor = string(identity, "actor_id")?.to_owned();
    let argv = vec![
        "verification".into(),
        "import".into(),
        "--repo".into(),
        repository.display().to_string(),
        path.display().to_string(),
        "--as".into(),
        actor.clone(),
        "--json".into(),
    ];
    Ok(VerificationImportPreviewDto {
        envelope_path: path.display().to_string(), envelope_sha256: root.clone(), envelope_size: bytes.len() as u64, envelope_base64: base64::engine::general_purpose::STANDARD.encode(&bytes), verification_record_id: id, verification_record_root: root, verifier: actor,
        proposal_id, proposal_root, submission_id: string(subject, "submission_id")?.into(), submission_root, claim_id,
        method_profile: string(method, "profile")?.into(), method_path: string(method, "implementation")?.into(), environment_root: string(method, "environment_root")?.into(), property: string(scope, "property")?.into(), outcome, declared_independent_of: strings(independence, "declared_independent_of")?, shared_dependencies: strings(independence, "shared_dependencies")?, output_artifact_ids: strings(&record, "output_artifact_ids")?, does_not_establish: strings(scope, "does_not_establish")?,
        repository_path: repository.display().to_string(), current_repository_root: inbox.repository_root, source_commit: git.head_commit.clone(), source_tree: git.head_tree.clone(), vela_binary_sha256: exact_binary(binary)?.sha256, argv, authority_effect: "none".into(), warning: "The envelope subject matches the exact current Proposal, Submission, Claim, and Repository roots. The signed Vela CLI verifies the signature again during import. Import remains non-authoritative and changes no Standing.".into(),
    })
}

pub(crate) fn import_verification(
    repository: &Path,
    binary: &Path,
    preview: &VerificationImportPreviewDto,
) -> Result<VerificationResultDto, PortError> {
    let outcome = run_cli(
        binary,
        repository,
        &preview.argv,
        "verification.import",
        "vela.verification-import-result.v1",
        false,
    )?;
    verification_result(outcome, &preview.proposal_id)
}

fn validate_decision_request(request: &DecisionRequestDto) -> Result<String, PortError> {
    for (value, label) in [
        (&request.proposal_id, "Proposal"),
        (&request.entry_root, "entry root"),
        (&request.reason, "Decision reason"),
        (&request.performer, "Decision performer"),
    ] {
        bounded_token(value, label)?;
    }
    let kind = if request.performer.starts_with("human:") {
        "human"
    } else if request.performer.starts_with("agent:") {
        "agent"
    } else {
        return Err(PortError::InvalidInput(
            "Decision performer must be human:<id> or agent:<id>".into(),
        ));
    };
    if !request.entry_root.starts_with("sha256:") || request.entry_root.len() != 71 {
        return Err(PortError::InvalidInput(
            "Decision entry root must be one full sha256 root".into(),
        ));
    }
    if let Some(reference) = &request.session_ref {
        bounded_token(reference, "session reference")?;
    }
    Ok(kind.into())
}

pub(crate) fn preview_decision(
    repository: &Path,
    binary: &Path,
    git: &GitSnapshotDto,
    request: DecisionRequestDto,
) -> Result<DecisionPreviewDto, PortError> {
    let performer_kind = validate_decision_request(&request)?;
    let inbox = decision_inbox(repository, binary)?;
    if let Some(refusal) = inbox.refusal {
        return Err(PortError::Process(format!(
            "Decision Inbox refused with {:?}: {}",
            refusal.code, refusal.message
        )));
    }
    let mut current = inbox
        .entries
        .into_iter()
        .find(|item| item.proposal_id == request.proposal_id)
        .ok_or_else(|| {
            PortError::InvalidInput("Proposal is not current in the Decision Inbox".into())
        })?;
    if current.entry_root != request.entry_root {
        return Err(PortError::InvalidInput(
            "Decision entry root is stale; refresh the Inbox".into(),
        ));
    }
    let show = match review_show(repository, binary, &request.proposal_id)? {
        CliOutcome::Success(value) => value,
        CliOutcome::Refusal(refusal) => {
            return Err(PortError::Process(format!(
                "review.show refused with {:?}: {}",
                refusal.code, refusal.message
            )));
        }
    };
    if string(&show, "repository_root")? != current.repository_root
        || string(&show, "proposal_root")? != current.proposal_root
    {
        return Err(PortError::Parse(
            "review.show disagrees with Decision Inbox roots".into(),
        ));
    }
    current.verifications = detailed_verifications(repository, &show, &current)?;
    let action = match request.action {
        DecisionActionDto::Accept => "accept",
        DecisionActionDto::Reject => "reject",
    };
    let mut argv = vec![
        "review".into(),
        action.into(),
        "--repo".into(),
        repository.display().to_string(),
        request.proposal_id.clone(),
        "--reason".into(),
        request.reason.clone(),
        "--if-entry-root".into(),
        request.entry_root.clone(),
        "--as".into(),
        request.performer.clone(),
    ];
    if let Some(reference) = &request.session_ref {
        argv.extend(["--session-ref".into(), reference.clone()]);
    }
    argv.push("--json".into());
    let expected_successor = match request.action {
        DecisionActionDto::Accept => current.standing_delta.if_accept.clone(),
        DecisionActionDto::Reject => current.standing_delta.if_reject.clone(),
    };
    Ok(DecisionPreviewDto { request, repository_path: repository.display().to_string(), source_commit: git.head_commit.clone(), source_tree: git.head_tree.clone(), vela_binary_sha256: exact_binary(binary)?.sha256, entry: current, performer_kind,
        repository_authority_principal: "Resolved and authenticated by signed Vela at execution; v0.977.3 exposes the actual principal only in the Decision result/readback.".into(),
        authentication: "local_os_session".into(), transaction_signer: "repository_authority".into(), ssh_agent_forwarded: std::env::var_os("SSH_AUTH_SOCK").is_some(), argv, expected_successor,
        warning: "This is the only authority-changing step. Verification outcome remains separate. Vela authenticates the Repository principal, evaluates policy, signs the transaction, and rejects any stale entry root.".into() })
}

fn replay_readback(
    repository: &Path,
    binary: &Path,
    proposal_id: &str,
) -> Result<DecisionReadbackDto, PortError> {
    let show = match review_show(repository, binary, proposal_id)? {
        CliOutcome::Success(value) => value,
        CliOutcome::Refusal(refusal) => {
            return Err(PortError::Process(format!(
                "post-Decision review.show refused with {:?}: {}",
                refusal.code, refusal.message
            )));
        }
    };
    let args = vec![
        "replay".into(),
        "--repo".into(),
        repository.display().to_string(),
        "--json".into(),
    ];
    let replay = match run_cli(
        binary,
        repository,
        &args,
        "replay",
        "vela.repository-verification.v3",
        false,
    )? {
        CliOutcome::Success(value) => value,
        CliOutcome::Refusal(refusal) => {
            return Err(PortError::Process(format!(
                "post-Decision replay refused with {:?}: {}",
                refusal.code, refusal.message
            )));
        }
    };
    let counts = object(&replay, "counts")?;
    let decision = show.get("decision").filter(|value| value.is_object());
    Ok(DecisionReadbackDto {
        status: string(&show, "status")?.into(),
        standing: decision.and_then(|value| optional_string(value, "standing").ok().flatten()),
        decision_actor: decision.and_then(|value| optional_string(value, "actor").ok().flatten()),
        decision_actor_class: decision
            .and_then(|value| optional_string(value, "actor_class").ok().flatten()),
        authority_principal_id: decision.and_then(|value| {
            optional_string(value, "authority_principal_id")
                .ok()
                .flatten()
        }),
        decision_event_id: decision
            .and_then(|value| optional_string(value, "event_id").ok().flatten()),
        applied_event_id: decision
            .and_then(|value| optional_string(value, "applied_event_id").ok().flatten()),
        event_root: decision.and_then(|value| optional_string(value, "event_root").ok().flatten()),
        repository_root: string(&replay, "repository_root")?.into(),
        replay_accepted_claims: counts
            .get("accepted_claims")
            .and_then(Value::as_u64)
            .ok_or_else(|| PortError::Parse("replay omitted accepted_claims".into()))?,
        replay_pending_claims: counts
            .get("pending_claims")
            .and_then(Value::as_u64)
            .ok_or_else(|| PortError::Parse("replay omitted pending_claims".into()))?,
    })
}

pub(crate) fn execute_decision(
    repository: &Path,
    binary: &Path,
    preview: &DecisionPreviewDto,
) -> Result<DecisionExecutionDto, PortError> {
    let action_name = match preview.request.action {
        DecisionActionDto::Accept => "accept",
        DecisionActionDto::Reject => "reject",
    };
    let expected_command = format!("review.{action_name}");
    let outcome = run_cli(
        binary,
        repository,
        &preview.argv,
        &expected_command,
        "vela.review-decision.v5",
        true,
    );
    let readback = replay_readback(repository, binary, &preview.request.proposal_id)?;
    let decision_committed = matches!(readback.status.as_str(), "accepted" | "rejected");
    let successor_matches_preview =
        readback.repository_root == preview.expected_successor.repository_root;
    let outcome = match outcome {
        Ok(value) => value,
        Err(error) if decision_committed => CliOutcome::Refusal(StructuredVelaRefusalDto {
            kind: "post_commit_receipt_unavailable".into(),
            code: None,
            message: format!(
                "Decision readback is committed but the command receipt was unavailable: {error}"
            ),
            hint: Some(
                "Do not retry. Inspect the actual Decision, Event, and Standing readback.".into(),
            ),
            command: expected_command.clone(),
            operation_id: None,
            changed: Some(true),
            next: Some("Use the committed readback; exact recovery is available only for a structured repository_incomplete operation.".into()),
        }),
        Err(error) => return Err(error),
    };
    match outcome {
        CliOutcome::Refusal(refusal) => Ok(DecisionExecutionDto {
            command_succeeded: false,
            decision_committed,
            successor_matches_preview,
            events_match_receipt: None,
            action: preview.request.action,
            proposal_id: preview.request.proposal_id.clone(),
            entry_root: preview.request.entry_root.clone(),
            decision_plan_root: None,
            event_ids: Vec::new(),
            authority_record_id: None,
            actual_performer: readback.decision_actor.clone(),
            actual_performer_kind: readback.decision_actor_class.clone(),
            actual_authority_principal: readback.authority_principal_id.clone(),
            authentication: None,
            transaction_signer: None,
            scientific_state_changed: None,
            refusal: Some(refusal),
            readback,
        }),
        CliOutcome::Success(value) => {
            if string(&value, "proposal_id")? != preview.request.proposal_id
                || string(&value, "actor_id")? != preview.request.performer
                || string(&value, "action")? != action_name
            {
                return Err(PortError::Parse(
                    "Decision result disagrees with reviewed request".into(),
                ));
            }
            let event_ids = strings(&value, "event_ids")?;
            let events_match_receipt = readback
                .decision_event_id
                .as_ref()
                .is_some_and(|event| event_ids.contains(event))
                && readback
                    .applied_event_id
                    .as_ref()
                    .is_none_or(|event| event_ids.contains(event));
            Ok(DecisionExecutionDto {
                command_succeeded: true,
                decision_committed,
                successor_matches_preview,
                events_match_receipt: Some(events_match_receipt),
                action: preview.request.action,
                proposal_id: preview.request.proposal_id.clone(),
                entry_root: preview.request.entry_root.clone(),
                decision_plan_root: Some(string(&value, "decision_plan_root")?.into()),
                event_ids,
                authority_record_id: optional_string(&value, "authority_record_id")?,
                actual_performer: optional_string(&value, "actor_id")?,
                actual_performer_kind: optional_string(&value, "actor_class")?,
                actual_authority_principal: optional_string(&value, "authority_principal_id")?,
                authentication: optional_string(&value, "authentication")?,
                transaction_signer: optional_string(&value, "transaction_signer")?,
                scientific_state_changed: value
                    .get("scientific_state_changed")
                    .and_then(Value::as_bool),
                refusal: None,
                readback,
            })
        }
    }
}

pub(crate) fn preview_recovery(
    repository: &Path,
    binary: &Path,
    git: &GitSnapshotDto,
    operation_id: &str,
) -> Result<RecoveryPreviewDto, PortError> {
    validate_operation_id(operation_id)?;
    Ok(RecoveryPreviewDto { repository_path: repository.display().to_string(), operation_id: operation_id.into(), source_commit: git.head_commit.clone(), source_tree: git.head_tree.clone(), vela_binary_sha256: exact_binary(binary)?.sha256, argv: vec!["recover".into(), "--repo".into(), repository.display().to_string(), operation_id.into(), "--json".into()], warning: "Recovery applies only the signed Vela transaction journal for this exact operation. It never retries or chooses a Decision.".into() })
}

pub(crate) fn recover_transaction(
    repository: &Path,
    binary: &Path,
    preview: &RecoveryPreviewDto,
) -> Result<RecoveryResultDto, PortError> {
    match run_cli(
        binary,
        repository,
        &preview.argv,
        "recover",
        "vela.recover-result.v1",
        false,
    )? {
        CliOutcome::Refusal(refusal) => Ok(RecoveryResultDto {
            operation_id: preview.operation_id.clone(),
            outcome: None,
            repository_blocked_after: None,
            continuation_status: None,
            next_command: None,
            refusal: Some(refusal),
        }),
        CliOutcome::Success(value) => {
            if string(&value, "operation_id")? != preview.operation_id {
                return Err(PortError::Parse(
                    "recovery result operation id changed".into(),
                ));
            }
            Ok(RecoveryResultDto {
                operation_id: preview.operation_id.clone(),
                outcome: optional_string(&value, "outcome")?,
                repository_blocked_after: value
                    .get("repository_blocked_after")
                    .and_then(Value::as_bool),
                continuation_status: optional_string(&value, "continuation_status")?,
                next_command: optional_string(&value, "next_command")?,
                refusal: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn recovery_operation_id_is_one_exact_lowercase_digest() {
        assert!(validate_operation_id(&format!("vop_{}", "a".repeat(64))).is_ok());
        for invalid in [
            "vop_short".to_string(),
            format!("vop_{}", "A".repeat(64)),
            format!("op_{}", "a".repeat(64)),
            format!("vop_{}", "g".repeat(64)),
        ] {
            assert!(
                validate_operation_id(&invalid).is_err(),
                "accepted {invalid}"
            );
        }
    }

    fn git(root: &Path, args: &[&str]) {
        let status = Command::new("/usr/bin/git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("run fixture Git");
        assert!(status.success(), "fixture Git failed: {args:?}");
    }

    fn method_bytes(provider: &str) -> Vec<u8> {
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "vela.review-method.v1",
            "profile": "exact-review",
            "property": "formal-correctness",
            "reviewer": {
                "kind": "agent",
                "display_name": "Fixture reviewer",
                "identifier": "fixture-reviewer",
                "provider": provider,
                "version": "model-1"
            },
            "attested_by_actor_id": "agent:fixture",
            "procedure": ["Check the exact retained evidence."],
            "required_output": ["A bounded retained report."],
            "does_not_establish": ["Repository acceptance."]
        }))
        .expect("serialize Method")
    }

    #[test]
    fn decision_actor_and_root_are_closed() {
        let mut request = DecisionRequestDto {
            proposal_id: "vpr_1234567890abcdef".into(),
            entry_root: format!("sha256:{}", "a".repeat(64)),
            action: DecisionActionDto::Accept,
            reason: "Bounded reason.".into(),
            performer: "agent:reviewer".into(),
            session_ref: None,
        };
        assert_eq!(validate_decision_request(&request).unwrap(), "agent");
        request.performer = "verifier:not-a-decision-performer".into();
        assert!(validate_decision_request(&request).is_err());
        request.performer = "human:reviewer".into();
        request.entry_root = "sha256:short".into();
        assert!(validate_decision_request(&request).is_err());
    }

    #[test]
    fn stable_errors_never_branch_on_message() {
        let source = include_str!("tranche_three.rs");
        assert!(!source.contains(&["message", ".starts_with"].concat()));
        assert!(!source.contains(&["message", ".contains"].concat()));
    }

    #[test]
    fn verification_import_refuses_untrusted_dialog_outcomes() {
        for outcome in ["pass\nDigest: fake", "unknown", ""] {
            let record = serde_json::json!({ "outcome": outcome });
            assert!(imported_outcome(&record).is_err());
        }
        assert_eq!(
            imported_outcome(&serde_json::json!({ "outcome": "inconclusive" })).unwrap(),
            "inconclusive"
        );
    }

    #[test]
    fn verification_import_subject_must_match_current_inbox_roots() {
        let value: Value = serde_json::from_str(include_str!(
            "../../../fixtures/core/v0.977.3/decision-inbox-v3.json"
        ))
        .expect("frozen Inbox JSON");
        let inbox = parse_decision_inbox_value(&value).expect("parse frozen Inbox");
        let entry = &inbox.entries[0];
        assert!(
            current_import_subject(
                &inbox,
                &entry.proposal_id,
                &entry.proposal_root,
                &entry.submission_root,
                &entry.claim_id,
            )
            .is_ok()
        );
        assert!(
            current_import_subject(
                &inbox,
                &entry.proposal_id,
                "sha256:stale",
                &entry.submission_root,
                &entry.claim_id,
            )
            .is_err()
        );
    }

    #[test]
    fn decision_facets_do_not_enrich_from_dirty_or_digest_mismatched_method() {
        let temp = tempfile::tempdir().expect("tempdir");
        git(temp.path(), &["init", "-q"]);
        git(temp.path(), &["config", "user.name", "Workbench test"]);
        git(
            temp.path(),
            &["config", "user.email", "workbench@example.invalid"],
        );
        let method_path = temp.path().join("method.json");
        std::fs::write(&method_path, method_bytes("provider-a")).expect("write Method");
        git(temp.path(), &["add", "method.json"]);
        git(temp.path(), &["commit", "-q", "-m", "fixture Method"]);

        let signed_root = digest(&method_bytes("provider-a"));
        let verified =
            verified_current_method(temp.path(), "method.json", &signed_root, "agent:fixture")
                .expect("tracked current Method should enrich");
        assert_eq!(verified.provider.as_deref(), Some("provider-a"));

        assert!(
            verified_current_method(temp.path(), "method.json", &signed_root, "agent:other")
                .is_none()
        );

        std::fs::write(&method_path, method_bytes("provider-b")).expect("dirty Method");
        assert!(
            verified_current_method(temp.path(), "method.json", &signed_root, "agent:fixture")
                .is_none()
        );

        git(temp.path(), &["add", "method.json"]);
        git(temp.path(), &["commit", "-q", "-m", "different Method"]);
        assert!(
            verified_current_method(temp.path(), "method.json", &signed_root, "agent:fixture")
                .is_none()
        );
    }

    #[test]
    fn frozen_decision_inbox_preserves_dependency_disclosure_without_inference() {
        let value: Value = serde_json::from_str(include_str!(
            "../../../fixtures/core/v0.977.3/decision-inbox-v3.json"
        ))
        .expect("frozen Inbox JSON");
        let parsed = parse_decision_inbox_value(&value).expect("parse frozen Inbox");
        assert_eq!(parsed.entries.len(), 1);
        let verification = &parsed.entries[0].verifications[0];
        assert_eq!(verification.verifier, "service:fixture-verifier");
        assert!(verification.declared_independent_of.is_empty());
        assert!(verification.shared_dependencies.is_empty());
        assert_eq!(
            verification.protocol_evidence_role.as_deref(),
            Some("requirement_satisfying")
        );
    }

    #[test]
    fn live_disposable_reject_round_trip_when_explicitly_requested() {
        let (Ok(repository), Ok(binary)) = (
            std::env::var("VELA_WORKBENCH_T3_MUTATION_REPO"),
            std::env::var("VELA_WORKBENCH_SMOKE_BINARY"),
        ) else {
            return;
        };
        let repository = Path::new(&repository).canonicalize().expect("fixture repo");
        assert!(
            repository
                .join(".vela-workbench-disposable-authority-fixture")
                .is_file(),
            "authority mutation test refuses any repository without its exact disposable marker"
        );
        let binary = Path::new(&binary).canonicalize().expect("Vela binary");
        let inbox = decision_inbox(&repository, &binary).expect("current Inbox");
        assert_eq!(inbox.entries.len(), 1, "fixture must contain one Proposal");
        let entry = &inbox.entries[0];
        let git = super::super::git::inspect(&repository).expect("fixture Git identity");
        let accept_preview = preview_decision(
            &repository,
            &binary,
            &git,
            DecisionRequestDto {
                proposal_id: entry.proposal_id.clone(),
                entry_root: entry.entry_root.clone(),
                action: DecisionActionDto::Accept,
                reason: "Exercise the independent-review refusal on a disposable fixture.".into(),
                performer: "agent:workbench-tranche3-qa".into(),
                session_ref: None,
            },
        )
        .expect("acceptance preview");
        let refused = execute_decision(&repository, &binary, &accept_preview)
            .expect("structured independent-review refusal");
        assert!(!refused.decision_committed);
        assert_eq!(refused.readback.status, "pending_review");
        assert_eq!(
            refused
                .refusal
                .as_ref()
                .and_then(|item| item.code.as_deref()),
            Some("missing_independent_verification")
        );
        assert_eq!(refused.readback.repository_root, entry.repository_root);

        let mut stale_request = accept_preview.request.clone();
        stale_request.action = DecisionActionDto::Reject;
        stale_request.entry_root = format!("sha256:{}", "0".repeat(64));
        assert!(preview_decision(&repository, &binary, &git, stale_request).is_err());
        assert_eq!(
            decision_inbox(&repository, &binary)
                .expect("Inbox after stale refusal")
                .repository_root,
            entry.repository_root
        );

        let preview = preview_decision(
            &repository,
            &binary,
            &git,
            DecisionRequestDto {
                proposal_id: entry.proposal_id.clone(),
                entry_root: entry.entry_root.clone(),
                action: DecisionActionDto::Reject,
                reason: "Reject only this disposable Workbench authority fixture.".into(),
                performer: "agent:workbench-tranche3-qa".into(),
                session_ref: None,
            },
        )
        .expect("Decision preview");
        let result = execute_decision(&repository, &binary, &preview).expect("Decision readback");
        assert!(result.decision_committed);
        assert_eq!(result.readback.status, "rejected");
        assert_eq!(result.readback.standing.as_deref(), Some("rejected"));
        assert!(result.successor_matches_preview);
        assert_eq!(
            decision_inbox(&repository, &binary)
                .expect("final Inbox")
                .entries
                .len(),
            0
        );
    }
}
