use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

const EMBEDDED_CATALOGUE: &str = include_str!("../data/rid/asmaul_husna.v1.json");

#[derive(Clone, Debug, Deserialize)]
pub struct RidCatalogue {
    pub schema_version: u16,
    pub catalogue_id: String,
    pub catalogue_version: String,
    pub enumeration: EnumerationMetadata,
    pub entries: Vec<CatalogueEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EnumerationMetadata {
    pub selected_variant: String,
    pub primary_reference: String,
    pub primary_reference_url: String,
    pub primary_reference_grade_as_displayed: String,
    pub corroborating_count_report: String,
    pub corroborating_count_report_url: String,
    pub corroborating_report_grade_as_displayed: String,
    pub corroborating_report_note: String,
    pub selection_note: String,
    pub translation_note: String,
    pub review_status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogueEntry {
    pub slot_id: u8,
    pub catalogue_entry_id: String,
    pub name: CatalogueName,
    pub technical_stat: TechnicalStat,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogueName {
    pub arabic: String,
    pub transliteration: String,
    pub gloss: Gloss,
    pub gloss_status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Gloss {
    pub id: String,
    pub en: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TechnicalStat {
    pub key: String,
    pub label: String,
    pub domain: String,
    pub domain_label: String,
    pub definition: String,
    pub evaluation_protocol: String,
    pub minimum_samples: u32,
    pub score_rubric: String,
    pub status: String,
}

impl RidCatalogue {
    pub fn embedded() -> Result<Self> {
        let catalogue: Self = serde_json::from_str(EMBEDDED_CATALOGUE)
            .context("embedded RID catalogue is not valid JSON")?;
        catalogue.validate()?;
        Ok(catalogue)
    }

    pub fn by_slot(&self, slot_id: u8) -> Option<&CatalogueEntry> {
        self.entries
            .get(usize::from(slot_id.saturating_sub(1)))
            .filter(|entry| entry.slot_id == slot_id)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported RID catalogue schema");
        ensure!(
            self.catalogue_id == "rocksoul.rid.asmaul_husna_inspired.v1",
            "unexpected RID catalogue identifier"
        );
        ensure!(
            self.entries.len() == 99,
            "RID catalogue must contain exactly 99 entries"
        );
        ensure!(
            !self.catalogue_version.trim().is_empty(),
            "RID catalogue version must not be empty"
        );
        ensure!(
            !self.enumeration.primary_reference_url.trim().is_empty()
                && !self
                    .enumeration
                    .corroborating_count_report_url
                    .trim()
                    .is_empty()
                && !self.enumeration.review_status.trim().is_empty(),
            "RID catalogue source metadata is incomplete"
        );

        let mut keys = HashSet::with_capacity(99);
        for (index, entry) in self.entries.iter().enumerate() {
            let expected = index as u8 + 1;
            ensure!(
                entry.slot_id == expected,
                "RID catalogue slot order mismatch at index {}",
                index + 1
            );
            ensure!(
                entry.catalogue_entry_id == format!("asmaul_husna:{expected:02}"),
                "RID catalogue ID mismatch at slot {expected}"
            );
            ensure!(
                !entry.name.arabic.trim().is_empty()
                    && !entry.name.transliteration.trim().is_empty()
                    && !entry.name.gloss.id.trim().is_empty()
                    && !entry.name.gloss.en.trim().is_empty(),
                "RID name metadata is incomplete at slot {expected}"
            );
            ensure!(
                !entry.technical_stat.key.trim().is_empty()
                    && !entry.technical_stat.definition.trim().is_empty()
                    && !entry.technical_stat.evaluation_protocol.trim().is_empty()
                    && entry.technical_stat.minimum_samples > 0,
                "RID technical stat metadata is incomplete at slot {expected}"
            );
            ensure!(
                keys.insert(entry.technical_stat.key.as_str()),
                "duplicate RID technical stat key: {}",
                entry.technical_stat.key
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RidCatalogue;

    #[test]
    fn embedded_catalogue_has_99_ordered_unique_stat_dimensions() {
        let catalogue = RidCatalogue::embedded().unwrap();

        assert_eq!(catalogue.entries.len(), 99);
        assert_eq!(
            catalogue.by_slot(1).unwrap().technical_stat.key,
            "input_modality_detection"
        );
        assert_eq!(
            catalogue.by_slot(99).unwrap().technical_stat.key,
            "long_horizon_trustworthiness"
        );
        assert!(catalogue.by_slot(0).is_none());
        assert!(catalogue.by_slot(100).is_none());
    }

    #[test]
    fn catalogue_discloses_enumeration_grade_and_review_boundary() {
        let catalogue = RidCatalogue::embedded().unwrap();

        assert_eq!(
            catalogue.enumeration.primary_reference_grade_as_displayed,
            "Da'if (Darussalam)"
        );
        assert_eq!(
            catalogue
                .enumeration
                .corroborating_report_grade_as_displayed,
            "Sahih (Darussalam)"
        );
        assert_eq!(
            catalogue.enumeration.review_status,
            "requires_religious_and_language_review"
        );
    }

    #[test]
    fn every_slot_has_a_distinct_technical_stat_key() {
        let catalogue = RidCatalogue::embedded().unwrap();
        let mut keys = catalogue
            .entries
            .iter()
            .map(|entry| entry.technical_stat.key.as_str())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();

        assert_eq!(keys.len(), 99);
    }
}
