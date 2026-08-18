use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

use crate::contracts::{
    EvidenceExportPreviewDto, EvidenceExportRequestDto, EvidenceExportResultDto, EvidenceItemDto,
    EvidenceSourceDto, GitSnapshotDto, NativeExecResultDto,
};

use super::PortError;

pub(crate) const MAX_EVIDENCE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct CapturedEvidence {
    pub dto: EvidenceItemDto,
    pub bytes: Vec<u8>,
}

fn digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn read_regular_bounded(path: &Path) -> Result<Vec<u8>, PortError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| PortError::InvalidInput(format!("inspect evidence file: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PortError::InvalidInput(
            "evidence must be one regular non-symlink file".into(),
        ));
    }
    if metadata.len() > MAX_EVIDENCE_BYTES {
        return Err(PortError::Unsupported(format!(
            "evidence file exceeds the {} byte exact-display limit",
            MAX_EVIDENCE_BYTES
        )));
    }
    let file = File::open(path)
        .map_err(|error| PortError::InvalidInput(format!("open evidence file: {error}")))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_EVIDENCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| PortError::Process(format!("read evidence file: {error}")))?;
    if bytes.len() as u64 > MAX_EVIDENCE_BYTES {
        return Err(PortError::Unsupported(
            "evidence file grew beyond the exact-display limit while reading".into(),
        ));
    }
    Ok(bytes)
}

fn media_type(path: &Path) -> (&'static str, &'static str) {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
    {
        "json" => ("application/json", "data"),
        "csv" => ("text/csv", "table"),
        "txt" | "log" => ("text/plain", "output"),
        "md" => ("text/markdown", "report"),
        "lean" => ("text/plain", "proof-source"),
        "rs" | "ts" | "tsx" | "py" => ("text/plain", "source"),
        "pdf" => ("application/pdf", "report"),
        _ => ("application/octet-stream", "artifact"),
    }
}

pub(crate) fn capture_file(
    root: &Path,
    git: &GitSnapshotDto,
    selected: &Path,
) -> Result<CapturedEvidence, PortError> {
    let canonical_root = std::fs::canonicalize(root)
        .map_err(|error| PortError::InvalidInput(format!("resolve repository: {error}")))?;
    let canonical = std::fs::canonicalize(selected)
        .map_err(|error| PortError::InvalidInput(format!("resolve evidence file: {error}")))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(PortError::InvalidInput(
            "selected evidence file is outside the selected repository".into(),
        ));
    }
    let relative = canonical
        .strip_prefix(&canonical_root)
        .map_err(|_| PortError::InvalidInput("derive evidence path".into()))?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
        || relative.components().next() == Some(Component::Normal(".git".as_ref()))
    {
        return Err(PortError::InvalidInput(
            "evidence path must be a repository-relative file outside .git".into(),
        ));
    }
    let bytes = read_regular_bounded(&canonical)?;
    let (media_type, kind_hint) = media_type(&canonical);
    let display_name = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("evidence")
        .to_string();
    Ok(CapturedEvidence {
        dto: EvidenceItemDto {
            source: EvidenceSourceDto::LocalFile {
                path: canonical.display().to_string(),
                repository_relative_path: relative.display().to_string(),
            },
            display_name,
            sha256: digest(&bytes),
            size: bytes.len() as u64,
            media_type: media_type.into(),
            kind_hint: kind_hint.into(),
            source_commit: git.head_commit.clone(),
            source_tree: git.head_tree.clone(),
            source_dirty: git.dirty,
            content_base64: STANDARD.encode(&bytes),
            content_utf8: String::from_utf8(bytes.clone()).ok(),
            private: true,
        },
        bytes,
    })
}

