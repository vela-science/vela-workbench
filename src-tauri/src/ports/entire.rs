use std::{env, fs};

use crate::contracts::EntireAvailabilityDto;

use super::git;

pub(crate) fn availability(snapshot: &crate::contracts::GitSnapshotDto) -> EntireAvailabilityDto {
    let cli_available = env::var_os("PATH")
        .map(|path| {
            env::split_paths(&path).any(|directory| {
                let candidate = directory.join(if cfg!(windows) {
                    "entire.exe"
                } else {
                    "entire"
                });
                fs::metadata(candidate).is_ok_and(|metadata| metadata.is_file())
            })
        })
        .unwrap_or(false);
    let count = snapshot.entire_checkpoints.len() as u64;
    let note = match (cli_available, count) {
        (true, 0) => {
            "Entire is installed; no checkpoint trailer appears in the inspected local Git window."
        }
        (true, _) => "Entire is installed; only opaque checkpoint references are shown.",
        (false, 0) => {
            "Entire not available. Ordinary Git history remains available; no substitute checkpoint store is created."
        }
        (false, _) => {
            "Entire CLI not available. Opaque checkpoint references remain visible from Git trailers."
        }
    };
    EntireAvailabilityDto {
        cli_available,
        checkpoint_reference_count: count,
        note: note.into(),
    }
}

#[allow(dead_code)]
pub(crate) fn inspect_repository(
    path: &std::path::Path,
) -> Result<EntireAvailabilityDto, super::PortError> {
    let snapshot = git::inspect(path)?;
    Ok(availability(&snapshot))
}

#[cfg(test)]
mod tests {
    #[test]
    fn absence_is_not_replaced_by_a_session_store() {
        let note = "Entire not available. Ordinary Git history remains available; no substitute checkpoint store is created.";
        assert!(note.contains("no substitute checkpoint store"));
    }
}
