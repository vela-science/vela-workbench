use std::collections::{BTreeMap, BTreeSet};

use url::Url;

use crate::contracts::{GitRemoteDto, ProblemHandoffDto};

use super::{PortError, launch};

const MAX_HANDOFF_BYTES: usize = 16 * 1024;
const MAX_ARTIFACTS: usize = 32;
const MAX_ARTIFACT_PATH_BYTES: usize = 1024;

fn required(values: &mut BTreeMap<String, String>, key: &str) -> Result<String, PortError> {
    values
        .remove(key)
        .ok_or_else(|| PortError::InvalidInput(format!("Problem handoff omitted {key}")))
}

fn exact_https(raw: &str, label: &str) -> Result<Url, PortError> {
    let url = Url::parse(raw)
        .map_err(|error| PortError::InvalidInput(format!("parse {label}: {error}")))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(PortError::InvalidInput(format!(
            "{label} must be an exact HTTPS URL without credentials, query, or fragment"
        )));
    }
    Ok(url)
}

fn problem_url(raw: &str) -> Result<String, PortError> {
    let url = exact_https(raw, "Problem URL")?;
    let components = url.path().trim_matches('/').split('/').collect::<Vec<_>>();
    if url.host_str() != Some("problems.science")
        || url.path().ends_with('/')
        || components.len() != 3
        || components[0] != "problems"
        || components[1..]
            .iter()
            .any(|component| component.is_empty() || matches!(*component, "." | ".."))
    {
        return Err(PortError::InvalidInput(
            "Problem URL must be one canonical problems.science /problems/ URL".into(),
        ));
    }
    Ok(url.to_string())
}

fn repository_url(raw: &str, label: &str) -> Result<String, PortError> {
    let mut url = exact_https(raw, label)?;
    let path = url
        .path()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string();
    if path.is_empty() || path == "/" || path.split('/').filter(|item| !item.is_empty()).count() < 2
    {
        return Err(PortError::InvalidInput(format!(
            "{label} must locate one repository"
        )));
    }
    url.set_path(&path);
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn source_revision(raw: &str) -> Result<String, PortError> {
    if !matches!(raw.len(), 40 | 64)
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PortError::InvalidInput(
            "source ref must be one lowercase full Git object id".into(),
        ));
    }
    Ok(raw.into())
}

fn artifact_path(raw: &str) -> Result<String, PortError> {
    if raw.is_empty()
        || raw.len() > MAX_ARTIFACT_PATH_BYTES
        || raw.starts_with('/')
        || raw.ends_with('/')
        || raw.contains('\\')
        || raw.chars().any(char::is_control)
        || raw
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(PortError::InvalidInput(
            "artifact must be one bounded source-relative path".into(),
        ));
    }
    Ok(raw.into())
}

pub(crate) fn parse(raw: &str) -> Result<ProblemHandoffDto, PortError> {
    if raw.len() > MAX_HANDOFF_BYTES {
        return Err(PortError::InvalidInput(
            "Problem handoff exceeds the 16 KiB browser-safe limit".into(),
        ));
    }
    let url = Url::parse(raw)
        .map_err(|error| PortError::InvalidInput(format!("parse Problem handoff: {error}")))?;
    if url.scheme() != "vela-workbench"
        || url.host_str() != Some("continue")
        || !matches!(url.path(), "" | "/")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(PortError::InvalidInput(
            "Problem handoff must use vela-workbench://continue with no credentials, path, or fragment"
                .into(),
        ));
    }

    let mut values = BTreeMap::new();
    let mut artifacts = Vec::new();
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "artifact" => artifacts.push(artifact_path(&value)?),
            "v" | "problem" | "source" | "ref" | "repository" => {
                let key = key.into_owned();
                if values.insert(key.clone(), value.into_owned()).is_some() {
                    return Err(PortError::InvalidInput(format!(
                        "Problem handoff repeated {key}"
                    )));
                }
            }
            _ => {
                return Err(PortError::InvalidInput(format!(
                    "Problem handoff contains unsupported field {key}"
                )));
            }
        }
    }
    if required(&mut values, "v")? != "1" {
        return Err(PortError::Unsupported(
            "Problem handoff version is not supported".into(),
        ));
    }
    if artifacts.len() > MAX_ARTIFACTS {
        return Err(PortError::InvalidInput(format!(
            "Problem handoff exceeds {MAX_ARTIFACTS} artifact references"
        )));
    }
    let mut unique = BTreeSet::new();
    if artifacts.iter().any(|path| !unique.insert(path.clone())) {
        return Err(PortError::InvalidInput(
            "Problem handoff repeats an artifact reference".into(),
        ));
    }
    let handoff = ProblemHandoffDto {
        schema: "vela.workbench.problem-handoff.v1".into(),
        handoff_url: raw.into(),
        problem_url: problem_url(&required(&mut values, "problem")?)?,
        source_repository_url: repository_url(
            &required(&mut values, "source")?,
            "source repository URL",
        )?,
        source_revision: source_revision(&required(&mut values, "ref")?)?,
        authority_repository_url: repository_url(
            &required(&mut values, "repository")?,
            "authority Repository URL",
        )?,
        artifact_paths: artifacts,
        authority_effect: "none".into(),
        boundary: "This handoff carries browser-safe locators and explicit source-relative artifact references only. It does not select a local checkout, transfer files or credentials, execute a tool, authenticate a principal, or imply a Repository Decision or Standing.".into(),
    };
    if !values.is_empty() {
        return Err(PortError::InvalidInput(
            "Problem handoff contains unconsumed fields".into(),
        ));
    }
    Ok(handoff)
}

