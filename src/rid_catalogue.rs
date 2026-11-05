use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

const EMBEDDED_CATALOGUE: &str = include_str!("../data/rid/universal_stats.v2.json");

#[derive(Clone, Debug, Deserialize)]
pub struct RidCatalogue {
    pub schema_version: u16,
    pub catalogue_id: String,
    pub catalogue_version: String,
    pub total_domains: usize,
    pub total_stats: usize,
    pub scoring: ScoringPolicy,
    pub domains: Vec<UniversalDomain>,
    pub entries: Vec<CatalogueEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScoringPolicy {
    pub default_minimum_samples: u32,
    pub safety_sensitive_evaluation_minimum_samples: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniversalDomain {
    pub id: String,
    pub key: String,
    pub label: String,
    pub definition: String,
    pub stat_id: String,
    pub slot: u8,
    pub measurement_target: String,
    pub inclusion_criteria: String,
    pub exclusion_criteria: String,
    pub overlap_with: Vec<String>,
    pub separation_rule: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogueEntry {
    pub id: u8,
    pub stat_id: String,
    pub key: String,
    pub label: String,
    pub domain: String,
    pub domain_label: String,
    pub definition: String,
    pub evaluation_protocol: String,
    pub status: String,
    pub initial_state: String,
    pub domain_id: String,
    pub measurement_target: String,
    pub inclusion_criteria: String,
    pub exclusion_criteria: String,
    pub overlap_with: Vec<String>,
    pub separation_rule: String,
}

impl RidCatalogue {
    pub fn embedded() -> Result<Self> {
        let catalogue: Self = serde_json::from_str(EMBEDDED_CATALOGUE)
            .context("embedded universal RID catalogue is not valid JSON")?;
        catalogue.validate()?;
        Ok(catalogue)
    }

    pub fn by_slot(&self, slot_id: u8) -> Option<&CatalogueEntry> {
        self.entries
            .get(usize::from(slot_id.saturating_sub(1)))
            .filter(|entry| entry.id == slot_id)
    }

    pub fn minimum_samples_for_slot(&self, slot_id: u8) -> Option<u32> {
        self.by_slot(slot_id)?;
        let safety_sensitive = (56..=66).contains(&slot_id);
        Some(if safety_sensitive {
            self.scoring.safety_sensitive_evaluation_minimum_samples
        } else {
            self.scoring.default_minimum_samples
        })
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 2, "unsupported universal RID catalogue schema");
        ensure!(
            self.catalogue_id == "rocksoul.rid.universal_stats.v2",
            "unexpected universal RID catalogue identifier"
        );
        ensure!(self.catalogue_version == "2.1.0", "unexpected universal RID catalogue version");
        ensure!(self.total_domains == 99 && self.total_stats == 99, "catalogue totals must both equal 99");
        ensure!(self.domains.len() == 99, "RID catalogue must contain exactly 99 domains");
        ensure!(self.entries.len() == 99, "RID catalogue must contain exactly 99 stats");
        ensure!(
            self.scoring.default_minimum_samples > 0
                && self.scoring.safety_sensitive_evaluation_minimum_samples
                    >= self.scoring.default_minimum_samples,
            "RID scoring sample policy is invalid"
        );

        let mut domain_ids = HashSet::with_capacity(99);
        let mut domain_keys = HashSet::with_capacity(99);
        let mut stat_ids = HashSet::with_capacity(99);
        let mut stat_keys = HashSet::with_capacity(99);
        for (index, domain) in self.domains.iter().enumerate() {
            let slot = index as u8 + 1;
            ensure!(domain.slot == slot, "RID domain slot order mismatch at {slot}");
            ensure!(domain.id == format!("rid_domain:{slot:02}"), "RID domain ID mismatch at {slot}");
            ensure!(domain.stat_id == format!("rid_stat:{slot:02}"), "RID domain stat link mismatch at {slot}");
            ensure!(
                !domain.key.trim().is_empty()
                    && !domain.label.trim().is_empty()
                    && !domain.definition.trim().is_empty()
                    && !domain.measurement_target.trim().is_empty()
                    && !domain.inclusion_criteria.trim().is_empty()
                    && !domain.exclusion_criteria.trim().is_empty()
                    && !domain.separation_rule.trim().is_empty(),
                "RID domain construct metadata is incomplete at {slot}"
            );
            ensure!(domain_ids.insert(domain.id.as_str()), "duplicate RID domain ID: {}", domain.id);
            ensure!(domain_keys.insert(domain.key.as_str()), "duplicate RID domain key: {}", domain.key);
        }

        for (index, entry) in self.entries.iter().enumerate() {
            let slot = index as u8 + 1;
            let domain = &self.domains[index];
            ensure!(entry.id == slot, "RID stat slot order mismatch at {slot}");
            ensure!(entry.stat_id == format!("rid_stat:{slot:02}"), "RID stat ID mismatch at {slot}");
            ensure!(entry.domain_id == domain.id, "RID stat/domain link mismatch at {slot}");
            ensure!(entry.domain == domain.key && entry.domain_label == domain.label, "RID stat domain metadata mismatch at {slot}");
            ensure!(
                entry.measurement_target == domain.measurement_target
                    && entry.inclusion_criteria == domain.inclusion_criteria
                    && entry.exclusion_criteria == domain.exclusion_criteria
                    && entry.overlap_with == domain.overlap_with
                    && entry.separation_rule == domain.separation_rule,
                "RID stat construct-boundary metadata mismatch at {slot}"
            );
            ensure!(
                !entry.key.trim().is_empty()
                    && !entry.label.trim().is_empty()
                    && !entry.definition.trim().is_empty()
                    && !entry.evaluation_protocol.trim().is_empty()
                    && entry.initial_state == "unrated",
                "RID stat metadata or initial state is invalid at {slot}"
            );
            ensure!(stat_ids.insert(entry.stat_id.as_str()), "duplicate RID stat ID: {}", entry.stat_id);
            ensure!(stat_keys.insert(entry.key.as_str()), "duplicate RID stat key: {}", entry.key);
            for adjacent in &entry.overlap_with {
                ensure!(domain_ids.contains(adjacent.as_str()), "unknown overlap domain {adjacent} at stat {slot}");
                ensure!(adjacent != &entry.domain_id, "domain cannot overlap itself at stat {slot}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RidCatalogue;

    #[test]
    fn embedded_catalogue_has_99_independent_domains_and_stats() {
        let catalogue = RidCatalogue::embedded().unwrap();
        assert_eq!(catalogue.domains.len(), 99);
        assert_eq!(catalogue.entries.len(), 99);
        assert_eq!(catalogue.by_slot(1).unwrap().label, "SEE");
        assert_eq!(catalogue.by_slot(99).unwrap().label, "RELIABILITY");
        assert!(catalogue.by_slot(0).is_none());
        assert!(catalogue.by_slot(100).is_none());
    }

    #[test]
    fn every_domain_has_distinct_construct_boundaries() {
        let catalogue = RidCatalogue::embedded().unwrap();
        assert!(catalogue.domains.iter().all(|domain| {
            !domain.measurement_target.is_empty()
                && !domain.inclusion_criteria.is_empty()
                && !domain.exclusion_criteria.is_empty()
                && !domain.separation_rule.is_empty()
        }));
    }

    #[test]
    fn overlap_references_are_valid_and_never_self_referential() {
        let catalogue = RidCatalogue::embedded().unwrap();
        for domain in &catalogue.domains {
            assert!(!domain.overlap_with.contains(&domain.id));
            for adjacent in &domain.overlap_with {
                assert!(catalogue.domains.iter().any(|candidate| &candidate.id == adjacent));
            }
        }
    }

    #[test]
    fn safety_sensitive_domains_use_higher_sample_floor() {
        let catalogue = RidCatalogue::embedded().unwrap();
        assert_eq!(catalogue.minimum_samples_for_slot(1), Some(20));
        assert_eq!(catalogue.minimum_samples_for_slot(56), Some(30));
        assert_eq!(catalogue.minimum_samples_for_slot(66), Some(30));
        assert_eq!(catalogue.minimum_samples_for_slot(100), None);
    }
}
