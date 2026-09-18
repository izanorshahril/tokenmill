use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

use serde_json::{Value, json};

use tokenmill_core::{
    ContextPackage, DeterministicPruner, MeasurementStatus, Observation, ObservationOutcome,
    PolicyFailure, PruneStatus, RouteStatus, RunPolicy, SaverReport, TransformResult, evaluate,
    validate_policy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpProcessConfig {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
}

impl AcpProcessConfig {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            working_directory: None,
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn with_working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }
}

#[derive(Debug)]
pub enum AcpTransportError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
    Json(serde_json::Error),
    UnexpectedEof,
    Protocol {
        code: i64,
        message: String,
    },
    InvalidPermissionOption(String),
    InvalidResponse(&'static str),
    UnexpectedMessage(String),
}

impl fmt::Display for AcpTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
            Self::Json(source) => write!(formatter, "invalid ACP JSON: {source}"),
            Self::UnexpectedEof => formatter.write_str("ACP process closed stdout unexpectedly"),
            Self::Protocol { code, message } => {
                write!(formatter, "ACP error {code}: {message}")
            }
            Self::InvalidPermissionOption(option_id) => {
                write!(
                    formatter,
                    "ACP permission option was not offered: {option_id}"
                )
            }
            Self::InvalidResponse(field) => write!(formatter, "ACP response missing {field}"),
            Self::UnexpectedMessage(message) => {
                write!(formatter, "unexpected ACP message: {message}")
            }
        }
    }
}

impl std::error::Error for AcpTransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json(source) => Some(source),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for AcpTransportError {
    fn from(source: serde_json::Error) -> Self {
        Self::Json(source)
    }
}

