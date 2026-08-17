use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ErrorEnvelopeWire {
    pub schema: String,
    pub ok: bool,
    pub command: String,
    pub error: ErrorBodyWire,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ErrorBodyWire {
    pub kind: String,
    pub code: Option<String>,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusV4Wire {
    pub schema: String,
    pub ok: bool,
    pub command: String,
    pub repository: StatusRepositoryWire,
    pub git: StatusGitWire,
    pub integrity: StatusIntegrityWire,
    pub roots: StatusRootsWire,
    pub counts: StatusCountsWire,
    pub decision_inbox: StatusDecisionInboxWire,
    pub actions: StatusActionsWire,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusRepositoryWire {
    pub id: String,
    pub name: String,
    pub profile_root: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusGitWire {
    pub role: String,
    pub commit: Option<String>,
    pub tree: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusIntegrityWire {
    pub replay: String,
    pub strict: String,
    pub blocker_count: u64,
    pub blockers_by_code: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusRootsWire {
    pub origin: Option<String>,
    pub repository: Option<String>,
    pub authority_keyset: Option<String>,
    pub authority_policy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusCountsWire {
    pub claims: u64,
    pub accepted_claims: u64,
    pub pending_claims: u64,
    pub pending_review: u64,
    pub accepted_review: u64,
    pub rejected_review: u64,
    pub withdrawn_review: u64,
    pub submissions: u64,
    pub verifications: u64,
    pub artifacts: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusDecisionInboxWire {
    pub pending_count: u64,
    pub protocol_ready_count: u64,
    pub protocol_blocked_count: u64,
    pub projection_root: Option<String>,
    pub first_entry_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusActionsWire {
    pub review: Option<StatusReviewActionWire>,
    pub work: StatusWorkActionWire,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusReviewActionWire {
    pub pending_count: u64,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub(crate) enum StatusWorkActionWire {
    DirectSubmission { command: String, note: String },
    AuthorityUninitialized { command: String, note: String },
}

impl StatusWorkActionWire {
    pub(crate) fn parts(&self) -> (&'static str, &str, &str) {
        match self {
            Self::DirectSubmission { command, note } => ("direct_submission", command, note),
            Self::AuthorityUninitialized { command, note } => {
                ("authority_uninitialized", command, note)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ClaimsV1Wire {
    pub schema: String,
    pub ok: bool,
    pub command: String,
    pub repository_id: String,
    pub repository_root: String,
    pub status: String,
    pub order: String,
    pub total: u64,
    pub returned: u64,
    pub unreadable_returned: u64,
    pub next_cursor: Option<String>,
    pub items: Vec<ClaimItemWire>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ClaimItemWire {
    pub claim_id: String,
    pub claim_root: String,
    pub standing: String,
    pub origin_era: String,
    pub readable: bool,
    pub assertion_kind: Option<String>,
    pub assertion: Option<String>,
    pub unreadable_reason: Option<String>,
    pub created_at: Option<String>,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IntegrationCheckV1Wire {
    pub schema: String,
    pub ok: bool,
    pub command: String,
    pub authority_effect: String,
    pub manifest_root: String,
    pub documents_checked: u64,
    pub does_not_establish: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IntegrationInspectionV1Wire {
    pub schema: String,
    pub ok: bool,
    pub command: String,
    pub authority_effect: String,
    pub repository: String,
    pub revision: String,
    pub manifest_root: String,
    pub profiles: Vec<IntegrationItemWire>,
    pub bindings: Vec<IntegrationItemWire>,
    pub methods: Vec<IntegrationItemWire>,
    pub does_not_establish: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IntegrationItemWire {
    pub kind: String,
    pub id: String,
    pub path: String,
    pub root: String,
}