pub(crate) fn capture_output(
    result: &NativeExecResultDto,
    stream: &str,
) -> Result<CapturedEvidence, PortError> {
    let output = match stream {
        "stdout" => &result.stdout,
        "stderr" => &result.stderr,
        _ => {
            return Err(PortError::InvalidInput(
                "command evidence stream must be stdout or stderr".into(),
            ));
        }
    };
    let bytes = STANDARD
        .decode(&output.content_base64)
        .map_err(|_| PortError::Parse("stored command output base64 is invalid".into()))?;
    if digest(&bytes) != output.sha256 || bytes.len() as u64 != output.size {
        return Err(PortError::Parse(
            "stored command output bytes disagree with their digest or size".into(),
        ));
    }
    Ok(CapturedEvidence {
        dto: EvidenceItemDto {
            source: EvidenceSourceDto::CommandOutput {
                run_id: result.run_id.clone(),
                stream: stream.into(),
            },
            display_name: format!("{}-{stream}.txt", result.run_id),
            sha256: output.sha256.clone(),
            size: output.size,
            media_type: "text/plain".into(),
            kind_hint: "command-output".into(),
            source_commit: result.source_commit.clone(),
            source_tree: result.source_tree.clone(),
            source_dirty: true,
            content_base64: output.content_base64.clone(),
            content_utf8: output.content_utf8.clone(),
            private: true,
        },
        bytes,
    })
}

