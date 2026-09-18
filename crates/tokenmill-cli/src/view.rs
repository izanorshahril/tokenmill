use serde_json::Value;
use tokenmill_core::{
    IntegrationMode, MIN_ACCEPTED_REDUCTION_PERCENT, MeasurementStatus, ObservationOutcome,
    RouteStatus,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReportedContextUsage {
    pub update_count: usize,
    pub latest_used: Option<u64>,
    pub latest_size: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskEvidence {
    Manual,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RedactedRunView {
    pub accepted: Option<bool>,
    pub task_success: Option<bool>,
    pub task_evidence: TaskEvidence,
    pub route_status: RouteStatus,
    pub measurement: MeasurementStatus,
    pub outcome: ObservationOutcome,
    pub mode: IntegrationMode,
    pub saver_name: Option<String>,
    pub before_estimated_tokens: usize,
    pub after_estimated_tokens: usize,
    pub estimated_tokens_saved: usize,
    pub reduction_percent: f64,
    pub failure_reason: Option<String>,
    pub reported_context_usage: Option<ReportedContextUsage>,
}

impl RedactedRunView {
    pub fn from_report(report: &Value, line_number: usize) -> Result<Self, String> {
        let accepted = optional_bool(report, "accepted", line_number)?;
        let task_success = optional_bool(report, "task_success", line_number)?;
        let task_evidence = parse_task_evidence(report, task_success, line_number)?;
        let comparison = object_field(report, "comparison", line_number)?;
        let before_estimated_tokens =
            usize_field(comparison, "baseline_after_estimated_tokens", line_number)?;
        let after_estimated_tokens = usize_field(
            comparison,
            "transformed_after_estimated_tokens",
            line_number,
        )?;
        let estimated_tokens_saved =
            usize_field(comparison, "estimated_tokens_saved", line_number)?;
        let reduction_percent = comparison
            .get("reduction_percent")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("line {line_number}: invalid reduction_percent"))?;
        let variants = report
            .get("variants")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("line {line_number}: missing variants"))?;
        let transformed = variants
            .iter()
            .find(|variant| variant.get("name").and_then(Value::as_str) == Some("saver-on"))
            .ok_or_else(|| format!("line {line_number}: missing saver-on variant"))?;
        let baseline = variants
            .iter()
            .find(|variant| variant.get("name").and_then(Value::as_str) == Some("saver-off"))
            .ok_or_else(|| format!("line {line_number}: missing saver-off variant"))?;
        let baseline_observation = object_field(baseline, "observation", line_number)?;
        let baseline_route_status = parse_route_status(baseline_observation, line_number)?;
        let baseline_outcome = parse_outcome(baseline_observation, line_number)?;
        let observation = object_field(transformed, "observation", line_number)?;
        let route_status = parse_route_status(observation, line_number)?;
        let measurement = parse_measurement(observation, line_number)?;
        let outcome = parse_outcome(observation, line_number)?;
        let mode = parse_mode(observation, line_number)?;
        validate_acceptance(
            accepted,
            task_success,
            estimated_tokens_saved,
            reduction_percent,
            baseline_route_status,
            baseline_outcome,
            route_status,
            outcome,
            line_number,
        )?;

        Ok(Self {
            accepted,
            task_success,
            task_evidence,
            route_status,
            measurement,
            outcome,
            mode,
            saver_name: optional_string(observation, "saver_name", line_number)?,
            before_estimated_tokens,
            after_estimated_tokens,
            estimated_tokens_saved,
            reduction_percent,
            failure_reason: optional_string(observation, "failure_reason", line_number)?,
            reported_context_usage: parse_usage(transformed, line_number)?,
        })
    }
}

fn validate_acceptance(
    accepted: Option<bool>,
    task_success: Option<bool>,
    estimated_tokens_saved: usize,
    reduction_percent: f64,
    baseline_route_status: RouteStatus,
    baseline_outcome: ObservationOutcome,
    route_status: RouteStatus,
    outcome: ObservationOutcome,
    line_number: usize,
) -> Result<(), String> {
    let expected = task_success.map(|success| {
        success
            && estimated_tokens_saved > 0
            && reduction_percent >= MIN_ACCEPTED_REDUCTION_PERCENT
            && baseline_route_status == RouteStatus::Verified
            && baseline_outcome == ObservationOutcome::Completed
            && route_status == RouteStatus::Verified
            && outcome == ObservationOutcome::Completed
    });
    if accepted != expected {
        return Err(format!(
            "line {line_number}: accepted does not match task_success and estimated savings"
        ));
    }
    Ok(())
}

fn parse_task_evidence(
    report: &Value,
    task_success: Option<bool>,
    line_number: usize,
) -> Result<TaskEvidence, String> {
    let source = report
        .get("task_success_source")
        .and_then(Value::as_str)
        .unwrap_or(if task_success.is_some() {
            "manual"
        } else {
            "unknown"
        });
    match source {
        "manual" if task_success.is_some() => Ok(TaskEvidence::Manual),
        "unknown" if task_success.is_none() => Ok(TaskEvidence::Unknown),
        _ => Err(format!(
            "line {line_number}: task_success_source does not match task_success"
        )),
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvaluationHistorySummary {
    pub run_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub unknown_count: usize,
    pub estimated_tokens_saved: usize,
    pub average_reduction_percent: f64,
    pub latest_run: Option<RedactedRunView>,
    reduction_total: f64,
}

impl EvaluationHistorySummary {
    pub fn record(&mut self, run: RedactedRunView) {
        match run.accepted {
            Some(true) => self.accepted_count += 1,
            Some(false) => self.rejected_count += 1,
            None => self.unknown_count += 1,
        }
        self.run_count += 1;
        self.estimated_tokens_saved = self
            .estimated_tokens_saved
            .saturating_add(run.estimated_tokens_saved);
        self.reduction_total += run.reduction_percent;
        self.average_reduction_percent = self.reduction_total / self.run_count as f64;
        self.latest_run = Some(run);
    }

    pub fn acceptance_rate_percent(&self) -> f64 {
        if self.run_count == 0 {
            0.0
        } else {
            self.accepted_count as f64 / self.run_count as f64 * 100.0
        }
    }
}

fn object_field<'a>(
    value: &'a Value,
    name: &str,
    line_number: usize,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("line {line_number}: missing {name}"))
}

fn optional_bool(value: &Value, name: &str, line_number: usize) -> Result<Option<bool>, String> {
    let field = value
        .get(name)
        .ok_or_else(|| format!("line {line_number}: missing {name}"))?;
    match field.as_bool() {
        Some(value) => Ok(Some(value)),
        None if field.is_null() => Ok(None),
        None => Err(format!(
            "line {line_number}: {name} must be boolean or null"
        )),
    }
}

fn usize_field(
    value: &serde_json::Map<String, Value>,
    name: &str,
    line_number: usize,
) -> Result<usize, String> {
    let number = value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("line {line_number}: invalid {name}"))?;
    usize::try_from(number).map_err(|_| format!("line {line_number}: {name} is too large"))
}

