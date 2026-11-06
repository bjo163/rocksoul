//! Deny-by-default metadata registry for future agent tools.
//!
//! This module only authorizes tool *descriptions*. It does not execute tools,
//! spawn processes, access the filesystem, or grant ambient network access.
//! Runtime execution must remain behind a separately reviewed sandbox boundary.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail, ensure};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ToolCapability {
    WorkspaceRead,
    WorkspaceWrite,
    MemoryRead,
    MemoryWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolEffect {
    ReadOnly,
    Mutating,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDescriptor {
    pub id: String,
    pub version: String,
    pub capability: ToolCapability,
    pub effect: ToolEffect,
    pub max_input_bytes: usize,
}

impl ToolDescriptor {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.id.is_empty()
                && self.id.len() <= 96
                && self.id.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'_' | b'-')
                }),
            "tool id must be 1-96 lowercase ASCII identifier characters"
        );
        ensure!(
            !self.version.trim().is_empty() && self.version.len() <= 64,
            "tool version must be non-empty and at most 64 bytes"
        );
        ensure!(
            (1..=1_048_576).contains(&self.max_input_bytes),
            "tool input limit must be between 1 byte and 1 MiB"
        );
        Ok(())
    }
}

/// Empty by default: a tool must be explicitly allowlisted and its capability
/// explicitly granted before it can be selected for a run.
#[derive(Clone, Debug, Default)]
pub struct ToolPolicy {
    pub allowed_tool_ids: BTreeSet<String>,
    pub granted_capabilities: BTreeSet<ToolCapability>,
    /// Aggregate per-invocation input ceiling. Zero denies all non-empty input.
    pub max_input_bytes: usize,
}

impl ToolPolicy {
    pub fn allow_tool(&mut self, id: impl Into<String>) {
        self.allowed_tool_ids.insert(id.into());
    }

    pub fn grant(&mut self, capability: ToolCapability) {
        self.granted_capabilities.insert(capability);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, ToolDescriptor>,
}

impl ToolRegistry {
    pub fn register(&mut self, descriptor: ToolDescriptor) -> Result<()> {
        descriptor.validate()?;
        let id = descriptor.id.clone();
        if self.tools.contains_key(&id) {
            bail!("tool id is already registered: {id}");
        }
        self.tools.insert(id, descriptor);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ToolDescriptor> {
        self.tools.get(id)
    }

    pub fn authorize<'a>(
        &'a self,
        id: &str,
        input_bytes: usize,
        policy: &ToolPolicy,
    ) -> Result<&'a ToolDescriptor> {
        let descriptor = self
            .tools
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {id}"))?;
        ensure!(
            policy.allowed_tool_ids.contains(id),
            "tool is not explicitly allowlisted"
        );
        ensure!(
            policy.granted_capabilities.contains(&descriptor.capability),
            "tool capability has not been granted"
        );
        ensure!(
            input_bytes <= descriptor.max_input_bytes && input_bytes <= policy.max_input_bytes,
            "tool input exceeds the configured byte budget"
        );
        Ok(descriptor)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{ToolCapability, ToolDescriptor, ToolEffect, ToolPolicy, ToolRegistry};

    fn descriptor() -> ToolDescriptor {
        ToolDescriptor {
            id: "workspace.read".to_owned(),
            version: "1.0.0".to_owned(),
            capability: ToolCapability::WorkspaceRead,
            effect: ToolEffect::ReadOnly,
            max_input_bytes: 128,
        }
    }

    #[test]
    fn registry_is_empty_and_policy_denies_by_default() {
        let registry = ToolRegistry::default();
        let policy = ToolPolicy::default();
        assert!(registry.is_empty());
        assert!(registry.authorize("workspace.read", 0, &policy).is_err());
    }

    #[test]
    fn authorization_requires_allowlist_and_capability_grant() {
        let mut registry = ToolRegistry::default();
        registry.register(descriptor()).unwrap();
        let mut policy = ToolPolicy {
            max_input_bytes: 128,
            ..ToolPolicy::default()
        };
        assert!(registry.authorize("workspace.read", 1, &policy).is_err());
        policy.allow_tool("workspace.read");
        assert!(registry.authorize("workspace.read", 1, &policy).is_err());
        policy.grant(ToolCapability::WorkspaceRead);
        assert!(registry.authorize("workspace.read", 1, &policy).is_ok());
    }

    #[test]
    fn authorization_enforces_both_input_budgets() {
        let mut registry = ToolRegistry::default();
        registry.register(descriptor()).unwrap();
        let mut policy = ToolPolicy {
            max_input_bytes: 64,
            ..ToolPolicy::default()
        };
        policy.allow_tool("workspace.read");
        policy.grant(ToolCapability::WorkspaceRead);
        assert!(registry.authorize("workspace.read", 65, &policy).is_err());
        policy.max_input_bytes = 256;
        assert!(registry.authorize("workspace.read", 129, &policy).is_err());
    }

    #[test]
    fn duplicate_and_invalid_descriptors_are_rejected() {
        let mut registry = ToolRegistry::default();
        registry.register(descriptor()).unwrap();
        assert!(registry.register(descriptor()).is_err());
        let mut invalid = descriptor();
        invalid.id = "Workspace.Read".to_owned();
        assert!(registry.register(invalid).is_err());
    }

    #[test]
    fn mutating_effect_is_explicit_metadata_not_implicit_permission() {
        let mut registry = ToolRegistry::default();
        let mut descriptor = descriptor();
        descriptor.id = "workspace.write".to_owned();
        descriptor.capability = ToolCapability::WorkspaceWrite;
        descriptor.effect = ToolEffect::Mutating;
        registry.register(descriptor).unwrap();
        let mut policy = ToolPolicy {
            max_input_bytes: 128,
            ..ToolPolicy::default()
        };
        policy.allow_tool("workspace.write");
        policy.grant(ToolCapability::WorkspaceRead);
        assert!(registry.authorize("workspace.write", 1, &policy).is_err());
        policy.grant(ToolCapability::WorkspaceWrite);
        assert!(registry.authorize("workspace.write", 1, &policy).is_ok());
    }
}
