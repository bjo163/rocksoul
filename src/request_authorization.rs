//! Deterministic request-grounded authorization for proposed side-effectful tool writes.
//!
//! This module commits the complete write set before untrusted tool observations are read.
//! Authority-bearing values must be present as exact tokens in the initiating user request;
//! tool results cannot add or redirect operations. This first slice intentionally denies
//! ungrounded values rather than guessing or silently accepting model-written authorization.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::tool_audit::ToolFailureCategory;

const MAX_TASK_ID_BYTES: usize = 96;
const MAX_TOOL_ID_BYTES: usize = 96;
const MAX_ARGUMENTS_PER_OPERATION: usize = 64;
const MAX_WRITE_OPERATIONS: usize = 64;
const MAX_ARGUMENT_VALUE_BYTES: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentRole {
    /// Selects the affected party, account, destination, amount, or object.
    Authority,
    /// Content that does not select the target of an operation.
    Payload,
    /// Typed control values whose exact allowed set is declared by policy.
    TemporalControl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgumentRule {
    pub role: ArgumentRole,
    pub required: bool,
    /// Used only for TemporalControl. Empty means deny all values.
    pub allowed_values: BTreeSet<String>,
}

impl ArgumentRule {
    pub fn authority(required: bool) -> Self {
        Self {
            role: ArgumentRole::Authority,
            required,
            allowed_values: BTreeSet::new(),
        }
    }

    pub fn payload(required: bool) -> Self {
        Self {
            role: ArgumentRole::Payload,
            required,
            allowed_values: BTreeSet::new(),
        }
    }

    pub fn temporal_control(
        required: bool,
        allowed_values: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            role: ArgumentRole::TemporalControl,
            required,
            allowed_values: allowed_values.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolWriteSchema {
    pub tool_id: String,
    pub arguments: BTreeMap<String, ArgumentRule>,
}

impl ToolWriteSchema {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.tool_id.is_empty() && self.tool_id.len() <= MAX_TOOL_ID_BYTES,
            "invalid tool id"
        );
        ensure!(
            !self.arguments.is_empty() && self.arguments.len() <= MAX_ARGUMENTS_PER_OPERATION,
            "invalid argument schema size"
        );
        for (name, rule) in &self.arguments {
            ensure!(
                !name.is_empty() && name.len() <= 96,
                "invalid argument name"
            );
            ensure!(
                name.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-')),
                "invalid argument name"
            );
            if rule.role == ArgumentRole::TemporalControl {
                ensure!(
                    !rule.allowed_values.is_empty(),
                    "temporal/control arguments require an explicit allowed-value set"
                );
                ensure!(
                    rule.allowed_values
                        .iter()
                        .all(|v| !v.is_empty() && v.len() <= MAX_ARGUMENT_VALUE_BYTES),
                    "invalid temporal/control allowed value"
                );
            } else {
                ensure!(
                    rule.allowed_values.is_empty(),
                    "allowed values are only valid for temporal/control arguments"
                );
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteProposal {
    pub tool_id: String,
    pub arguments: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct RequestGrounding {
    task_id: String,
    request_sha256: String,
    original_request: String,
}

impl RequestGrounding {
    /// Construct only from the initiating user request, never from model or tool output.
    pub fn from_user_request(
        task_id: impl Into<String>,
        original_request: impl Into<String>,
    ) -> Result<Self> {
        let task_id = task_id.into();
        let original_request = original_request.into();
        ensure!(
            !task_id.is_empty() && task_id.len() <= MAX_TASK_ID_BYTES,
            "invalid task id"
        );
        ensure!(
            !original_request.trim().is_empty(),
            "initiating user request is empty"
        );
        let digest = Sha256::digest(original_request.as_bytes());
        let request_sha256 = digest.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            task_id,
            request_sha256,
            original_request,
        })
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    fn grounds_exact_token(&self, value: &str) -> bool {
        contains_exact_token(&self.original_request, value)
    }

    /// Bind a single operation to an explicit user confirmation. The confirmation
    /// must contain every authority-bearing value as an exact token. The resulting
    /// grant is task-bound, request-bound, operation-bound, and single-use.
    pub fn confirm_operation(
        &self,
        schema: &ToolWriteSchema,
        proposal: &WriteProposal,
        confirmation: &str,
    ) -> Result<ConfirmationGrant> {
        schema.validate()?;
        validate_proposal_shape(schema, proposal)
            .map_err(|reason| anyhow::anyhow!("write proposal is invalid: {}", reason.as_str()))?;
        ensure!(
            !confirmation.trim().is_empty() && confirmation.len() <= MAX_ARGUMENT_VALUE_BYTES * 8,
            "confirmation text is empty or too large"
        );
        let mut authority_values = BTreeMap::new();
        for (name, value) in &proposal.arguments {
            let rule = schema
                .arguments
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("unclassified write argument"))?;
            if rule.role == ArgumentRole::Authority {
                ensure!(
                    contains_exact_token(confirmation, value),
                    "confirmation does not contain the exact authority value for {name}"
                );
                authority_values.insert(name.clone(), value.clone());
            }
        }
        ensure!(
            !authority_values.is_empty(),
            "operation has no authority-bearing values to confirm"
        );
        Ok(ConfirmationGrant {
            task_id: self.task_id.clone(),
            request_sha256: self.request_sha256.clone(),
            operation_digest: operation_digest(proposal),
            authority_values,
            used: false,
        })
    }
}

/// A task-bound, single-use grant created only from explicit user confirmation.
/// Model output and tool observations cannot construct or amend this type.
#[derive(Debug, Eq, PartialEq)]
pub struct ConfirmationGrant {
    task_id: String,
    request_sha256: String,
    operation_digest: String,
    authority_values: BTreeMap<String, String>,
    used: bool,
}

impl ConfirmationGrant {
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub fn operation_digest(&self) -> &str {
        &self.operation_digest
    }

    pub fn is_used(&self) -> bool {
        self.used
    }
}

fn contains_exact_token(text: &str, value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_ARGUMENT_VALUE_BYTES {
        return false;
    }
    text.match_indices(value).any(|(start, matched)| {
        let end = start + matched.len();
        let before_ok = start == 0 || !text[..start].chars().next_back().is_some_and(is_word_char);
        let after_ok = end == text.len() || !text[end..].chars().next().is_some_and(is_word_char);
        before_ok && after_ok
    })
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@' | '.' | '-' | '+')
}

fn operation_digest(proposal: &WriteProposal) -> String {
    let mut canonical = proposal.tool_id.clone();
    for (name, value) in &proposal.arguments {
        canonical.push('\u{1f}');
        canonical.push_str(name);
        canonical.push('\u{1f}');
        canonical.push_str(value);
    }
    Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Deterministic, content-free authorization reason codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationReason {
    Allowed,
    UnknownTool,
    UnclassifiedArgument,
    UngroundedAuthority,
    TemporalControlDenied,
    MissingRequiredArgument,
    WriteSetExhausted,
    WriteMutation,
    ConfirmationRequired,
    ConfirmationMismatch,
    ConfirmationReused,
    InvalidProposal,
    CommitAfterObservation,
    WriteSetNotCommitted,
    AlreadyCommitted,
    CandidateFieldNotAllowed,
    CandidateCannotCreateAuthority,
}

impl AuthorizationReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::UnknownTool => "unknown_tool",
            Self::UnclassifiedArgument => "unclassified_argument",
            Self::UngroundedAuthority => "ungrounded_authority",
            Self::TemporalControlDenied => "temporal_control_denied",
            Self::MissingRequiredArgument => "missing_required_argument",
            Self::WriteSetExhausted => "write_set_exhausted",
            Self::WriteMutation => "write_mutation",
            Self::ConfirmationRequired => "confirmation_required",
            Self::ConfirmationMismatch => "confirmation_mismatch",
            Self::ConfirmationReused => "confirmation_reused",
            Self::InvalidProposal => "invalid_proposal",
            Self::CommitAfterObservation => "commit_after_observation",
            Self::WriteSetNotCommitted => "write_set_not_committed",
            Self::AlreadyCommitted => "already_committed",
            Self::CandidateFieldNotAllowed => "candidate_field_not_allowed",
            Self::CandidateCannotCreateAuthority => "candidate_cannot_create_authority",
        }
    }

    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn audit_failure(self) -> Option<ToolFailureCategory> {
        match self {
            Self::Allowed => None,
            Self::UnknownTool | Self::TemporalControlDenied => {
                Some(ToolFailureCategory::PolicyDenied)
            }
            Self::UnclassifiedArgument => Some(ToolFailureCategory::UnclassifiedArgument),
            Self::CommitAfterObservation
            | Self::WriteSetNotCommitted
            | Self::AlreadyCommitted
            | Self::CandidateFieldNotAllowed
            | Self::CandidateCannotCreateAuthority => Some(ToolFailureCategory::PolicyDenied),
            Self::UngroundedAuthority => Some(ToolFailureCategory::UngroundedAuthority),
            Self::MissingRequiredArgument | Self::InvalidProposal => {
                Some(ToolFailureCategory::InvalidInput)
            }
            Self::WriteSetExhausted | Self::WriteMutation => {
                Some(ToolFailureCategory::WriteSetViolation)
            }
            Self::ConfirmationRequired => Some(ToolFailureCategory::ConfirmationRequired),
            Self::ConfirmationMismatch | Self::ConfirmationReused => {
                Some(ToolFailureCategory::ConfirmationRejected)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: AuthorizationReason,
    pub operation_index: Option<usize>,
}

fn validate_proposal_shape(
    schema: &ToolWriteSchema,
    proposal: &WriteProposal,
) -> std::result::Result<(), AuthorizationReason> {
    if proposal.tool_id != schema.tool_id {
        return Err(AuthorizationReason::InvalidProposal);
    }
    if proposal.arguments.len() > MAX_ARGUMENTS_PER_OPERATION {
        return Err(AuthorizationReason::InvalidProposal);
    }
    for (name, value) in &proposal.arguments {
        if value.len() > MAX_ARGUMENT_VALUE_BYTES {
            return Err(AuthorizationReason::InvalidProposal);
        }
        let rule = schema
            .arguments
            .get(name)
            .ok_or(AuthorizationReason::UnclassifiedArgument)?;
        if rule.role == ArgumentRole::TemporalControl && !rule.allowed_values.contains(value) {
            return Err(AuthorizationReason::TemporalControlDenied);
        }
    }
    for (name, rule) in &schema.arguments {
        if rule.required && !proposal.arguments.contains_key(name) {
            return Err(AuthorizationReason::MissingRequiredArgument);
        }
    }
    Ok(())
}

/// Immutable committed operations. There is deliberately no mutation API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedWriteSet {
    task_id: String,
    request_sha256: String,
    operations: Vec<WriteProposal>,
}

impl CommittedWriteSet {
    /// Validate and freeze every proposed write before any untrusted tool result is observed.
    pub fn commit(
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
    ) -> Result<Self> {
        Self::commit_with_reason(grounding, schemas, proposals)
            .map_err(|reason| anyhow::anyhow!("write authorization denied: {}", reason.as_str()))
    }

    /// Same as `commit`, but returns a deterministic content-free reason code.
    pub fn commit_with_reason(
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
    ) -> std::result::Result<Self, AuthorizationReason> {
        if proposals.is_empty() || proposals.len() > MAX_WRITE_OPERATIONS {
            return Err(AuthorizationReason::InvalidProposal);
        }
        for proposal in &proposals {
            let schema = schemas
                .get(&proposal.tool_id)
                .ok_or(AuthorizationReason::UnknownTool)?;
            schema
                .validate()
                .map_err(|_| AuthorizationReason::InvalidProposal)?;
            validate_proposal_shape(schema, proposal)?;
            for (name, value) in &proposal.arguments {
                let rule = schema
                    .arguments
                    .get(name)
                    .ok_or(AuthorizationReason::UnclassifiedArgument)?;
                if rule.role == ArgumentRole::Authority && !grounding.grounds_exact_token(value) {
                    return Err(AuthorizationReason::UngroundedAuthority);
                }
            }
        }
        Ok(Self {
            task_id: grounding.task_id.clone(),
            request_sha256: grounding.request_sha256.clone(),
            operations: proposals,
        })
    }

    /// Commit a write set where an ungrounded authority value is allowed only
    /// when the user explicitly confirmed that exact operation. Each grant is
    /// consumed once and must match the task, request digest, operation digest,
    /// and every authority value.
    pub fn commit_with_confirmations(
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
        confirmations: &mut [ConfirmationGrant],
    ) -> Result<Self> {
        ensure!(
            !proposals.is_empty() && proposals.len() <= MAX_WRITE_OPERATIONS,
            "write set size is outside the allowed range"
        );
        for proposal in &proposals {
            let schema = schemas
                .get(&proposal.tool_id)
                .ok_or_else(|| anyhow::anyhow!("unknown or unclassified write tool"))?;
            schema.validate()?;
            validate_proposal_shape(schema, proposal).map_err(|reason| {
                anyhow::anyhow!("write proposal is invalid: {}", reason.as_str())
            })?;
            let ungrounded: Vec<(&String, &String)> = proposal
                .arguments
                .iter()
                .filter(|(name, value)| {
                    schema.arguments.get(*name).is_some_and(|rule| {
                        rule.role == ArgumentRole::Authority
                            && !grounding.grounds_exact_token(value)
                    })
                })
                .collect();
            if ungrounded.is_empty() {
                continue;
            }
            let digest = operation_digest(proposal);
            let grant = confirmations
                .iter_mut()
                .find(|grant| {
                    grant.operation_digest == digest
                        && grant.task_id == grounding.task_id
                        && grant.request_sha256 == grounding.request_sha256
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "authority value requires explicit confirmation ({})",
                        AuthorizationReason::ConfirmationRequired.as_str()
                    )
                })?;
            ensure!(
                !grant.used,
                "confirmation was already consumed ({})",
                AuthorizationReason::ConfirmationReused.as_str()
            );
            for (name, value) in ungrounded {
                ensure!(
                    grant.authority_values.get(name) == Some(value),
                    "confirmation does not match authority value ({})",
                    AuthorizationReason::ConfirmationMismatch.as_str()
                );
            }
            grant.used = true;
        }
        Ok(Self {
            task_id: grounding.task_id.clone(),
            request_sha256: grounding.request_sha256.clone(),
            operations: proposals,
        })
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub fn operations(&self) -> &[WriteProposal] {
        &self.operations
    }

    pub fn execution_cursor(&self) -> WriteExecutionCursor {
        WriteExecutionCursor {
            committed: self.clone(),
            next_index: 0,
        }
    }
}

/// Consumes the committed sequence exactly once and in order. An observation cannot append,
/// remove, reorder, duplicate, or broaden the write set.
#[derive(Clone, Debug)]
pub struct WriteExecutionCursor {
    committed: CommittedWriteSet,
    next_index: usize,
}

impl WriteExecutionCursor {
    pub fn authorize_next(&mut self, candidate: &WriteProposal) -> Result<usize> {
        let decision = self.authorize_next_decision(candidate);
        if decision.allowed {
            Ok(decision.operation_index.unwrap_or_default())
        } else {
            anyhow::bail!("write authorization denied: {}", decision.reason.as_str())
        }
    }

    pub fn authorize_next_decision(&mut self, candidate: &WriteProposal) -> AuthorizationDecision {
        let Some(expected) = self.committed.operations.get(self.next_index) else {
            return AuthorizationDecision {
                allowed: false,
                reason: AuthorizationReason::WriteSetExhausted,
                operation_index: None,
            };
        };
        if candidate != expected {
            return AuthorizationDecision {
                allowed: false,
                reason: AuthorizationReason::WriteMutation,
                operation_index: None,
            };
        }
        let index = self.next_index;
        self.next_index += 1;
        AuthorizationDecision {
            allowed: true,
            reason: AuthorizationReason::Allowed,
            operation_index: Some(index),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.next_index == self.committed.operations.len()
    }

    pub fn remaining(&self) -> usize {
        self.committed
            .operations
            .len()
            .saturating_sub(self.next_index)
    }
}

/// Policy for the narrow case where a tool result may supply a typed candidate.
/// Only payload or policy-declared control values can be filled this way; an
/// authority value can never be created by an observation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ToolResultCandidatePolicy {
    pub allowed_fields: BTreeSet<String>,
}

impl ToolResultCandidatePolicy {
    pub fn allow(&mut self, field: impl Into<String>) {
        self.allowed_fields.insert(field.into());
    }

    pub fn apply(
        &self,
        schema: &ToolWriteSchema,
        proposal: &mut WriteProposal,
        field: &str,
        candidate: &str,
    ) -> std::result::Result<(), AuthorizationReason> {
        if !self.allowed_fields.contains(field) {
            return Err(AuthorizationReason::CandidateFieldNotAllowed);
        }
        if candidate.is_empty() || candidate.len() > MAX_ARGUMENT_VALUE_BYTES {
            return Err(AuthorizationReason::InvalidProposal);
        }
        let rule = schema
            .arguments
            .get(field)
            .ok_or(AuthorizationReason::UnclassifiedArgument)?;
        match rule.role {
            ArgumentRole::Authority => Err(AuthorizationReason::CandidateCannotCreateAuthority),
            ArgumentRole::Payload => {
                proposal
                    .arguments
                    .insert(field.to_owned(), candidate.to_owned());
                Ok(())
            }
            ArgumentRole::TemporalControl => {
                if rule.allowed_values.contains(candidate) {
                    proposal
                        .arguments
                        .insert(field.to_owned(), candidate.to_owned());
                    Ok(())
                } else {
                    Err(AuthorizationReason::TemporalControlDenied)
                }
            }
        }
    }
}

/// Kernel-owned gate that makes "commit before the first untrusted observation"
/// an enforced invariant instead of a convention. Once any tool observation is
/// recorded, no new write set can be committed; only the frozen set may execute.
#[derive(Debug, Default)]
pub struct WriteCommitmentGate {
    cursor: Option<WriteExecutionCursor>,
    observations_seen: u64,
}

impl WriteCommitmentGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observations_seen(&self) -> u64 {
        self.observations_seen
    }

    pub fn is_committed(&self) -> bool {
        self.cursor.is_some()
    }

    pub fn is_complete(&self) -> bool {
        self.cursor
            .as_ref()
            .is_some_and(WriteExecutionCursor::is_complete)
    }

    /// The observation source calls this whenever an untrusted result enters the run.
    pub fn note_observation(&mut self) -> Result<()> {
        self.observations_seen = self
            .observations_seen
            .checked_add(1)
            .context("observation counter exhausted")?;
        Ok(())
    }

    pub fn commit(
        &mut self,
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
    ) -> std::result::Result<(), AuthorizationReason> {
        if self.observations_seen > 0 {
            return Err(AuthorizationReason::CommitAfterObservation);
        }
        if self.cursor.is_some() {
            return Err(AuthorizationReason::AlreadyCommitted);
        }
        let committed = CommittedWriteSet::commit_with_reason(grounding, schemas, proposals)?;
        self.cursor = Some(committed.execution_cursor());
        Ok(())
    }

    /// Commit including explicit, task-bound confirmation grants.
    pub fn commit_with_confirmations(
        &mut self,
        grounding: &RequestGrounding,
        schemas: &BTreeMap<String, ToolWriteSchema>,
        proposals: Vec<WriteProposal>,
        confirmations: &mut [ConfirmationGrant],
    ) -> Result<()> {
        ensure!(
            self.observations_seen == 0,
            "write set cannot be committed after an untrusted observation ({})",
            AuthorizationReason::CommitAfterObservation.as_str()
        );
        ensure!(
            self.cursor.is_none(),
            "a write set is already committed for this run"
        );
        let committed = CommittedWriteSet::commit_with_confirmations(
            grounding,
            schemas,
            proposals,
            confirmations,
        )?;
        self.cursor = Some(committed.execution_cursor());
        Ok(())
    }

    /// Authorize the next write in the frozen set. A missing commitment is a
    /// denial, never an implicit acceptance.
    pub fn authorize_next(&mut self, candidate: &WriteProposal) -> AuthorizationDecision {
        let Some(cursor) = self.cursor.as_mut() else {
            return AuthorizationDecision {
                allowed: false,
                reason: AuthorizationReason::WriteSetNotCommitted,
                operation_index: None,
            };
        };
        cursor.authorize_next_decision(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> ToolWriteSchema {
        ToolWriteSchema {
            tool_id: "mail.send".to_owned(),
            arguments: BTreeMap::from([
                ("recipient".to_owned(), ArgumentRule::authority(true)),
                ("subject".to_owned(), ArgumentRule::payload(true)),
                ("body".to_owned(), ArgumentRule::payload(true)),
                (
                    "send_at".to_owned(),
                    ArgumentRule::temporal_control(false, ["now".to_owned(), "draft".to_owned()]),
                ),
            ]),
        }
    }

    fn proposal(recipient: &str) -> WriteProposal {
        WriteProposal {
            tool_id: "mail.send".to_owned(),
            arguments: BTreeMap::from([
                ("recipient".to_owned(), recipient.to_owned()),
                ("subject".to_owned(), "Project update".to_owned()),
                ("body".to_owned(), "The report is ready.".to_owned()),
                ("send_at".to_owned(), "draft".to_owned()),
            ]),
        }
    }

    #[test]
    fn exact_request_grounded_authority_can_be_committed() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to alex@example.com")
                .unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let committed =
            CommittedWriteSet::commit(&grounding, &schemas, vec![proposal("alex@example.com")])
                .unwrap();
        assert_eq!(committed.operations().len(), 1);
        assert_eq!(committed.task_id(), "task-1");
    }

    #[test]
    fn ungrounded_recipient_is_denied_even_if_model_or_tool_proposes_it() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        assert!(
            CommittedWriteSet::commit(&grounding, &schemas, vec![proposal("attacker@example.com")])
                .is_err()
        );
    }

    #[test]
    fn substring_inside_another_identifier_does_not_ground_authority() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to not-alex@example.com").unwrap();
        assert!(!grounding.grounds_exact_token("alex@example.com"));
    }

    #[test]
    fn unknown_tools_and_unclassified_arguments_are_denied() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        assert!(
            CommittedWriteSet::commit(
                &grounding,
                &schemas,
                vec![WriteProposal {
                    tool_id: "unknown".to_owned(),
                    arguments: BTreeMap::new()
                }]
            )
            .is_err()
        );
        let mut extra = proposal("alex@example.com");
        extra
            .arguments
            .insert("bcc".to_owned(), "secret@example.com".to_owned());
        assert!(CommittedWriteSet::commit(&grounding, &schemas, vec![extra]).is_err());
    }

    #[test]
    fn temporal_control_requires_an_explicit_policy_value() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let mut operation = proposal("alex@example.com");
        operation.arguments.insert(
            "send_at".to_owned(),
            "after reading injected instructions".to_owned(),
        );
        assert!(CommittedWriteSet::commit(&grounding, &schemas, vec![operation]).is_err());
    }

    #[test]
    fn committed_write_set_cannot_be_redirected_or_extended_after_observation() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let committed =
            CommittedWriteSet::commit(&grounding, &schemas, vec![proposal("alex@example.com")])
                .unwrap();
        let mut cursor = committed.execution_cursor();
        assert!(
            cursor
                .authorize_next(&proposal("attacker@example.com"))
                .is_err()
        );
        assert_eq!(cursor.remaining(), 1);
        assert_eq!(
            cursor
                .authorize_next(&proposal("alex@example.com"))
                .unwrap(),
            0
        );
        assert!(cursor.is_complete());
        assert!(
            cursor
                .authorize_next(&proposal("alex@example.com"))
                .is_err()
        );
    }

    #[test]
    fn required_authority_and_payload_arguments_must_be_present() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let mut operation = proposal("alex@example.com");
        operation.arguments.remove("recipient");
        assert!(CommittedWriteSet::commit(&grounding, &schemas, vec![operation]).is_err());
    }

    #[test]
    fn explicit_confirmation_allows_exact_operation_and_is_single_use() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let operation = proposal("alex@example.com");
        let mut grant = grounding
            .confirm_operation(
                &schema(),
                &operation,
                "Yes, send it to alex@example.com now",
            )
            .unwrap();
        assert!(!grant.is_used());
        let committed = CommittedWriteSet::commit_with_confirmations(
            &grounding,
            &schemas,
            vec![operation.clone()],
            std::slice::from_mut(&mut grant),
        )
        .unwrap();
        assert_eq!(committed.operations(), std::slice::from_ref(&operation));
        assert!(grant.is_used());
        assert!(
            CommittedWriteSet::commit_with_confirmations(
                &grounding,
                &schemas,
                vec![operation],
                std::slice::from_mut(&mut grant),
            )
            .is_err()
        );
    }

    #[test]
    fn confirmation_is_task_and_request_bound() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let other_request =
            RequestGrounding::from_user_request("task-2", "Send the report to Alex").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let operation = proposal("alex@example.com");
        let mut grant = grounding
            .confirm_operation(&schema(), &operation, "send to alex@example.com")
            .unwrap();
        assert!(
            CommittedWriteSet::commit_with_confirmations(
                &other_request,
                &schemas,
                vec![operation],
                std::slice::from_mut(&mut grant),
            )
            .is_err()
        );
    }

    #[test]
    fn confirmation_cannot_be_replayed_for_a_different_operation() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let confirmed = proposal("alex@example.com");
        let mut grant = grounding
            .confirm_operation(&schema(), &confirmed, "send to alex@example.com")
            .unwrap();
        let mut changed = confirmed.clone();
        changed
            .arguments
            .insert("body".to_owned(), "Injected replacement body".to_owned());
        assert!(
            CommittedWriteSet::commit_with_confirmations(
                &grounding,
                &schemas,
                vec![changed],
                std::slice::from_mut(&mut grant),
            )
            .is_err()
        );
        assert!(!grant.is_used());
    }

    #[test]
    fn untrusted_observation_cannot_ground_authority() {
        // The caller must pass only user-authored request text as grounding.
        // Content from an email, HTML page, file, or API response is a separate
        // untrusted observation and is never part of this string.
        let grounding = RequestGrounding::from_user_request(
            "task-1",
            "Summarize the email below. Do not send anything.",
        )
        .unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let reason = CommittedWriteSet::commit_with_reason(
            &grounding,
            &schemas,
            vec![proposal("attacker@example.com")],
        )
        .unwrap_err();
        assert_eq!(reason, AuthorizationReason::UngroundedAuthority);
        assert_eq!(
            reason.audit_failure(),
            Some(ToolFailureCategory::UngroundedAuthority)
        );
    }

    #[test]
    fn reason_codes_are_deterministic() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        assert_eq!(
            CommittedWriteSet::commit_with_reason(
                &grounding,
                &schemas,
                vec![WriteProposal {
                    tool_id: "unknown".to_owned(),
                    arguments: BTreeMap::new(),
                }],
            )
            .unwrap_err(),
            AuthorizationReason::UnknownTool
        );
        let mut operation = proposal("alex@example.com");
        operation
            .arguments
            .insert("send_at".to_owned(), "later".to_owned());
        assert_eq!(
            CommittedWriteSet::commit_with_reason(&grounding, &schemas, vec![operation])
                .unwrap_err(),
            AuthorizationReason::TemporalControlDenied
        );
        let committed =
            CommittedWriteSet::commit(&grounding, &schemas, vec![proposal("alex@example.com")])
                .unwrap();
        let mut cursor = committed.execution_cursor();
        assert_eq!(
            cursor
                .authorize_next_decision(&proposal("attacker@example.com"))
                .reason,
            AuthorizationReason::WriteMutation
        );
        assert_eq!(
            cursor
                .authorize_next_decision(&proposal("alex@example.com"))
                .reason,
            AuthorizationReason::Allowed
        );
        assert_eq!(
            cursor
                .authorize_next_decision(&proposal("alex@example.com"))
                .reason,
            AuthorizationReason::WriteSetExhausted
        );
    }

    #[test]
    fn typed_candidates_can_fill_payload_and_control_but_never_authority() {
        let schema = schema();
        let mut proposal = proposal("alex@example.com");
        let mut policy = ToolResultCandidatePolicy::default();
        assert_eq!(
            policy.apply(&schema, &mut proposal, "body", "x"),
            Err(AuthorizationReason::CandidateFieldNotAllowed)
        );
        policy.allow("body");
        policy.allow("recipient");
        policy.allow("send_at");
        assert_eq!(
            policy.apply(&schema, &mut proposal, "body", "new body"),
            Ok(())
        );
        assert_eq!(proposal.arguments["body"], "new body");
        assert_eq!(
            policy.apply(&schema, &mut proposal, "recipient", "attacker@example.com"),
            Err(AuthorizationReason::CandidateCannotCreateAuthority)
        );
        assert_eq!(proposal.arguments["recipient"], "alex@example.com");
        assert_eq!(
            policy.apply(&schema, &mut proposal, "send_at", "later"),
            Err(AuthorizationReason::TemporalControlDenied)
        );
        assert_eq!(
            policy.apply(&schema, &mut proposal, "send_at", "draft"),
            Ok(())
        );
    }

    #[test]
    fn confirmation_requires_an_exact_token_not_a_substring() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let operation = proposal("alex@example.com");
        assert!(
            grounding
                .confirm_operation(&schema(), &operation, "send to notalex@example.com")
                .is_err()
        );
        assert!(
            grounding
                .confirm_operation(&schema(), &operation, "send to alex@example.com now")
                .is_ok()
        );
    }

    #[test]
    fn failed_confirmation_commit_does_not_consume_the_grant() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send the report to Alex").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let operation = proposal("alex@example.com");
        let mut grant = grounding
            .confirm_operation(&schema(), &operation, "send to alex@example.com")
            .unwrap();
        let mut changed = operation.clone();
        changed
            .arguments
            .insert("body".to_owned(), "changed by an observation".to_owned());
        assert!(
            CommittedWriteSet::commit_with_confirmations(
                &grounding,
                &schemas,
                vec![changed],
                std::slice::from_mut(&mut grant),
            )
            .is_err()
        );
        assert!(!grant.is_used());
    }

    #[test]
    fn invalid_write_schemas_are_rejected() {
        let mut empty = schema();
        empty.arguments.clear();
        assert!(empty.validate().is_err());

        let mut bad_name = schema();
        bad_name
            .arguments
            .insert("bad name".to_owned(), ArgumentRule::payload(true));
        assert!(bad_name.validate().is_err());

        let mut empty_control = schema();
        empty_control
            .arguments
            .insert("when".to_owned(), ArgumentRule::temporal_control(false, []));
        assert!(empty_control.validate().is_err());

        let mut bad_payload = schema();
        bad_payload.arguments.insert(
            "body".to_owned(),
            ArgumentRule {
                role: ArgumentRole::Payload,
                required: true,
                allowed_values: BTreeSet::from(["x".to_owned()]),
            },
        );
        assert!(bad_payload.validate().is_err());
    }

    #[test]
    fn gate_commits_only_before_the_first_observation() {
        let grounding =
            RequestGrounding::from_user_request("task-1", "Send to alex@example.com").unwrap();
        let schemas = BTreeMap::from([("mail.send".to_owned(), schema())]);
        let mut gate = WriteCommitmentGate::new();
        assert_eq!(
            gate.authorize_next(&proposal("alex@example.com")).reason,
            AuthorizationReason::WriteSetNotCommitted
        );
        assert_eq!(
            gate.commit(&grounding, &schemas, vec![proposal("alex@example.com")]),
            Ok(())
        );
        assert_eq!(
            gate.commit(&grounding, &schemas, vec![proposal("alex@example.com")]),
            Err(AuthorizationReason::AlreadyCommitted)
        );
        assert_eq!(
            gate.authorize_next(&proposal("attacker@example.com"))
                .reason,
            AuthorizationReason::WriteMutation
        );
        assert!(gate.authorize_next(&proposal("alex@example.com")).allowed);
        assert!(gate.is_complete());
        gate.note_observation().unwrap();
        let mut second = WriteCommitmentGate::new();
        second.note_observation().unwrap();
        assert_eq!(
            second.commit(&grounding, &schemas, vec![proposal("alex@example.com")]),
            Err(AuthorizationReason::CommitAfterObservation)
        );
        assert_eq!(
            AuthorizationReason::CommitAfterObservation.audit_failure(),
            Some(ToolFailureCategory::PolicyDenied)
        );
    }
}
