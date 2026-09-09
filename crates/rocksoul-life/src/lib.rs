use rocksoul_core::{Identity, RockSoulStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeState {
    pub identity: Identity,
    pub cognitive_age: u16,
    pub level: u32,
    pub xp: u64,
    pub trust: u8,
    pub status: RockSoulStatus,
}

impl LifeState {
    #[must_use]
    pub fn born_now() -> Self {
        Self {
            identity: Identity::born_now(),
            cognitive_age: 0,
            level: 1,
            xp: 0,
            trust: 0,
            status: RockSoulStatus::Idle,
        }
    }

    pub fn set_status(&mut self, status: RockSoulStatus) {
        self.status = status;
    }

    pub fn grant_validated_xp(&mut self, amount: u64, verified: bool) -> bool {
        if !verified || amount == 0 {
            return false;
        }

        self.xp = self.xp.saturating_add(amount);
        true
    }

    pub fn promote_cognitive_age(&mut self, new_age: u16, evaluation_passed: bool) -> bool {
        if !evaluation_passed || new_age <= self.cognitive_age {
            return false;
        }

        self.cognitive_age = new_age;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn birth_state_is_level_one_without_experience() {
        let life = LifeState::born_now();

        assert_eq!(life.cognitive_age, 0);
        assert_eq!(life.level, 1);
        assert_eq!(life.xp, 0);
        assert_eq!(life.trust, 0);
        assert_eq!(life.status, RockSoulStatus::Idle);
    }

    #[test]
    fn unverified_work_does_not_earn_xp() {
        let mut life = LifeState::born_now();

        assert!(!life.grant_validated_xp(100, false));
        assert_eq!(life.xp, 0);
    }

    #[test]
    fn cognitive_age_requires_evaluation_and_forward_progress() {
        let mut life = LifeState::born_now();

        assert!(!life.promote_cognitive_age(5, false));
        assert!(life.promote_cognitive_age(5, true));
        assert!(!life.promote_cognitive_age(4, true));
        assert_eq!(life.cognitive_age, 5);
    }
}
