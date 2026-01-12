use canvas_models::{
    AskCommand, AskCommandKind, AskPlan, AskPrompt, AskRationale,
};

pub trait AskPlanner {
    fn plan(&self, prompt: &AskPrompt) -> AskPlan;
}

#[derive(Debug, Default)]
pub struct RuleBasedAskPlanner;

impl AskPlanner for RuleBasedAskPlanner {
    fn plan(&self, prompt: &AskPrompt) -> AskPlan {
        let normalized = prompt.as_str().to_lowercase();
        if normalized.contains("auth") && normalized.contains("check") {
            AskPlan::new(
                prompt.clone(),
                rationale("Detected intent to validate credentials."),
                vec![AskCommand::new(AskCommandKind::AuthCheck, Vec::new())],
            )
        } else if normalized.contains("list")
            && normalized.contains("course")
        {
            AskPlan::new(
                prompt.clone(),
                rationale("Detected intent to list courses."),
                vec![AskCommand::new(AskCommandKind::CourseList, Vec::new())],
            )
        } else {
            AskPlan::new(
                prompt.clone(),
                rationale("No matching command yet."),
                Vec::new(),
            )
        }
    }
}

fn rationale(value: &'static str) -> AskRationale {
    AskRationale::try_from(value).expect("non-empty rationale")
}