fn optional_string(
    value: &serde_json::Map<String, Value>,
    name: &str,
    line_number: usize,
) -> Result<Option<String>, String> {
    let Some(field) = value.get(name) else {
        return Ok(None);
    };
    match field.as_str() {
        Some(value) => Ok(Some(value.to_owned())),
        None if field.is_null() => Ok(None),
        None => Err(format!(
            "line {line_number}: {name} must be a string or null"
        )),
    }
}

fn parse_usage(value: &Value, line_number: usize) -> Result<Option<ReportedContextUsage>, String> {
    let Some(usage) = value.get("usage") else {
        return Ok(None);
    };
    let usage = usage
        .as_object()
        .ok_or_else(|| format!("line {line_number}: usage must be an object"))?;
    let update_count = usize_field(usage, "update_count", line_number)?;
    let latest_used = optional_u64(usage, "latest_used", line_number)?;
    let latest_size = optional_u64(usage, "latest_size", line_number)?;
    Ok(Some(ReportedContextUsage {
        update_count,
        latest_used,
        latest_size,
    }))
}

fn optional_u64(
    value: &serde_json::Map<String, Value>,
    name: &str,
    line_number: usize,
) -> Result<Option<u64>, String> {
    let Some(field) = value.get(name) else {
        return Ok(None);
    };
    match field.as_u64() {
        Some(value) => Ok(Some(value)),
        None if field.is_null() => Ok(None),
        None => Err(format!(
            "line {line_number}: {name} must be an integer or null"
        )),
    }
}