pub struct AcpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_request_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpInitializeResult {
    pub protocol_version: u64,
    pub agent_name: Option<String>,
    pub agent_version: Option<String>,
    pub auth_method_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcpPermissionRequest {
    pub request_id: Value,
    pub params: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcpPermissionDecision {
    Cancel,
    Select(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcpSessionUpdate {
    pub session_id: String,
    pub update_type: String,
    pub update: Value,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AcpUsageSummary {
    pub update_count: usize,
    pub latest_used: Option<u64>,
    pub latest_size: Option<u64>,
}

impl AcpUsageSummary {
    pub fn from_updates(updates: &[Value]) -> Self {
        let mut summary = Self {
            update_count: updates.len(),
            ..Self::default()
        };
        for update in updates {
            if let Some(used) = update.get("used").and_then(Value::as_u64) {
                summary.latest_used = Some(used);
            }
            if let Some(size) = update.get("size").and_then(Value::as_u64) {
                summary.latest_size = Some(size);
            }
        }
        summary
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcpPromptResult {
    pub text: String,
    pub updates: Vec<AcpSessionUpdate>,
    pub usage_updates: Vec<Value>,
    pub stop_reason: String,
}

impl AcpPromptResult {
    pub fn usage_summary(&self) -> AcpUsageSummary {
        AcpUsageSummary::from_updates(&self.usage_updates)
    }
}

impl AcpProcess {
    pub fn spawn(config: &AcpProcessConfig) -> Result<Self, AcpTransportError> {
        let mut command = Command::new(&config.program);
        command
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if let Some(working_directory) = &config.working_directory {
            command.current_dir(working_directory);
        }

        let mut child = command.spawn().map_err(|source| AcpTransportError::Io {
            operation: "spawn ACP process",
            source,
        })?;
        let stdin = child.stdin.take().ok_or_else(|| AcpTransportError::Io {
            operation: "open ACP stdin",
            source: io::Error::other("ACP stdin was not piped"),
        })?;
        let stdout = child.stdout.take().ok_or_else(|| AcpTransportError::Io {
            operation: "open ACP stdout",
            source: io::Error::other("ACP stdout was not piped"),
        })?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_request_id: 1,
        })
    }

    pub fn initialize(&mut self) -> Result<AcpInitializeResult, AcpTransportError> {
        let result = self.request(
            "initialize",
            json!({
                "protocolVersion": 1,
                "clientCapabilities": {},
                "clientInfo": {
                    "name": "tokenmill",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )?;

        let protocol_version = result
            .get("protocolVersion")
            .and_then(Value::as_u64)
            .ok_or(AcpTransportError::InvalidResponse("result.protocolVersion"))?;
        let agent_name = result
            .get("agentInfo")
            .and_then(|info| info.get("name"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let agent_version = result
            .get("agentInfo")
            .and_then(|info| info.get("version"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let auth_method_ids = result
            .get("authMethods")
            .and_then(Value::as_array)
            .map(|methods| {
                methods
                    .iter()
                    .filter_map(|method| method.get("id").and_then(Value::as_str))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();

        Ok(AcpInitializeResult {
            protocol_version,
            agent_name,
            agent_version,
            auth_method_ids,
        })
    }

    pub fn new_session(&mut self, cwd: &Path) -> Result<String, AcpTransportError> {
        let cwd = cwd.canonicalize().map_err(|source| AcpTransportError::Io {
            operation: "resolve ACP session cwd",
            source,
        })?;
        if !cwd.is_absolute() {
            return Err(AcpTransportError::InvalidResponse("absolute session cwd"));
        }

        let result = self.request(
            "session/new",
            json!({
                "cwd": cwd.to_string_lossy(),
                "mcpServers": []
            }),
        )?;
        result
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(AcpTransportError::InvalidResponse("result.sessionId"))
    }

    pub fn prompt(
        &mut self,
        session_id: &str,
        text: &str,
    ) -> Result<AcpPromptResult, AcpTransportError> {
        self.prompt_with_permission_handler(session_id, text, |_| AcpPermissionDecision::Cancel)
    }

    pub fn prompt_context(
        &mut self,
        session_id: &str,
        context: &ContextPackage,
    ) -> Result<AcpPromptResult, AcpTransportError> {
        self.prompt(session_id, &render_context(context))
    }

    pub fn prompt_with_permission_handler<F>(
        &mut self,
        session_id: &str,
        text: &str,
        mut permission_handler: F,
    ) -> Result<AcpPromptResult, AcpTransportError>
    where
        F: FnMut(&AcpPermissionRequest) -> AcpPermissionDecision,
    {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        prompt_turn(
            &mut self.stdin,
            &mut self.stdout,
            request_id,
            session_id,
            text,
            &mut permission_handler,
        )
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, AcpTransportError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        write_json_line(
            &mut self.stdin,
            &json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": params
            }),
        )?;

        loop {
            let message = read_json_line(&mut self.stdout)?;
            let Some(message_id) = message.get("id").and_then(Value::as_u64) else {
                continue;
            };
            if message_id != request_id {
                return Err(AcpTransportError::UnexpectedMessage(message.to_string()));
            }
            if let Some(error) = message.get("error") {
                let code = error.get("code").and_then(Value::as_i64).unwrap_or(-1);
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown ACP error")
                    .to_owned();
                return Err(AcpTransportError::Protocol { code, message });
            }
            return message
                .get("result")
                .cloned()
                .ok_or(AcpTransportError::InvalidResponse("result"));
        }
    }
}

impl Drop for AcpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn write_json_line(writer: &mut impl Write, message: &Value) -> Result<(), AcpTransportError> {
    let encoded = serde_json::to_string(message)?;
    writer
        .write_all(encoded.as_bytes())
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|source| AcpTransportError::Io {
            operation: "write ACP request",
            source,
        })
}

fn read_json_line(reader: &mut impl BufRead) -> Result<Value, AcpTransportError> {
    let mut line = String::new();
    let bytes_read = reader
        .read_line(&mut line)
        .map_err(|source| AcpTransportError::Io {
            operation: "read ACP response",
            source,
        })?;
    if bytes_read == 0 {
        return Err(AcpTransportError::UnexpectedEof);
    }
    Ok(serde_json::from_str(line.trim_end())?)
}

fn render_context(context: &ContextPackage) -> String {
    context
        .items
        .iter()
        .map(|item| format!("[{}]\n{}", item.id, item.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn prompt_turn<F>(
    writer: &mut impl Write,
    reader: &mut impl BufRead,
    request_id: u64,
    session_id: &str,
    text: &str,
    permission_handler: &mut F,
) -> Result<AcpPromptResult, AcpTransportError>
where
    F: FnMut(&AcpPermissionRequest) -> AcpPermissionDecision,
{
    write_json_line(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "session/prompt",
            "params": {
                "sessionId": session_id,
                "prompt": [{
                    "type": "text",
                    "text": text
                }]
            }
        }),
    )?;

    let mut text_output = String::new();
    let mut updates = Vec::new();
    let mut usage_updates = Vec::new();

    loop {
        let message = read_json_line(reader)?;
        if message.get("method").and_then(Value::as_str) == Some("session/update") {
            let params = message
                .get("params")
                .ok_or(AcpTransportError::InvalidResponse("params"))?;
            let update_session_id = params
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or(AcpTransportError::InvalidResponse("params.sessionId"))?;
            let update = params
                .get("update")
                .cloned()
                .ok_or(AcpTransportError::InvalidResponse("params.update"))?;
            let update_type = update
                .get("sessionUpdate")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned();

            if update_type == "agent_message_chunk"
                && update
                    .get("content")
                    .and_then(|content| content.get("type"))
                    == Some(&Value::String("text".to_owned()))
                && let Some(chunk) = update
                    .get("content")
                    .and_then(|content| content.get("text"))
                    .and_then(Value::as_str)
            {
                text_output.push_str(chunk);
            }
            if update_type == "usage_update" {
                usage_updates.push(update.clone());
            }
            updates.push(AcpSessionUpdate {
                session_id: update_session_id.to_owned(),
                update_type,
                update,
            });
            continue;
        }

        if let (Some(method), Some(incoming_id)) = (
            message.get("method").and_then(Value::as_str),
            message.get("id"),
        ) {
            if method != "session/request_permission" {
                return Err(AcpTransportError::UnexpectedMessage(message.to_string()));
            }
            let permission_request = AcpPermissionRequest {
                request_id: incoming_id.clone(),
                params: message.get("params").cloned().unwrap_or(Value::Null),
            };
            let outcome = permission_outcome(
                &permission_request.params,
                permission_handler(&permission_request),
            )?;
            write_json_line(
                writer,
                &json!({
                    "jsonrpc": "2.0",
                    "id": incoming_id,
                    "result": outcome
                }),
            )?;
            continue;
        }

        let Some(message_id) = message.get("id") else {
            continue;
        };
        if message_id != &json!(request_id) {
            return Err(AcpTransportError::UnexpectedMessage(message.to_string()));
        }
        if let Some(error) = message.get("error") {
            let code = error.get("code").and_then(Value::as_i64).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown ACP error")
                .to_owned();
            return Err(AcpTransportError::Protocol { code, message });
        }
        let result = message
            .get("result")
            .ok_or(AcpTransportError::InvalidResponse("result"))?;
        let stop_reason = result
            .get("stopReason")
            .and_then(Value::as_str)
            .ok_or(AcpTransportError::InvalidResponse("result.stopReason"))?
            .to_owned();
        return Ok(AcpPromptResult {
            text: text_output,
            updates,
            usage_updates,
            stop_reason,
        });
    }
}

fn permission_outcome(
    params: &Value,
    decision: AcpPermissionDecision,
) -> Result<Value, AcpTransportError> {
    match decision {
        AcpPermissionDecision::Cancel => Ok(json!({ "outcome": { "outcome": "cancelled" } })),
        AcpPermissionDecision::Select(option_id) => {
            let options = params
                .get("options")
                .and_then(Value::as_array)
                .ok_or(AcpTransportError::InvalidResponse("params.options"))?;
            let option_is_offered = options.iter().any(|option| {
                option
                    .get("optionId")
                    .and_then(Value::as_str)
                    .is_some_and(|offered_id| offered_id == option_id)
            });
            if !option_is_offered {
                return Err(AcpTransportError::InvalidPermissionOption(option_id));
            }
            Ok(json!({
                "outcome": { "outcome": "selected", "optionId": option_id }
            }))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpRequest {
    pub run_id: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub route_status: RouteStatus,
    pub measurement: MeasurementStatus,
    pub context: ContextPackage,
    pub task_success: Option<bool>,
}

impl AcpRequest {
    pub fn new(run_id: impl Into<String>, context: ContextPackage) -> Self {
        Self {
            run_id: run_id.into(),
            provider: None,
            model: None,
            route_status: RouteStatus::Verified,
            measurement: MeasurementStatus::Estimated,
            context,
            task_success: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpResult {
    pub transformed_context: Option<ContextPackage>,
    pub observation: Observation,
    pub failure: Option<PolicyFailure>,
}

pub struct AcpAdapter {
    policy: RunPolicy,
    pruner: DeterministicPruner,
}

impl AcpAdapter {
    pub fn new(policy: RunPolicy, max_estimated_tokens: usize) -> Self {
        Self {
            policy,
            pruner: DeterministicPruner::new(max_estimated_tokens),
        }
    }

    pub fn policy(&self) -> RunPolicy {
        self.policy
    }

    pub fn process(&self, request: AcpRequest) -> AcpResult {
        let started = Instant::now();
        let before_estimated_tokens = request.context.estimated_tokens();
        let route_status = if self.policy.routing_enabled {
            request.route_status
        } else {
            RouteStatus::Bypassed
        };

        if !self.policy.routing_enabled {
            return AcpResult {
                transformed_context: Some(request.context.clone()),
                observation: bypassed_observation(
                    &request,
                    before_estimated_tokens,
                    started.elapsed().as_millis() as u64,
                    self.policy,
                ),
                failure: None,
            };
        }

        let validation = validate_policy(self.policy, route_status, request.measurement);
        let result = match validation {
            Ok(()) => {
                let transformed = if self.policy.saver_enabled {
                    self.pruner.apply(&request.context)
                } else {
                    unchanged_result(&request.context, request.measurement)
                };
                AcpResult {
                    transformed_context: Some(transformed.package.clone()),
                    observation: completed_observation(
                        &request,
                        route_status,
                        &transformed.report,
                        started.elapsed().as_millis() as u64,
                        self.policy,
                    ),
                    failure: None,
                }
            }
            Err(failure) => AcpResult {
                transformed_context: None,
                observation: rejected_observation(
                    &request,
                    route_status,
                    before_estimated_tokens,
                    failure_message(&failure),
                    started.elapsed().as_millis() as u64,
                    self.policy,
                ),
                failure: Some(failure),
            },
        };

        result
    }
}

fn bypassed_observation(
    request: &AcpRequest,
    estimated_tokens: usize,
    latency_millis: u64,
    policy: RunPolicy,
) -> Observation {
    Observation {
        run_id: request.run_id.clone(),
        adapter: "acp".to_owned(),
        route_status: RouteStatus::Bypassed,
        provider: request.provider.clone(),
        model: request.model.clone(),
        mode: policy.mode,
        saver_enabled: policy.saver_enabled,
        routing_enabled: false,
        saver_name: None,
        before_estimated_tokens: estimated_tokens,
        after_estimated_tokens: estimated_tokens,
        measurement: request.measurement,
        latency_millis: Some(latency_millis),
        failure_reason: None,
        task_success: request.task_success,
        outcome: ObservationOutcome::Bypassed,
    }
}

fn unchanged_result(context: &ContextPackage, measurement: MeasurementStatus) -> TransformResult {
    let estimated_tokens = context.estimated_tokens();
    TransformResult {
        package: context.clone(),
        report: SaverReport {
            saver_name: "none",
            before_estimated_tokens: estimated_tokens,
            after_estimated_tokens: estimated_tokens,
            dropped_item_ids: Vec::new(),
            status: PruneStatus::Unchanged,
            measurement,
        },
    }
}

fn completed_observation(
    request: &AcpRequest,
    route_status: RouteStatus,
    report: &SaverReport,
    latency_millis: u64,
    policy: RunPolicy,
) -> Observation {
    Observation {
        run_id: request.run_id.clone(),
        adapter: "acp".to_owned(),
        route_status,
        provider: request.provider.clone(),
        model: request.model.clone(),
        mode: policy.mode,
        saver_enabled: policy.saver_enabled,
        routing_enabled: policy.routing_enabled,
        saver_name: policy.saver_enabled.then(|| report.saver_name.to_owned()),
        before_estimated_tokens: report.before_estimated_tokens,
        after_estimated_tokens: report.after_estimated_tokens,
        measurement: report.measurement,
        latency_millis: Some(latency_millis),
        failure_reason: None,
        task_success: request.task_success,
        outcome: ObservationOutcome::Completed,
    }
}

fn rejected_observation(
    request: &AcpRequest,
    route_status: RouteStatus,
    before_estimated_tokens: usize,
    failure_reason: &'static str,
    latency_millis: u64,
    policy: RunPolicy,
) -> Observation {
    Observation {
        run_id: request.run_id.clone(),
        adapter: "acp".to_owned(),
        route_status,
        provider: request.provider.clone(),
        model: request.model.clone(),
        mode: policy.mode,
        saver_enabled: policy.saver_enabled,
        routing_enabled: policy.routing_enabled,
        saver_name: None,
        before_estimated_tokens,
        after_estimated_tokens: before_estimated_tokens,
        measurement: request.measurement,
        latency_millis: Some(latency_millis),
        failure_reason: Some(failure_reason.to_owned()),
        task_success: None,
        outcome: ObservationOutcome::Rejected,
    }
}

fn failure_message(failure: &PolicyFailure) -> &'static str {
    match failure {
        PolicyFailure::RoutingDisabled => "routing-disabled",
        PolicyFailure::RouteUnverified => "route-unverified",
        PolicyFailure::MeasurementUnavailable => "measurement-unavailable",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCase {
    pub name: String,
    pub request: AcpRequest,
    pub task_success: bool,
}

impl ReplayCase {
    pub fn new(name: impl Into<String>, mut request: AcpRequest, task_success: bool) -> Self {
        request.task_success = Some(task_success);
        Self {
            name: name.into(),
            request,
            task_success,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayResult {
    pub name: String,
    pub evaluation: tokenmill_core::EvaluationResult,
    pub observation: Observation,
    pub transformed_context: ContextPackage,
}

pub struct ReplayHarness {
    adapter: AcpAdapter,
}

impl ReplayHarness {
    pub fn new(adapter: AcpAdapter) -> Self {
        Self { adapter }
    }

    pub fn run(&self, case: ReplayCase) -> ReplayResult {
        let baseline = case.request.context.clone();
        let result = self.adapter.process(case.request);
        let transformed_context = result
            .transformed_context
            .unwrap_or_else(|| baseline.clone());
        let task_success = result.observation.task_success.unwrap_or(case.task_success);
        let evaluation = evaluate(&baseline, &transformed_context, task_success);

        ReplayResult {
            name: case.name,
            evaluation,
            observation: result.observation,
            transformed_context,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use serde_json::json;

    use super::{
        AcpAdapter, AcpPermissionDecision, AcpProcess, AcpProcessConfig, AcpRequest,
        AcpTransportError, ReplayCase, ReplayHarness, prompt_turn, read_json_line, render_context,
        write_json_line,
    };
    use tokenmill_core::{
        ContextItem, ContextKind, ContextPackage, IntegrationMode, MeasurementStatus,
        ObservationOutcome, PolicyFailure, RouteStatus, RunPolicy,
    };

    fn context() -> ContextPackage {
        ContextPackage::new([
            ContextItem::new(
                "instructions",
                ContextKind::Instruction,
                "Preserve correctness and follow repository instructions.",
                true,
            ),
            ContextItem::new(
                "tool-output",
                ContextKind::ToolOutput,
                "Verbose command output that is not required for this task.",
                false,
            ),
        ])
    }

    #[test]
    fn compatible_mode_replays_unverified_routes() {
        let request = AcpRequest {
            route_status: RouteStatus::Unverified,
            provider: Some("copilot".to_owned()),
            model: Some("host-selected".to_owned()),
            ..AcpRequest::new("run-compatible", context())
        };
        let adapter = AcpAdapter::new(
            RunPolicy {
                mode: IntegrationMode::Compatible,
                ..RunPolicy::default()
            },
            20,
        );
        let result = ReplayHarness::new(adapter).run(ReplayCase::new("compatible", request, true));

        assert_eq!(result.observation.outcome, ObservationOutcome::Completed);
        assert_eq!(result.observation.route_status, RouteStatus::Unverified);
        assert!(result.evaluation.accepted());
    }

    #[test]
    fn strict_mode_rejects_unverified_routes() {
        let request = AcpRequest {
            route_status: RouteStatus::Unverified,
            ..AcpRequest::new("run-strict", context())
        };
        let result = AcpAdapter::new(RunPolicy::default(), 20).process(request);

        assert_eq!(result.failure, Some(PolicyFailure::RouteUnverified));
        assert_eq!(result.observation.outcome, ObservationOutcome::Rejected);
        assert_eq!(
            result.observation.failure_reason.as_deref(),
            Some("route-unverified")
        );
    }

    #[test]
    fn saver_off_preserves_context_and_observes_the_run() {
        let request = AcpRequest {
            measurement: MeasurementStatus::Exact,
            ..AcpRequest::new("run-off", context())
        };
        let adapter = AcpAdapter::new(
            RunPolicy {
                saver_enabled: false,
                ..RunPolicy::default()
            },
            1,
        );
        let result = adapter.process(request);

        assert!(result.failure.is_none());
        assert_eq!(result.observation.saver_name, None);
        assert_eq!(
            result.observation.before_estimated_tokens,
            result.observation.after_estimated_tokens
        );
    }

    #[test]
    fn routing_off_passes_context_through_and_marks_bypass() {
        let request = AcpRequest::new("run-routing-off", context());
        let adapter = AcpAdapter::new(
            RunPolicy {
                routing_enabled: false,
                ..RunPolicy::default()
            },
            1,
        );
        let result = adapter.process(request.clone());

        assert_eq!(result.failure, None);
        assert_eq!(result.transformed_context, Some(request.context));
        assert_eq!(result.observation.outcome, ObservationOutcome::Bypassed);
        assert_eq!(result.observation.route_status, RouteStatus::Bypassed);
    }

    #[test]
    fn stdio_codec_writes_and_reads_one_json_message_per_line() {
        let mut encoded = Vec::new();
        write_json_line(&mut encoded, &json!({"jsonrpc": "2.0", "id": 1}))
            .expect("message should encode");

        assert_eq!(encoded.iter().filter(|byte| **byte == b'\n').count(), 1);
        let decoded =
            read_json_line(&mut BufReader::new(encoded.as_slice())).expect("message should decode");
        assert_eq!(decoded["id"], 1);
        assert_eq!(decoded["jsonrpc"], "2.0");
    }

    #[test]
    fn process_spawn_reports_launch_failures() {
        let result = AcpProcess::spawn(&AcpProcessConfig::new(
            "tokenmill-command-that-does-not-exist",
        ));

        assert!(matches!(
            result,
            Err(AcpTransportError::Io {
                operation: "spawn ACP process",
                ..
            })
        ));
    }

    #[test]
    fn context_rendering_preserves_item_order_and_labels() {
        let context = ContextPackage::new([
            ContextItem::new(
                "instructions",
                ContextKind::Instruction,
                "Keep it correct.",
                true,
            ),
            ContextItem::new(
                "repository",
                ContextKind::Repository,
                "Rust workspace.",
                false,
            ),
        ]);

        assert_eq!(
            render_context(&context),
            "[instructions]\nKeep it correct.\n\n[repository]\nRust workspace."
        );
    }

    #[test]
    fn prompt_turn_collects_updates_and_answers_permission_requests() {
        let mut input = Vec::new();
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": "session-1",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {"type": "text", "text": "Hello "}
                    }
                }
            }),
        )
        .expect("update should encode");
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "id": "permission-1",
                "method": "session/request_permission",
                "params": {
                    "sessionId": "session-1",
                    "options": [{"optionId": "allow-once"}]
                }
            }),
        )
        .expect("permission request should encode");
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": "session-1",
                    "update": {
                        "sessionUpdate": "usage_update",
                        "used": 12,
                        "size": 100
                    }
                }
            }),
        )
        .expect("usage update should encode");
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": "session-1",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {"type": "text", "text": "world"}
                    }
                }
            }),
        )
        .expect("update should encode");
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "id": 7,
                "result": {"stopReason": "end_turn"}
            }),
        )
        .expect("prompt response should encode");

        let mut output = Vec::new();
        let mut reader = BufReader::new(input.as_slice());
        let mut selected_request_id = None;
        let result = prompt_turn(
            &mut output,
            &mut reader,
            7,
            "session-1",
            "Say hello",
            &mut |request| {
                selected_request_id = request.request_id.as_str().map(str::to_owned);
                AcpPermissionDecision::Select("allow-once".to_owned())
            },
        )
        .expect("prompt turn should complete");

        assert_eq!(result.text, "Hello world");
        assert_eq!(result.updates.len(), 3);
        assert_eq!(result.usage_updates.len(), 1);
        assert_eq!(result.usage_summary().update_count, 1);
        assert_eq!(result.usage_summary().latest_used, Some(12));
        assert_eq!(result.usage_summary().latest_size, Some(100));
        assert_eq!(result.stop_reason, "end_turn");
        assert_eq!(selected_request_id.as_deref(), Some("permission-1"));

        let mut output_reader = BufReader::new(output.as_slice());
        let prompt_request = read_json_line(&mut output_reader).expect("prompt should be written");
        assert_eq!(prompt_request["method"], "session/prompt");
        assert_eq!(prompt_request["params"]["sessionId"], "session-1");
        assert_eq!(prompt_request["params"]["prompt"][0]["text"], "Say hello");

        let permission_response =
            read_json_line(&mut output_reader).expect("permission response should be written");
        assert_eq!(permission_response["id"], "permission-1");
        assert_eq!(
            permission_response["result"]["outcome"]["outcome"],
            "selected"
        );
        assert_eq!(
            permission_response["result"]["outcome"]["optionId"],
            "allow-once"
        );
    }

    #[test]
    fn prompt_turn_rejects_unoffered_permission_options() {
        let mut input = Vec::new();
        write_json_line(
            &mut input,
            &json!({
                "jsonrpc": "2.0",
                "id": "permission-1",
                "method": "session/request_permission",
                "params": {
                    "sessionId": "session-1",
                    "options": [{"optionId": "allow-once"}]
                }
            }),
        )
        .expect("permission request should encode");

        let mut output = Vec::new();
        let mut reader = BufReader::new(input.as_slice());
        let result = prompt_turn(
            &mut output,
            &mut reader,
            7,
            "session-1",
            "Say hello",
            &mut |_| AcpPermissionDecision::Select("not-offered".to_owned()),
        );

        assert!(matches!(
            result,
            Err(AcpTransportError::InvalidPermissionOption(option_id))
                if option_id == "not-offered"
        ));
    }
}
