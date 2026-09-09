use crate::policy::{ActionMode, Guardian, PolicyError};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub mode: ActionMode,
    pub side_effects: bool,
    pub reversible: bool,
}
pub fn execute_skill(
    manifest: &SkillManifest,
    guardian: &mut Guardian,
    authorized: bool,
) -> Result<String, PolicyError> {
    guardian.consume_step()?;
    guardian.consume_tool()?;
    guardian.decide(manifest.mode, authorized)?;
    if manifest.side_effects
        && matches!(manifest.mode, ActionMode::ReadOnly | ActionMode::Simulated)
    {
        return Ok("SIMULATED".into());
    }
    Ok("ALLOWED".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simulated_skill_has_no_side_effect() {
        let m = SkillManifest {
            name: "fixture".into(),
            mode: ActionMode::Simulated,
            side_effects: true,
            reversible: true,
        };
        assert_eq!(
            execute_skill(&m, &mut Guardian::default(), false).unwrap(),
            "SIMULATED"
        );
    }
}