pub(crate) fn source_matches(
    remotes: &[GitRemoteDto],
    head_commit: &str,
    handoff: &ProblemHandoffDto,
) -> (bool, bool) {
    let remote_matches = remotes.iter().any(|remote| {
        remote.operation == "fetch"
            && matches!(
                launch::https_remote(&remote.url),
                Ok(url) if url == handoff.source_repository_url
            )
    });
    (remote_matches, head_commit == handoff.source_revision)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REF: &str = "0123456789012345678901234567890123456789";

    fn valid() -> String {
        format!(
            "vela-workbench://continue?v=1&problem=https%3A%2F%2Fproblems.science%2Fproblems%2Ferdos-problems%2F94&source=https%3A%2F%2Fgithub.com%2Fvela-science%2Flean-proofs.git&ref={REF}&repository=https%3A%2F%2Fgithub.com%2Fvela-science%2Fmath.git&artifact=Erdos%2FProblem94.lean&artifact=evidence%2Fcheck.json"
        )
    }

    #[test]
    fn exact_browser_safe_handoff_preserves_separate_axes() {
        let parsed = parse(&valid()).expect("valid handoff");
        assert_eq!(
            parsed.problem_url,
            "https://problems.science/problems/erdos-problems/94"
        );
        assert_eq!(
            parsed.source_repository_url,
            "https://github.com/vela-science/lean-proofs"
        );
        assert_eq!(
            parsed.authority_repository_url,
            "https://github.com/vela-science/math"
        );
        assert_eq!(parsed.source_revision, REF);
        assert_eq!(
            parsed.artifact_paths,
            ["Erdos/Problem94.lean", "evidence/check.json"]
        );
        assert_eq!(parsed.authority_effect, "none");
    }

    #[test]
    fn untrusted_fields_credentials_and_paths_fail_closed() {
        let cases = [
            valid().replace("&ref=", "&upload=https%3A%2F%2Fevil.invalid&ref="),
            valid().replace(
                "https%3A%2F%2Fgithub.com",
                "https%3A%2F%2Ftoken%40github.com",
            ),
            valid().replace("Erdos%2FProblem94.lean", "..%2Fsecret"),
            valid().replace("&ref=", "&ref=main&ignored="),
            valid().replace("problems.science", "example.invalid"),
            valid().replace("erdos-problems", ""),
            valid().replace("erdos-problems", "."),
        ];
        for candidate in cases {
            assert!(parse(&candidate).is_err(), "accepted {candidate}");
        }
    }

    #[test]
    fn duplicates_and_unsupported_versions_fail_closed() {
        assert!(parse(&valid().replace("?v=1", "?v=1&v=1")).is_err());
        assert!(parse(&valid().replace("?v=1", "?v=2")).is_err());
        assert!(parse(&format!("{}&artifact=Erdos%2FProblem94.lean", valid())).is_err());
    }

    #[test]
    fn local_source_requires_both_fetch_remote_and_exact_head() {
        let handoff = parse(&valid()).expect("handoff");
        let remotes = vec![GitRemoteDto {
            name: "origin".into(),
            url: "git@github.com:vela-science/lean-proofs.git".into(),
            operation: "fetch".into(),
        }];
        assert_eq!(
            source_matches(&remotes, &handoff.source_revision, &handoff),
            (true, true)
        );
        assert_eq!(
            source_matches(&remotes, &"f".repeat(40), &handoff),
            (true, false)
        );
        let wrong = vec![GitRemoteDto {
            url: "https://github.com/vela-science/math".into(),
            ..remotes[0].clone()
        }];
        assert_eq!(
            source_matches(&wrong, &handoff.source_revision, &handoff),
            (false, true)
        );
    }
}
