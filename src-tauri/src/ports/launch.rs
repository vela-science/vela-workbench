use std::{path::Path, time::Duration};

use url::Url;

use crate::contracts::{GitRemoteDto, LaunchKindDto, LaunchResultDto};

use super::{PortError, ProcessSpec, git, run_bounded};

fn https_remote(raw: &str) -> Result<String, PortError> {
    let candidate = if let Some(rest) = raw.strip_prefix("git@") {
        let (host, path) = rest.split_once(':').ok_or_else(|| {
            PortError::Unsupported("SSH remote does not use the host:path form".into())
        })?;
        format!("https://{host}/{path}")
    } else if raw.starts_with("ssh://") {
        let url = Url::parse(raw)
            .map_err(|error| PortError::Unsupported(format!("parse SSH remote: {error}")))?;
        let host = url
            .host_str()
            .ok_or_else(|| PortError::Unsupported("SSH remote has no host".into()))?;
        format!("https://{host}{}", url.path())
    } else {
        raw.to_string()
    };
    let mut url = Url::parse(&candidate)
        .map_err(|error| PortError::Unsupported(format!("parse forge remote: {error}")))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(PortError::Unsupported(
            "forge handoff requires an HTTPS remote without embedded credentials".into(),
        ));
    }
    url.set_query(None);
    url.set_fragment(None);
    let trimmed = url
        .path()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string();
    url.set_path(&trimmed);
    Ok(url.to_string().trim_end_matches('/').to_string())
}

fn fetch_remote(remotes: &[GitRemoteDto]) -> Result<String, PortError> {
    let remote = remotes
        .iter()
        .find(|remote| remote.name == "origin" && remote.operation == "fetch")
        .or_else(|| remotes.iter().find(|remote| remote.operation == "fetch"))
        .ok_or_else(|| PortError::Unavailable("repository has no fetch remote".into()))?;
    https_remote(&remote.url)
}

#[cfg(target_os = "macos")]
fn launch_open(cwd: &Path, args: &[&str], label: &str) -> Result<(), PortError> {
    let mut spec = ProcessSpec::new("/usr/bin/open", cwd).args(args);
    spec.timeout = Duration::from_secs(5);
    spec.max_stdout = 16 * 1024;
    spec.max_stderr = 16 * 1024;
    let output = run_bounded(spec)?;
    if !output.success {
        return Err(PortError::Process(format!(
            "open {label} failed with {:?}",
            output.exit_code
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_local(cwd: &Path, app: &str, target: &str) -> Result<(), PortError> {
    launch_open(cwd, &["-a", app, target], app)
}

#[cfg(target_os = "macos")]
fn launch_https(cwd: &Path, target: &str) -> Result<(), PortError> {
    launch_open(cwd, &[target], "HTTPS destination")
}

#[cfg(not(target_os = "macos"))]
fn launch_local(_cwd: &Path, _app: &str, _target: &str) -> Result<(), PortError> {
    Err(PortError::Unsupported(
        "this Workbench build has implemented exact local-app handoff on macOS only".into(),
    ))
}

#[cfg(not(target_os = "macos"))]
fn launch_https(_cwd: &Path, _target: &str) -> Result<(), PortError> {
    Err(PortError::Unsupported(
        "this Workbench build has implemented exact HTTPS handoff on macOS only".into(),
    ))
}

pub(crate) fn launch(repository: &Path, kind: LaunchKindDto) -> Result<LaunchResultDto, PortError> {
    let snapshot = git::inspect(repository)?;
    let root = snapshot.root.clone();
    let root_path = Path::new(&root);
    match kind {
        LaunchKindDto::Terminal => {
            launch_local(root_path, "Terminal", &root)?;
            Ok(LaunchResultDto {
                target: root,
                owner: "Terminal".into(),
            })
        }
        LaunchKindDto::Cursor => {
            launch_local(root_path, "Cursor", &root)?;
            Ok(LaunchResultDto {
                target: root,
                owner: "Cursor".into(),
            })
        }
        LaunchKindDto::VisualStudioCode => {
            launch_local(root_path, "Visual Studio Code", &root)?;
            Ok(LaunchResultDto {
                target: root,
                owner: "Visual Studio Code".into(),
            })
        }
        LaunchKindDto::Forge => {
            let destination = fetch_remote(&snapshot.remotes)?;
            launch_https(root_path, &destination)?;
            Ok(LaunchResultDto {
                target: destination,
                owner: "configured Git forge".into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::https_remote;

    #[test]
    fn github_ssh_remote_becomes_exact_https_locator() {
        assert_eq!(
            https_remote("git@github.com:vela-science/vela.git").expect("remote converts"),
            "https://github.com/vela-science/vela"
        );
    }

    #[test]
    fn embedded_credentials_are_refused() {
        assert!(https_remote("https://token@example.com/owner/repo.git").is_err());
    }

    #[test]
    fn query_and_fragment_do_not_cross_the_handoff() {
        assert_eq!(
            https_remote("https://example.com/owner/repo.git?token=no#private")
                .expect("remote normalizes"),
            "https://example.com/owner/repo"
        );
    }
}
