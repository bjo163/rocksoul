use crate::sense::SenseConnector;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseCapability {
    pub id: String,
    pub kind: String,
    pub read_only: bool,
    pub scope: String,
    pub redaction: String,
    pub max_age: Duration,
    pub max_calls: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("sense capability id is invalid")]
    InvalidId,
    #[error("sense capability must be read-only")]
    MutationNotAllowed,
    #[error("sense capability is already registered")]
    Duplicate,
    #[error("sense capability is not registered")]
    Missing,
}

#[derive(Debug, Default)]
pub struct SenseRegistry {
    capabilities: Vec<SenseCapability>,
}

impl SenseRegistry {
    pub fn register(&mut self, capability: SenseCapability) -> Result<(), RegistryError> {
        if capability.id.trim().is_empty()
            || capability.kind.trim().is_empty()
            || capability.scope.trim().is_empty()
        {
            return Err(RegistryError::InvalidId);
        }
        if !capability.read_only {
            return Err(RegistryError::MutationNotAllowed);
        }
        if self
            .capabilities
            .iter()
            .any(|item| item.id == capability.id)
        {
            return Err(RegistryError::Duplicate);
        }
        self.capabilities.push(capability);
        Ok(())
    }
    pub fn capabilities(&self) -> &[SenseCapability] {
        &self.capabilities
    }
    pub fn contains(&self, id: &str) -> bool {
        self.capabilities.iter().any(|item| item.id == id)
    }
    pub fn observe<C: SenseConnector>(
        &self,
        id: &str,
        connector: &C,
        subject: &str,
    ) -> Result<crate::cognition::Observation, RegistryError> {
        if !self.contains(id) {
            return Err(RegistryError::Missing);
        }
        connector
            .observe(subject)
            .map_err(|_| RegistryError::Missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sense::FixtureSense;
    #[test]
    fn registry_declares_read_only_scoped_senses() {
        let mut registry = SenseRegistry::default();
        registry
            .register(SenseCapability {
                id: "repository".into(),
                kind: "repository".into(),
                read_only: true,
                scope: "local".into(),
                redaction: "none".into(),
                max_age: Duration::from_secs(60),
                max_calls: 10,
            })
            .unwrap();
        assert!(registry.contains("repository"));
        assert_eq!(registry.capabilities().len(), 1);
    }
    #[test]
    fn registry_rejects_mutation_duplicate_and_missing() {
        let mut registry = SenseRegistry::default();
        let capability = SenseCapability {
            id: "x".into(),
            kind: "fixture".into(),
            read_only: false,
            scope: "test".into(),
            redaction: "all".into(),
            max_age: Duration::ZERO,
            max_calls: 1,
        };
        assert_eq!(
            registry.register(capability.clone()),
            Err(RegistryError::MutationNotAllowed)
        );
        let mut safe = capability;
        safe.read_only = true;
        registry.register(safe.clone()).unwrap();
        assert_eq!(registry.register(safe), Err(RegistryError::Duplicate));
        let connector = FixtureSense {
            name: "fixture".into(),
            payload: serde_json::json!({"ok":true}),
            max_bytes: 100,
            online: true,
        };
        assert_eq!(
            registry.observe("missing", &connector, "x"),
            Err(RegistryError::Missing)
        );
    }
}
