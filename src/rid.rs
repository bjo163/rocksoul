use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};

pub const RID_SCHEMA_VERSION: u16 = 1;
pub const LEVEL_CURVE_VERSION: u16 = 1;
pub const STAT_COUNT: usize = 99;
pub const MAX_XP_PER_EVENT: u32 = 100;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct StatId(u8);

impl StatId {
    pub fn new(value: u8) -> Result<Self, RidError> {
        if (1..=STAT_COUNT as u8).contains(&value) {
            Ok(Self(value))
        } else {
            Err(RidError(format!("stat ID {value} is outside 1..=99")))
        }
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub fn catalogue_entry_id(self) -> String {
        format!("asmaul_husna:{:02}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuhIdentity {
    pub instance_id: String,
    /// Unix timestamp in seconds when this instance was initialized.
    pub created_at_unix: u64,
}

impl RuhIdentity {
    pub fn age_seconds_at(&self, now_unix: u64) -> u64 {
        now_unix.saturating_sub(self.created_at_unix)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XpSource {
    VerifiedTask,
    SkillEvaluation,
    ReliabilityImprovement,
    TransferEvaluation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct XpEvent {
    pub event_id: String,
    pub amount: u32,
    pub source: XpSource,
    pub occurred_at_unix: u64,
    pub evidence_ref: String,
    pub policy_version: u16,
}

impl XpEvent {
    fn validate(&self) -> Result<(), RidError> {
        if self.event_id.trim().is_empty() {
            return Err(RidError("XP event ID must not be empty".into()));
        }
        if !(1..=MAX_XP_PER_EVENT).contains(&self.amount) {
            return Err(RidError(format!(
                "XP amount must be between 1 and {MAX_XP_PER_EVENT}"
            )));
        }
        if self.occurred_at_unix == 0 || self.policy_version == 0 {
            return Err(RidError(
                "XP timestamp and policy version must be valid".into(),
            ));
        }
        if self.evidence_ref.trim().is_empty() {
            return Err(RidError("XP event requires an evidence reference".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XpRecordOutcome {
    Recorded,
    Duplicate,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct XpLedger {
    events: Vec<XpEvent>,
}

impl XpLedger {
    pub fn events(&self) -> &[XpEvent] {
        &self.events
    }

    pub fn total_xp(&self) -> u64 {
        self.events
            .iter()
            .map(|event| u64::from(event.amount))
            .fold(0, u64::saturating_add)
    }

    pub fn record(&mut self, event: XpEvent) -> Result<XpRecordOutcome, RidError> {
        event.validate()?;
        if let Some(existing) = self
            .events
            .iter()
            .find(|item| item.event_id == event.event_id)
        {
            return if existing == &event {
                Ok(XpRecordOutcome::Duplicate)
            } else {
                Err(RidError(format!(
                    "XP event ID {} was reused with different data",
                    event.event_id
                )))
            };
        }
        self.events.push(event);
        Ok(XpRecordOutcome::Recorded)
    }

    fn validate(&self) -> Result<(), RidError> {
        let mut ids = HashSet::with_capacity(self.events.len());
        for event in &self.events {
            event.validate()?;
            if !ids.insert(event.event_id.as_str()) {
                return Err(RidError("duplicate XP event ID".into()));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatRecord {
    pub id: StatId,
    pub catalogue_entry_id: String,
    pub score: Option<u8>,
    pub confidence: Option<u8>,
    pub evidence_refs: Vec<String>,
    pub last_evaluated_at_unix: Option<u64>,
    pub rubric_version: Option<String>,
}

impl StatRecord {
    fn unrated(id: StatId) -> Self {
        Self {
            id,
            catalogue_entry_id: id.catalogue_entry_id(),
            score: None,
            confidence: None,
            evidence_refs: Vec::new(),
            last_evaluated_at_unix: None,
            rubric_version: None,
        }
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence_refs.len()
    }

    pub fn record_evaluation(
        &mut self,
        score: u8,
        confidence: u8,
        evidence_refs: Vec<String>,
        evaluated_at_unix: u64,
        rubric_version: String,
    ) -> Result<(), RidError> {
        if score > 100 || confidence > 100 {
            return Err(RidError("score and confidence must be in 0..=100".into()));
        }
        if evidence_refs.is_empty()
            || evidence_refs.iter().any(|value| value.trim().is_empty())
            || evaluated_at_unix == 0
            || rubric_version.trim().is_empty()
        {
            return Err(RidError(
                "stat evaluation requires evidence, time, and rubric".into(),
            ));
        }
        self.score = Some(score);
        self.confidence = Some(confidence);
        self.evidence_refs = evidence_refs;
        self.last_evaluated_at_unix = Some(evaluated_at_unix);
        self.rubric_version = Some(rubric_version);
        Ok(())
    }

    fn validate(&self) -> Result<(), RidError> {
        let id = self.id.value();
        if !(1..=STAT_COUNT as u8).contains(&id)
            || self.catalogue_entry_id != self.id.catalogue_entry_id()
        {
            return Err(RidError(format!(
                "invalid stat or catalogue reference: {id}"
            )));
        }
        match self.score {
            None if self.confidence.is_none()
                && self.evidence_refs.is_empty()
                && self.last_evaluated_at_unix.is_none()
                && self.rubric_version.is_none() =>
            {
                Ok(())
            }
            Some(score)
                if score <= 100
                    && self.confidence.is_some_and(|value| value <= 100)
                    && !self.evidence_refs.is_empty()
                    && self
                        .evidence_refs
                        .iter()
                        .all(|value| !value.trim().is_empty())
                    && self.last_evaluated_at_unix.is_some_and(|value| value > 0)
                    && self
                        .rubric_version
                        .as_ref()
                        .is_some_and(|value| !value.trim().is_empty()) =>
            {
                Ok(())
            }
            _ => Err(RidError(format!("inconsistent measurement for stat {id}"))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuhProfile {
    schema_version: u16,
    level_curve_version: u16,
    identity: RuhIdentity,
    xp_ledger: XpLedger,
    stats: Vec<StatRecord>,
}

impl RuhProfile {
    pub fn new(identity: RuhIdentity) -> Result<Self, RidError> {
        let profile = Self {
            schema_version: RID_SCHEMA_VERSION,
            level_curve_version: LEVEL_CURVE_VERSION,
            identity,
            xp_ledger: XpLedger::default(),
            stats: (1..=STAT_COUNT as u8)
                .map(|value| StatRecord::unrated(StatId(value)))
                .collect(),
        };
        profile.validate()?;
        Ok(profile)
    }

    pub fn identity(&self) -> &RuhIdentity {
        &self.identity
    }

    pub fn age_seconds_at(&self, now_unix: u64) -> u64 {
        self.identity.age_seconds_at(now_unix)
    }

    pub fn total_xp(&self) -> u64 {
        self.xp_ledger.total_xp()
    }

    pub fn level(&self) -> u32 {
        level_for_xp(self.total_xp())
    }

    pub fn xp_events(&self) -> &[XpEvent] {
        self.xp_ledger.events()
    }

    pub fn stats(&self) -> &[StatRecord] {
        &self.stats
    }

    pub fn stat(&self, id: StatId) -> Option<&StatRecord> {
        self.stats.iter().find(|record| record.id == id)
    }

    /// Call only after a trusted kernel verifier authorizes the event and evidence.
    pub fn record_verified_xp(&mut self, event: XpEvent) -> Result<XpRecordOutcome, RidError> {
        self.xp_ledger.record(event)
    }

    /// The caller must be a trusted evaluator; models must not self-authorize scores.
    pub fn record_stat_evaluation(
        &mut self,
        id: StatId,
        score: u8,
        confidence: u8,
        evidence_refs: Vec<String>,
        evaluated_at_unix: u64,
        rubric_version: String,
    ) -> Result<(), RidError> {
        let stat = self
            .stats
            .iter_mut()
            .find(|record| record.id == id)
            .ok_or_else(|| RidError(format!("unknown stat ID {}", id.value())))?;
        stat.record_evaluation(
            score,
            confidence,
            evidence_refs,
            evaluated_at_unix,
            rubric_version,
        )
    }

    pub fn validate(&self) -> Result<(), RidError> {
        if self.schema_version != RID_SCHEMA_VERSION {
            return Err(RidError("unsupported RID schema version".into()));
        }
        if self.level_curve_version != LEVEL_CURVE_VERSION {
            return Err(RidError("unsupported RID level curve version".into()));
        }
        if self.identity.instance_id.trim().is_empty() || self.identity.created_at_unix == 0 {
            return Err(RidError("invalid RID identity".into()));
        }
        self.xp_ledger.validate()?;
        if self.stats.len() != STAT_COUNT {
            return Err(RidError(format!("RID requires exactly {STAT_COUNT} stats")));
        }
        let mut ids = HashSet::with_capacity(STAT_COUNT);
        for stat in &self.stats {
            stat.validate()?;
            if !ids.insert(stat.id) {
                return Err(RidError(format!("duplicate stat ID {}", stat.id.value())));
            }
        }
        for value in 1..=STAT_COUNT as u8 {
            if !ids.contains(&StatId(value)) {
                return Err(RidError(format!("missing stat ID {value}")));
            }
        }
        Ok(())
    }
}

/// Level N is reached at 50 * (N - 1) * N cumulative XP.
pub fn level_for_xp(total_xp: u64) -> u32 {
    let mut level = 1_u32;
    loop {
        let Some(threshold) = 50_u64
            .checked_mul(u64::from(level))
            .and_then(|value| value.checked_mul(u64::from(level) + 1))
        else {
            break;
        };
        if total_xp < threshold || level == u32::MAX {
            break;
        }
        level += 1;
    }
    level
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RidError(pub String);

impl fmt::Display for RidError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RidError {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_XP_PER_EVENT, RidError, RuhIdentity, RuhProfile, STAT_COUNT, StatId, XpEvent,
        XpRecordOutcome, XpSource, level_for_xp,
    };

    fn profile() -> RuhProfile {
        RuhProfile::new(RuhIdentity {
            instance_id: "rocksoul-test".into(),
            created_at_unix: 1_800_000_000,
        })
        .unwrap()
    }

    fn event(id: &str, amount: u32) -> XpEvent {
        XpEvent {
            event_id: id.into(),
            amount,
            source: XpSource::VerifiedTask,
            occurred_at_unix: 1_800_000_100,
            evidence_ref: "test://verified-task/1".into(),
            policy_version: 1,
        }
    }

    #[test]
    fn profile_starts_with_99_unrated_slots() {
        let profile = profile();
        assert_eq!(profile.stats().len(), STAT_COUNT);
        assert!(profile.stats().iter().all(|stat| stat.score.is_none()));
        assert_eq!(profile.stats()[0].catalogue_entry_id, "asmaul_husna:01");
        assert_eq!(profile.stats()[98].catalogue_entry_id, "asmaul_husna:99");
        profile.validate().unwrap();
    }

    #[test]
    fn stat_ids_are_bounded() {
        assert!(StatId::new(1).is_ok());
        assert!(StatId::new(99).is_ok());
        assert!(StatId::new(0).is_err());
        assert!(StatId::new(100).is_err());
    }

    #[test]
    fn age_comes_from_instance_creation_time() {
        let profile = profile();
        assert_eq!(profile.age_seconds_at(1_800_000_120), 120);
        assert_eq!(profile.age_seconds_at(1_700_000_000), 0);
    }

    #[test]
    fn xp_is_idempotent_and_level_is_derived() {
        let mut profile = profile();
        let item = event("event-1", 100);
        assert_eq!(
            profile.record_verified_xp(item.clone()).unwrap(),
            XpRecordOutcome::Recorded
        );
        assert_eq!(
            profile.record_verified_xp(item).unwrap(),
            XpRecordOutcome::Duplicate
        );
        assert_eq!(profile.total_xp(), 100);
        assert_eq!(profile.level(), 2);
    }

    #[test]
    fn conflicting_xp_event_id_is_rejected() {
        let mut profile = profile();
        profile.record_verified_xp(event("event-1", 10)).unwrap();
        assert!(profile.record_verified_xp(event("event-1", 11)).is_err());
        assert_eq!(profile.total_xp(), 10);
    }

    #[test]
    fn xp_event_amount_is_bounded() {
        let mut profile = profile();
        assert!(matches!(
            profile.record_verified_xp(event("too-large", MAX_XP_PER_EVENT + 1)),
            Err(RidError(_))
        ));
        assert_eq!(profile.total_xp(), 0);
    }

    #[test]
    fn level_curve_is_deterministic() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(99), 1);
        assert_eq!(level_for_xp(100), 2);
        assert_eq!(level_for_xp(299), 2);
        assert_eq!(level_for_xp(300), 3);
        assert_eq!(level_for_xp(600), 4);
    }

    #[test]
    fn stat_measurements_need_evidence_and_rubric() {
        let mut profile = profile();
        let id = StatId::new(1).unwrap();
        assert!(
            profile
                .record_stat_evaluation(id, 80, 90, Vec::new(), 1_800_000_200, "v1".into(),)
                .is_err()
        );

        profile
            .record_stat_evaluation(
                id,
                80,
                90,
                vec!["eval://suite/1".into()],
                1_800_000_200,
                "v1".into(),
            )
            .unwrap();
        let stat = profile.stat(id).unwrap();
        assert_eq!(stat.score, Some(80));
        assert_eq!(stat.confidence, Some(90));
        assert_eq!(stat.evidence_count(), 1);
        profile.validate().unwrap();
    }

    #[test]
    fn stat_score_and_confidence_are_bounded() {
        let mut profile = profile();
        let id = StatId::new(1).unwrap();
        let evidence = vec!["eval://suite/1".into()];
        assert!(
            profile
                .record_stat_evaluation(id, 101, 90, evidence.clone(), 1_800_000_200, "v1".into(),)
                .is_err()
        );
        assert!(
            profile
                .record_stat_evaluation(id, 80, 101, evidence, 1_800_000_200, "v1".into(),)
                .is_err()
        );
    }

    #[test]
    fn malformed_profile_fails_validation() {
        let mut profile = profile();
        profile.stats.pop();
        assert!(profile.validate().is_err());
    }
}
