//! Protocol-neutral context modeling and deterministic local evaluation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextKind {
    Instruction,
    Prompt,
    Conversation,
    Repository,
    CommandOutput,
    ToolOutput,
}

impl ContextKind {
    fn pruning_priority(self) -> u8 {
        match self {
            Self::ToolOutput => 10,
            Self::CommandOutput => 20,
            Self::Repository => 30,
            Self::Conversation => 40,
            Self::Prompt => 50,
            Self::Instruction => 60,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextItem {
    pub id: String,
    pub kind: ContextKind,
    pub content: String,
    pub protected: bool,
}

impl ContextItem {
    pub fn new(
        id: impl Into<String>,
        kind: ContextKind,
        content: impl Into<String>,
        protected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            content: content.into(),
            protected,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContextPackage {
    pub items: Vec<ContextItem>,
}

impl ContextPackage {
    pub fn new(items: impl IntoIterator<Item = ContextItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }

    pub fn estimated_tokens(&self) -> usize {
        self.items
            .iter()
            .map(|item| estimate_tokens(&item.content))
            .sum()
    }
}

/// Estimates tokens without claiming provider-specific billing accuracy.
pub fn estimate_tokens(content: &str) -> usize {
    let character_estimate = (content.chars().count() + 3) / 4;
    character_estimate.max(content.split_whitespace().count())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationMode {
    Strict,
    Compatible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunPolicy {
    pub saver_enabled: bool,
    pub routing_enabled: bool,
    pub mode: IntegrationMode,
    pub require_measurement: bool,
}

impl Default for RunPolicy {
    fn default() -> Self {
        Self {
            saver_enabled: true,
            routing_enabled: true,
            mode: IntegrationMode::Strict,
            require_measurement: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteStatus {
    Verified,
    Unverified,
    Bypassed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationOutcome {
    Completed,
    Bypassed,
    Rejected,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub run_id: String,
    pub adapter: String,
    pub route_status: RouteStatus,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub mode: IntegrationMode,
    pub saver_enabled: bool,
    pub routing_enabled: bool,
    pub saver_name: Option<String>,
    pub before_estimated_tokens: usize,
    pub after_estimated_tokens: usize,
    pub measurement: MeasurementStatus,
    pub latency_millis: Option<u64>,
    pub failure_reason: Option<String>,
    pub task_success: Option<bool>,
    pub outcome: ObservationOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyFailure {
    RoutingDisabled,
    RouteUnverified,
    MeasurementUnavailable,
}

pub fn validate_policy(
    policy: RunPolicy,
    route_status: RouteStatus,
    measurement: MeasurementStatus,
) -> Result<(), PolicyFailure> {
    if !policy.routing_enabled {
        return Err(PolicyFailure::RoutingDisabled);
    }

    if policy.mode == IntegrationMode::Strict && route_status != RouteStatus::Verified {
        return Err(PolicyFailure::RouteUnverified);
    }

    if policy.mode == IntegrationMode::Strict
        && policy.require_measurement
        && measurement == MeasurementStatus::Unmeasured
    {
        return Err(PolicyFailure::MeasurementUnavailable);
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementStatus {
    Estimated,
    Exact,
    Unmeasured,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PruneStatus {
    Applied,
    Unchanged,
    BudgetUnreachable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaverReport {
    pub saver_name: &'static str,
    pub before_estimated_tokens: usize,
    pub after_estimated_tokens: usize,
    pub dropped_item_ids: Vec<String>,
    pub transformed_item_ids: Vec<String>,
    pub status: PruneStatus,
    pub measurement: MeasurementStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformResult {
    pub package: ContextPackage,
    pub report: SaverReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicPruner {
    pub max_estimated_tokens: usize,
}

impl DeterministicPruner {
    pub fn new(max_estimated_tokens: usize) -> Self {
        Self {
            max_estimated_tokens,
        }
    }

    pub fn apply(&self, input: &ContextPackage) -> TransformResult {
        let mut package = input.clone();
        let before_estimated_tokens = package.estimated_tokens();
        let mut dropped_item_ids = Vec::new();

        while package.estimated_tokens() > self.max_estimated_tokens {
            let candidate = package
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.protected)
                .min_by_key(|(_, item)| (item.kind.pruning_priority(), item.id.as_str()))
                .map(|(index, _)| index);

            let Some(candidate_index) = candidate else {
                break;
            };

            let dropped_item = package.items.remove(candidate_index);
            dropped_item_ids.push(dropped_item.id);
        }

        let after_estimated_tokens = package.estimated_tokens();
        let status = if dropped_item_ids.is_empty() {
            if before_estimated_tokens > self.max_estimated_tokens {
                PruneStatus::BudgetUnreachable
            } else {
                PruneStatus::Unchanged
            }
        } else {
            PruneStatus::Applied
        };

        TransformResult {
            package,
            report: SaverReport {
                saver_name: "deterministic-pruner",
                before_estimated_tokens,
                after_estimated_tokens,
                dropped_item_ids,
                transformed_item_ids: Vec::new(),
                status,
                measurement: MeasurementStatus::Estimated,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConsecutiveOutputCompactor;

impl ConsecutiveOutputCompactor {
    pub fn apply(&self, input: &ContextPackage) -> TransformResult {
        let mut package = input.clone();
        let before_estimated_tokens = package.estimated_tokens();
        let mut transformed_item_ids = Vec::new();

        for item in &mut package.items {
            if item.protected
                || !matches!(
                    item.kind,
                    ContextKind::CommandOutput | ContextKind::ToolOutput
                )
            {
                continue;
            }

            let compacted = compact_output(&item.content);
            if compacted != item.content {
                item.content = compacted;
                transformed_item_ids.push(item.id.clone());
            }
        }

        let after_estimated_tokens = package.estimated_tokens();
        let status = if transformed_item_ids.is_empty() {
            PruneStatus::Unchanged
        } else {
            PruneStatus::Applied
        };

        TransformResult {
            package,
            report: SaverReport {
                saver_name: "consecutive-output-compactor",
                before_estimated_tokens,
                after_estimated_tokens,
                dropped_item_ids: Vec::new(),
                transformed_item_ids,
                status,
                measurement: MeasurementStatus::Estimated,
            },
        }
    }
}

fn compact_output(content: &str) -> String {
    let mut lines = Vec::new();
    let mut previous_line = None;
    let mut previous_was_blank = false;

    for line in content.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            if previous_was_blank {
                continue;
            }
            previous_was_blank = true;
        } else {
            if !previous_was_blank && previous_line == Some(line) {
                continue;
            }
            previous_line = Some(line);
            previous_was_blank = false;
        }
        lines.push(line);
    }

    while lines.last() == Some(&"") {
        lines.pop();
    }
    let mut compacted = lines.join("\n");
    if content.ends_with('\n') && !compacted.is_empty() {
        compacted.push('\n');
    }
    compacted
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationResult {
    pub baseline_estimated_tokens: usize,
    pub transformed_estimated_tokens: usize,
    pub estimated_tokens_saved: usize,
    pub reduction_percent: f64,
    pub task_success: bool,
}

pub const MIN_ACCEPTED_REDUCTION_PERCENT: f64 = 15.0;

impl EvaluationResult {
    pub fn accepted(&self) -> bool {
        self.task_success
            && self.estimated_tokens_saved > 0
            && self.reduction_percent >= MIN_ACCEPTED_REDUCTION_PERCENT
    }
}

pub fn evaluate(
    baseline: &ContextPackage,
    transformed: &ContextPackage,
    task_success: bool,
) -> EvaluationResult {
    let baseline_estimated_tokens = baseline.estimated_tokens();
    let transformed_estimated_tokens = transformed.estimated_tokens();
    let estimated_tokens_saved =
        baseline_estimated_tokens.saturating_sub(transformed_estimated_tokens);
    let reduction_percent = if baseline_estimated_tokens == 0 {
        0.0
    } else {
        estimated_tokens_saved as f64 / baseline_estimated_tokens as f64 * 100.0
    };

    EvaluationResult {
        baseline_estimated_tokens,
        transformed_estimated_tokens,
        estimated_tokens_saved,
        reduction_percent,
        task_success,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConsecutiveOutputCompactor, ContextItem, ContextKind, ContextPackage, DeterministicPruner,
        PruneStatus, evaluate,
    };

    fn sample_package() -> ContextPackage {
        ContextPackage::new([
            ContextItem::new(
                "instructions",
                ContextKind::Instruction,
                "Follow the repository instructions and preserve correctness.",
                true,
            ),
            ContextItem::new(
                "repository",
                ContextKind::Repository,
                "src/lib.rs contains the context model and deterministic pruning pipeline.",
                false,
            ),
            ContextItem::new(
                "tool-output",
                ContextKind::ToolOutput,
                "A large command output that can be safely removed for this task.",
                false,
            ),
        ])
    }

    #[test]
    fn pruner_drops_low_priority_unprotected_items_first() {
        let input = sample_package();
        let result = DeterministicPruner::new(20).apply(&input);

        assert_eq!(result.report.status, PruneStatus::Applied);
        assert_eq!(
            result.report.dropped_item_ids,
            ["tool-output", "repository"]
        );
        assert!(
            result
                .package
                .items
                .iter()
                .any(|item| item.id == "instructions")
        );
    }

    #[test]
    fn protected_items_make_an_unreachable_budget_visible() {
        let input = ContextPackage::new([ContextItem::new(
            "instructions",
            ContextKind::Instruction,
            "These instructions are protected and cannot be removed.",
            true,
        )]);
        let result = DeterministicPruner::new(1).apply(&input);

        assert_eq!(result.report.status, PruneStatus::BudgetUnreachable);
        assert_eq!(
            result.report.before_estimated_tokens,
            result.report.after_estimated_tokens
        );
    }

    #[test]
    fn output_compactor_removes_repeated_unprotected_output_lines() {
        let input = ContextPackage::new([
            ContextItem::new(
                "instructions",
                ContextKind::Instruction,
                "Keep this instruction exactly as written.",
                true,
            ),
            ContextItem::new(
                "build-output",
                ContextKind::CommandOutput,
                "compile\ncompile\nwarning\n\n\n",
                false,
            ),
        ]);

        let result = ConsecutiveOutputCompactor.apply(&input);

        assert_eq!(result.report.saver_name, "consecutive-output-compactor");
        assert_eq!(result.report.transformed_item_ids, ["build-output"]);
        assert_eq!(result.package.items[1].content, "compile\nwarning\n");
        assert_eq!(
            result.report.measurement,
            super::MeasurementStatus::Estimated
        );
    }

    #[test]
    fn output_compactor_has_positive_paired_fixture_savings() {
        let baseline = ContextPackage::new([
            ContextItem::new(
                "instructions",
                ContextKind::Instruction,
                "Preserve the build result and report warnings.",
                true,
            ),
            ContextItem::new(
                "build-output",
                ContextKind::CommandOutput,
                "compile\ncompile\ncompile\nwarning: unused import\nwarning: unused import\n",
                false,
            ),
        ]);
        let deterministic = DeterministicPruner::new(baseline.estimated_tokens()).apply(&baseline);
        let compacted = ConsecutiveOutputCompactor.apply(&baseline);
        let deterministic_evaluation = evaluate(&baseline, &deterministic.package, true);
        let compacted_evaluation = evaluate(&baseline, &compacted.package, true);

        assert_eq!(deterministic_evaluation.estimated_tokens_saved, 0);
        assert!(compacted_evaluation.estimated_tokens_saved > 0);
        assert!(compacted_evaluation.accepted());
        assert_eq!(
            compacted.report.measurement,
            super::MeasurementStatus::Estimated
        );
    }

    #[test]
    fn evaluation_requires_savings_and_task_success() {
        let baseline = sample_package();
        let transformed = DeterministicPruner::new(20).apply(&baseline).package;
        let successful = evaluate(&baseline, &transformed, true);
        let unsuccessful = evaluate(&baseline, &transformed, false);

        assert!(successful.accepted());
        assert!(!unsuccessful.accepted());
        assert!(successful.reduction_percent > 0.0);
    }

    #[test]
    fn evaluation_rejects_reduction_below_v1_threshold() {
        let result = super::EvaluationResult {
            baseline_estimated_tokens: 100,
            transformed_estimated_tokens: 90,
            estimated_tokens_saved: 10,
            reduction_percent: 10.0,
            task_success: true,
        };

        assert!(!result.accepted());
    }
}