pub(crate) fn preview_export(
    captured: &CapturedEvidence,
    request: EvidenceExportRequestDto,
    destination: &Path,
) -> Result<EvidenceExportPreviewDto, PortError> {
    if request.expected_sha256 != captured.dto.sha256 {
        return Err(PortError::Unsupported(
            "evidence bytes changed after capture; capture again".into(),
        ));
    }
    if request.exclusions.len() > 32
        || request
            .exclusions
            .iter()
            .any(|value| value.trim().is_empty() || value.len() > 1024)
    {
        return Err(PortError::InvalidInput(
            "evidence exclusions must contain at most 32 non-empty bounded entries".into(),
        ));
    }
    let (bytes, derived) = if let Some(text) = &request.derived_utf8 {
        if !request.redaction_confirmed || request.exclusions.is_empty() {
            return Err(PortError::InvalidInput(
                "derived redacted output requires redaction confirmation and at least one exclusion"
                    .into(),
            ));
        }
        if text.len() as u64 > MAX_EVIDENCE_BYTES {
            return Err(PortError::Unsupported(
                "derived evidence exceeds the exact-display limit".into(),
            ));
        }
        (text.as_bytes().to_vec(), true)
    } else {
        (captured.bytes.clone(), false)
    };
    let destination = destination.to_path_buf();
    if destination.exists() {
        return Err(PortError::InvalidInput(
            "export destination already exists; choose a new path".into(),
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| PortError::InvalidInput("export destination has no parent".into()))?;
    let canonical_parent = std::fs::canonicalize(parent)
        .map_err(|error| PortError::InvalidInput(format!("resolve export parent: {error}")))?;
    if !canonical_parent.is_dir() {
        return Err(PortError::InvalidInput(
            "export destination parent is not a directory".into(),
        ));
    }
    let destination =
        canonical_parent.join(destination.file_name().ok_or_else(|| {
            PortError::InvalidInput("export destination has no file name".into())
        })?);
    if let EvidenceSourceDto::LocalFile { path, .. } = &captured.dto.source
        && destination == Path::new(path)
    {
        return Err(PortError::InvalidInput(
            "export cannot overwrite or mutate the selected source evidence".into(),
        ));
    }
    let exclusions = request.exclusions.clone();
    let redaction_confirmed = request.redaction_confirmed;
    Ok(EvidenceExportPreviewDto {
        request,
        destination: destination.display().to_string(),
        source_sha256: captured.dto.sha256.clone(),
        source_size: captured.dto.size,
        output_sha256: digest(&bytes),
        output_size: bytes.len() as u64,
        derived,
        exclusions,
        redaction_confirmed,
        output_base64: STANDARD.encode(&bytes),
        output_utf8: String::from_utf8(bytes).ok(),
        warning: if derived {
            "This creates a new derived redacted file. The selected source evidence is never edited."
                .into()
        } else {
            "This creates one exact local copy. It does not upload, sync, or mutate the source."
                .into()
        },
    })
}

pub(crate) fn export(
    captured: &CapturedEvidence,
    expected: &EvidenceExportPreviewDto,
) -> Result<EvidenceExportResultDto, PortError> {
    let rebuilt = preview_export(
        captured,
        expected.request.clone(),
        Path::new(&expected.destination),
    )?;
    if &rebuilt != expected {
        return Err(PortError::Unsupported(
            "evidence export preview is stale; preview again".into(),
        ));
    }
    let bytes = STANDARD
        .decode(&expected.output_base64)
        .map_err(|_| PortError::Parse("export preview bytes are invalid".into()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&expected.destination)
        .map_err(|error| PortError::Process(format!("create evidence export: {error}")))?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(&expected.destination);
        return Err(PortError::Process(format!(
            "write evidence export: {error}"
        )));
    }
    drop(file);
    let written = match read_regular_bounded(Path::new(&expected.destination)) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = std::fs::remove_file(&expected.destination);
            return Err(error);
        }
    };
    if digest(&written) != expected.output_sha256 || written.len() as u64 != expected.output_size {
        let _ = std::fs::remove_file(&expected.destination);
        return Err(PortError::Process(
            "written export disagrees with the reviewed bytes".into(),
        ));
    }
    let source_unchanged = match &captured.dto.source {
        EvidenceSourceDto::LocalFile { path, .. } => read_regular_bounded(Path::new(path))
            .map(|source| digest(&source) == captured.dto.sha256)
            .unwrap_or(false),
        EvidenceSourceDto::CommandOutput { .. } => true,
    };
    if !source_unchanged {
        let _ = std::fs::remove_file(&expected.destination);
        return Err(PortError::Unsupported(
            "selected source evidence changed during export; the new destination was removed"
                .into(),
        ));
    }
    Ok(EvidenceExportResultDto {
        destination: expected.destination.clone(),
        sha256: expected.output_sha256.clone(),
        size: expected.output_size,
        derived: expected.derived,
        source_unchanged,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::contracts::{EvidenceExportRequestDto, EvidenceSourceDto, GitSnapshotDto};

    use super::{capture_file, export, preview_export};

    fn git(root: &std::path::Path) -> GitSnapshotDto {
        GitSnapshotDto {
            root: root.display().to_string(),
            branch: Some("main".into()),
            detached: false,
            head_commit: "1".repeat(40),
            head_tree: "2".repeat(40),
            upstream: None,
            ahead: 0,
            behind: 0,
            dirty: true,
            conflicted: false,
            changed_paths: 1,
            worktrees: Vec::new(),
            remotes: Vec::new(),
            entire_checkpoints: Vec::new(),
        }
    }

    #[test]
    fn derived_redaction_never_mutates_selected_source() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("private.txt");
        fs::write(&source, "secret\npublic\n").expect("source");
        let captured = capture_file(temp.path(), &git(temp.path()), &source).expect("capture");
        let destination = temp.path().join("derived.txt");
        let preview = preview_export(
            &captured,
            EvidenceExportRequestDto {
                source: EvidenceSourceDto::LocalFile {
                    path: source.display().to_string(),
                    repository_relative_path: "private.txt".into(),
                },
                expected_sha256: captured.dto.sha256.clone(),
                exclusions: vec!["removed secret line".into()],
                redaction_confirmed: true,
                derived_utf8: Some("public\n".into()),
            },
            &destination,
        )
        .expect("preview");
        let result = export(&captured, &preview).expect("export");
        assert!(result.derived);
        assert!(result.source_unchanged);
        assert_eq!(
            fs::read_to_string(&source).expect("source"),
            "secret\npublic\n"
        );
        assert_eq!(
            fs::read_to_string(&destination).expect("derived"),
            "public\n"
        );
    }

    #[test]
    fn stale_source_removes_the_new_export_and_preserves_both_source_versions() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("private.txt");
        fs::write(&source, "reviewed bytes\n").expect("source");
        let captured = capture_file(temp.path(), &git(temp.path()), &source).expect("capture");
        let destination = temp.path().join("exact-copy.txt");
        let preview = preview_export(
            &captured,
            EvidenceExportRequestDto {
                source: captured.dto.source.clone(),
                expected_sha256: captured.dto.sha256.clone(),
                exclusions: Vec::new(),
                redaction_confirmed: false,
                derived_utf8: None,
            },
            &destination,
        )
        .expect("preview");
        fs::write(&source, "concurrently changed\n").expect("race source");
        let error = export(&captured, &preview).expect_err("stale source refuses");
        assert!(error.to_string().contains("destination was removed"));
        assert!(!destination.exists());
        assert_eq!(
            fs::read_to_string(&source).expect("source"),
            "concurrently changed\n"
        );
    }
}
