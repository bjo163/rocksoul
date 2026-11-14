//! Deterministic request-grounded authorization for proposed side-effectful tool writes.
//!
//! This module commits the complete write set before untrusted tool observations are read.
//! Authority-bearing values must be present as exact tokens in the initiating user request;
//! tool results cannot add or redirect operations. This first slice intentionally denies
//! ungrounded values rather than guessing or silently accepting model-written authorization.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use sha2::{Digest, Sha256};

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
            !self.arguments.is_empty()
                && self.arguments.len() <= MAX_ARGUMENTS_PER_OPERATION,
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
        if value.is_empty() || value.len() > MAX_ARGUMENT_VALUE_BYTES {
            return false;
        }
        self.original_request
            .match_indices(value)
            .any(|(start, matched)| {
                let end = start + matched.len();
                let before_ok = start == 0
                    || !self.original_request[..start]
                        .chars()
                        .next_back()
                        .is_some_and(is_word_char);
                let after_ok = end == self.original_request.len()
                    || !self.original_request[end..]
                        .chars()
                        .next()
                        .is_some_and(is_word_char);
                before_ok && after_ok
            })
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@' | '.' | '-' | '+')
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
        ensure!(
            !proposals.is_empty() && proposals.len() <= MAX_WRITE_OPERATIONS,
            "write set size is outside the allowed range"
        );
        for proposal in &proposals {
            let schema = schemas
                .get(&proposal.tool_id)
                .ok_or_else(|| anyhow::anyhow!("unknown or unclassified write tool"))?;
            schema.validate()?;
            ensure!(proposal.tool_id == schema.tool_id, "tool schema key mismatch");
            ensure!(
                proposal.arguments.len() <= MAX_ARGUMENTS_PER_OPERATION,
                "too many write arguments"
            );
            for (name, value) in &proposal.arguments {
                ensure!(
                    value.len() <= MAX_ARGUMENT_VALUE_BYTES,
                    "write argument exceeds size limit"
                );
                let rule = schema
                    .arguments
                    .get(name)
                    .ok_or_else(|| anyhow::anyhow!("unclassified write argument"))?;
                match rule.role {
                    ArgumentRole::Authority => ensure!(
                        grounding.grounds_exact_token(value),
                        "authority value is not grounded in the initiating user request"
                    ),
                    ArgumentRole::Payload => {}
                    ArgumentRole::TemporalControl => ensure!(
                        rule.allowed_values.contains(value),
                        "temporal/control value is not explicitly allowed"
                    ),
                }
            }
            for (name, rule) in &schema.arguments {
                if rule.required {
                    ensure!(
                        proposal.arguments.contains_key(name),
                        "required write argument is missing"
                    );
                }
            }
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
        let expected = self
            .committed
            .operations
            .get(self.next_index)
            .ok_or_else(|| anyhow::anyhow!("committed write set is exhausted"))?;
        ensure!(
            candidate == expected,
            "write differs from the immutable committed operation"
        );
        let index = self.next_index;
        self.next_index += 1;
        Ok(index)
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
                    ArgumentRule::temporal_control(
                        false,
                        ["now".to_owned(), "draft".to_owned()],
                    ),
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
            CommittedWriteSet::commit(
                &grounding,
                &schemas,
                vec![proposal("attacker@example.com")]
            )
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
}
