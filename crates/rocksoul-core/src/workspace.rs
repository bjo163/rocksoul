use crate::{
    brain::{Brain, BrainRequest},
    policy::{Guardian, PolicyError},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CognitiveWorkspace {
    pub goal: String,
    pub constraints: Vec<String>,
    pub facts: Vec<String>,
    pub beliefs: Vec<String>,
    pub unknowns: Vec<String>,
    pub hypotheses: Vec<String>,
    pub evidence: Vec<String>,
    pub plan: Vec<String>,
    pub actions: Vec<String>,
    pub observations: Vec<String>,
    pub risk: u8,
    pub confidence: u8,
}
pub fn bounded_think<B: Brain>(
    workspace: &mut CognitiveWorkspace,
    brain: &B,
    guardian: &mut Guardian,
) -> Result<(), PolicyError> {
    guardian.consume_step()?;
    let response = brain.infer(&BrainRequest {
        input: workspace.goal.clone(),
        max_tokens: 128,
    });
    workspace.hypotheses.push(response.proposal);
    workspace.confidence = response.confidence;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::*;
    #[test]
    fn workspace_keeps_fallback_advisory() {
        let b = DeterministicBrain {
            manifest: BrainManifest {
                name: "fallback".into(),
                role: "nano".into(),
                schema_version: 1,
                artifact_sha256: "fixture".into(),
            },
        };
        let mut w = CognitiveWorkspace {
            goal: "inspect".into(),
            ..Default::default()
        };
        bounded_think(&mut w, &b, &mut Guardian::default()).unwrap();
        assert_eq!(w.confidence, 0);
        assert!(w.facts.is_empty());
    }
}