fn parse_route_status(
    value: &serde_json::Map<String, Value>,
    line_number: usize,
) -> Result<RouteStatus, String> {
    match value.get("route_status").and_then(Value::as_str) {
        Some("verified") => Ok(RouteStatus::Verified),
        Some("unverified") => Ok(RouteStatus::Unverified),
        Some("bypassed") => Ok(RouteStatus::Bypassed),
        Some("unavailable") => Ok(RouteStatus::Unavailable),
        _ => Err(format!("line {line_number}: invalid route_status")),
    }
}

fn parse_measurement(
    value: &serde_json::Map<String, Value>,
    line_number: usize,
) -> Result<MeasurementStatus, String> {
    match value.get("measurement").and_then(Value::as_str) {
        Some("estimated") => Ok(MeasurementStatus::Estimated),
        Some("exact") => Ok(MeasurementStatus::Exact),
        Some("unmeasured") => Ok(MeasurementStatus::Unmeasured),
        _ => Err(format!("line {line_number}: invalid measurement")),
    }
}

fn parse_outcome(
    value: &serde_json::Map<String, Value>,
    line_number: usize,
) -> Result<ObservationOutcome, String> {
    match value.get("outcome").and_then(Value::as_str) {
        Some("completed") => Ok(ObservationOutcome::Completed),
        Some("bypassed") => Ok(ObservationOutcome::Bypassed),
        Some("rejected") => Ok(ObservationOutcome::Rejected),
        Some("failed") => Ok(ObservationOutcome::Failed),
        _ => Err(format!("line {line_number}: invalid outcome")),
    }
}

fn parse_mode(
    value: &serde_json::Map<String, Value>,
    line_number: usize,
) -> Result<IntegrationMode, String> {
    match value.get("mode").and_then(Value::as_str) {
        Some("strict") => Ok(IntegrationMode::Strict),
        Some("compatible") => Ok(IntegrationMode::Compatible),
        _ => Err(format!("line {line_number}: invalid mode")),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{EvaluationHistorySummary, RedactedRunView, TaskEvidence};
    use tokenmill_core::{IntegrationMode, MeasurementStatus, ObservationOutcome, RouteStatus};

    #[test]
    fn history_summary_keeps_unknown_task_evidence_and_latest_run() {
        let mut summary = EvaluationHistorySummary::default();
        summary.record(RedactedRunView {
            accepted: None,
            task_success: None,
            task_evidence: TaskEvidence::Unknown,
            route_status: RouteStatus::Unverified,
            measurement: MeasurementStatus::Estimated,
            outcome: ObservationOutcome::Completed,
            mode: IntegrationMode::Compatible,
            saver_name: Some("deterministic-pruner".to_owned()),
            before_estimated_tokens: 20,
            after_estimated_tokens: 10,
            estimated_tokens_saved: 10,
            reduction_percent: 50.0,
            failure_reason: None,
            reported_context_usage: None,
        });

        assert_eq!(summary.run_count, 1);
        assert_eq!(summary.unknown_count, 1);
        assert_eq!(summary.acceptance_rate_percent(), 0.0);
        assert_eq!(summary.latest_run.as_ref().unwrap().task_success, None);
        assert_eq!(
            summary.latest_run.as_ref().unwrap().route_status,
            RouteStatus::Unverified
        );
    }

    #[test]
    fn redacted_view_rejects_accepted_unknown_task_evidence() {
        let report = json!({
            "accepted": true,
            "task_success": null,
            "task_success_source": "unknown",
            "comparison": {
                "baseline_after_estimated_tokens": 20,
                "transformed_after_estimated_tokens": 10,
                "estimated_tokens_saved": 10,
                "reduction_percent": 50.0
            },
            "variants": [{
                "name": "saver-off",
                "observation": {
                    "route_status": "verified",
                    "measurement": "estimated",
                    "outcome": "completed",
                    "mode": "strict",
                    "saver_name": null,
                    "failure_reason": null
                }
            }, {
                "name": "saver-on",
                "observation": {
                    "route_status": "verified",
                    "measurement": "estimated",
                    "outcome": "completed",
                    "mode": "strict",
                    "saver_name": "deterministic-pruner",
                    "failure_reason": null
                }
            }]
        });

        let error = RedactedRunView::from_report(&report, 1)
            .expect_err("contradictory accepted evidence must fail");

        assert!(error.contains("accepted does not match"));
    }
}
