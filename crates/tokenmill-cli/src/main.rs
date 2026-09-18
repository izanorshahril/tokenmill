use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use tokenmill_acp::{
    AcpAdapter, AcpProcess, AcpProcessConfig, AcpRequest, AcpUsageSummary, ReplayCase,
    ReplayHarness,
};
use tokenmill_core::{
    ContextItem, ContextKind, ContextPackage, DeterministicPruner, IntegrationMode, RouteStatus,
    RunPolicy, evaluate,
};

fn main() {
    match std::env::args().nth(1).as_deref() {
        None | Some("demo") => run_demo(),
        Some("replay") => run_replay(),
        Some("acp-check") => run_acp_check(std::env::args().skip(2), false),
        Some("acp-session-check") => run_acp_check(std::env::args().skip(2), true),
        Some("acp-prompt") => run_acp_prompt(std::env::args().skip(2)),
        Some("acp-context-prompt") => run_acp_context_prompt(std::env::args().skip(2)),
        Some("acp-paired-context-prompt") => {
            run_acp_paired_context_prompt(std::env::args().skip(2))
        }
        Some("eval-history") => run_evaluation_history(std::env::args().skip(2)),
        Some("help") | Some("--help") | Some("-h") => print_help(),
        Some(command) => {
            eprintln!("Unknown command: {command}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn run_acp_prompt(mut arguments: impl Iterator<Item = String>) {
    let Some(program) = arguments.next() else {
        eprintln!("acp-prompt requires an ACP agent executable path");
        print_help();
        std::process::exit(2);
    };
    let Some(cwd) = arguments.next().map(PathBuf::from) else {
        eprintln!("acp-prompt requires a workspace path");
        print_help();
        std::process::exit(2);
    };
    let prompt = arguments.collect::<Vec<_>>().join(" ");
    if prompt.is_empty() {
        eprintln!("acp-prompt requires prompt text");
        print_help();
        std::process::exit(2);
    }

    let config = AcpProcessConfig::new(program.clone())
        .with_arg("--acp")
        .with_working_directory(cwd.clone());
    let mut process = match AcpProcess::spawn(&config) {
        Ok(process) => process,
        Err(error) => {
            eprintln!("ACP prompt failed to start: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = process.initialize() {
        eprintln!("ACP initialize failed: {error}");
        std::process::exit(1);
    }
    let session_id = match process.new_session(&cwd) {
        Ok(session_id) => session_id,
        Err(error) => {
            eprintln!("ACP session/new failed: {error}");
            std::process::exit(1);
        }
    };
    let result = match process.prompt(&session_id, &prompt) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("ACP session/prompt failed: {error}");
            std::process::exit(1);
        }
    };

    print!("{}", result.text);
    if !result.text.ends_with('\n') {
        println!();
    }
    println!("stop reason: {}", result.stop_reason);
    println!("updates: {}", result.updates.len());
    print_usage_summary(&result.usage_summary());
}

fn run_acp_context_prompt(mut arguments: impl Iterator<Item = String>) {
    let Some(program) = arguments.next() else {
        eprintln!("acp-context-prompt requires an ACP agent executable path");
        print_help();
        std::process::exit(2);
    };
    let Some(cwd) = arguments.next().map(PathBuf::from) else {
        eprintln!("acp-context-prompt requires a workspace path");
        print_help();
        std::process::exit(2);
    };
    let Some(context_path) = arguments.next().map(PathBuf::from) else {
        eprintln!("acp-context-prompt requires a context JSON path");
        print_help();
        std::process::exit(2);
    };
    let Some(max_estimated_tokens) = arguments.next() else {
        eprintln!("acp-context-prompt requires a maximum estimated token count");
        print_help();
        std::process::exit(2);
    };
    let options = match parse_context_prompt_options(arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("acp-context-prompt options failed: {error}");
            std::process::exit(2);
        }
    };
    let max_estimated_tokens = match max_estimated_tokens.parse::<usize>() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid maximum estimated token count: {error}");
            std::process::exit(2);
        }
    };
    let context = match read_context_package(&context_path) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("context JSON failed: {error}");
            std::process::exit(1);
        }
    };
    let config = AcpProcessConfig::new(program.clone())
        .with_arg("--acp")
        .with_working_directory(cwd.clone());
    let mut process = match AcpProcess::spawn(&config) {
        Ok(process) => process,
        Err(error) => {
            eprintln!("ACP context prompt failed to start: {error}");
            std::process::exit(1);
        }
    };
    let initialization = match process.initialize() {
        Ok(initialization) => initialization,
        Err(error) => {
            eprintln!("ACP initialize failed: {error}");
            std::process::exit(1);
        }
    };
    let mut request = AcpRequest::new("live-context", context);
    request.provider = initialization.agent_name.clone();
    request.route_status =
        route_status_for_agent(Path::new(&program), initialization.agent_name.as_deref());
    let transformed = AcpAdapter::new(options.policy, max_estimated_tokens).process(request);
    let Some(transformed_context) = transformed.transformed_context else {
        eprintln!("context saver rejected the run: {:?}", transformed.failure);
        std::process::exit(1);
    };

    let session_id = match process.new_session(&cwd) {
        Ok(session_id) => session_id,
        Err(error) => {
            eprintln!("ACP session/new failed: {error}");
            std::process::exit(1);
        }
    };
    let result = match process.prompt_context(&session_id, &transformed_context) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("ACP session/prompt failed: {error}");
            std::process::exit(1);
        }
    };

    println!(
        "estimated tokens: {} -> {}",
        transformed.observation.before_estimated_tokens,
        transformed.observation.after_estimated_tokens
    );
    print!("{}", result.text);
    if !result.text.ends_with('\n') {
        println!();
    }
    println!("stop reason: {}", result.stop_reason);
    println!("updates: {}", result.updates.len());
    let usage_summary = result.usage_summary();
    print_usage_summary(&usage_summary);
    if let Some(report_path) = options.report_path {
        if let Err(error) =
            write_observation_report(&report_path, &transformed.observation, &usage_summary)
        {
            eprintln!("observation report failed: {error}");
            std::process::exit(1);
        }
        println!("observation report: {}", report_path.display());
    }
}

fn run_acp_paired_context_prompt(mut arguments: impl Iterator<Item = String>) {
    let Some(program) = arguments.next() else {
        eprintln!("acp-paired-context-prompt requires an ACP agent executable path");
        print_help();
        std::process::exit(2);
    };
    let Some(cwd) = arguments.next().map(PathBuf::from) else {
        eprintln!("acp-paired-context-prompt requires a workspace path");
        print_help();
        std::process::exit(2);
    };
    let Some(context_path) = arguments.next().map(PathBuf::from) else {
        eprintln!("acp-paired-context-prompt requires a context JSON path");
        print_help();
        std::process::exit(2);
    };
    let Some(max_estimated_tokens) = arguments.next() else {
        eprintln!("acp-paired-context-prompt requires a maximum estimated token count");
        print_help();
        std::process::exit(2);
    };
    let options = match parse_paired_context_options(arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("acp-paired-context-prompt options failed: {error}");
            std::process::exit(2);
        }
    };
    let max_estimated_tokens = match max_estimated_tokens.parse::<usize>() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("invalid maximum estimated token count: {error}");
            std::process::exit(2);
        }
    };
    let context = match read_context_package(&context_path) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("context JSON failed: {error}");
            std::process::exit(1);
        }
    };
    let config = AcpProcessConfig::new(program.clone())
        .with_arg("--acp")
        .with_working_directory(cwd.clone());
    let mut process = match AcpProcess::spawn(&config) {
        Ok(process) => process,
        Err(error) => {
            eprintln!("ACP paired prompt failed to start: {error}");
            std::process::exit(1);
        }
    };
    let initialization = match process.initialize() {
        Ok(initialization) => initialization,
        Err(error) => {
            eprintln!("ACP initialize failed: {error}");
            std::process::exit(1);
        }
    };
    let provider = initialization.agent_name.clone();
    let route_status =
        route_status_for_agent(Path::new(&program), initialization.agent_name.as_deref());
    let saver_off = match run_live_context_variant(
        &mut process,
        &cwd,
        &context,
        RunPolicy {
            saver_enabled: false,
            mode: options.mode,
            ..RunPolicy::default()
        },
        route_status,
        provider.clone(),
        max_estimated_tokens,
        "live-paired-saver-off",
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("saver-off live evaluation failed: {error}");
            std::process::exit(1);
        }
    };
    let saver_on = match run_live_context_variant(
        &mut process,
        &cwd,
        &context,
        RunPolicy {
            mode: options.mode,
            ..RunPolicy::default()
        },
        route_status,
        provider,
        max_estimated_tokens,
        "live-paired-saver-on",
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("saver-on live evaluation failed: {error}");
            std::process::exit(1);
        }
    };

    let estimated_tokens_saved = saver_off
        .observation
        .after_estimated_tokens
        .saturating_sub(saver_on.observation.after_estimated_tokens);
    let reduction_percent = if saver_off.observation.after_estimated_tokens == 0 {
        0.0
    } else {
        estimated_tokens_saved as f64 / saver_off.observation.after_estimated_tokens as f64 * 100.0
    };
    let accepted = options
        .task_success
        .map(|task_success| task_success && estimated_tokens_saved > 0);
    println!("Tokenmill paired live context evaluation");
    println!("task success: {}", task_success_label(options.task_success));
    println!(
        "saver-off estimated tokens: {} -> {}",
        saver_off.observation.before_estimated_tokens, saver_off.observation.after_estimated_tokens
    );
    println!(
        "saver-on estimated tokens: {} -> {}",
        saver_on.observation.before_estimated_tokens, saver_on.observation.after_estimated_tokens
    );
    println!("estimated tokens saved: {estimated_tokens_saved}");
    println!("reduction versus saver-off: {reduction_percent:.1}%");
    println!("accepted: {}", accepted_label(accepted));
    print_live_variant_summary(&saver_off);
    print_live_variant_summary(&saver_on);

    if let Some(report_path) = options.report_path {
        if let Err(error) = write_paired_observation_report(
            &report_path,
            &saver_off,
            &saver_on,
            options.task_success,
        ) {
            eprintln!("paired observation report failed: {error}");
            std::process::exit(1);
        }
        println!("paired observation report: {}", report_path.display());
    }
    if let Some(history_path) = options.history_path {
        if let Err(error) = append_paired_observation_report(
            &history_path,
            &saver_off,
            &saver_on,
            options.task_success,
        ) {
            eprintln!("evaluation history failed: {error}");
            std::process::exit(1);
        }
        println!("evaluation history: {}", history_path.display());
    }
}

fn run_evaluation_history(mut arguments: impl Iterator<Item = String>) {
    let Some(path) = arguments.next().map(PathBuf::from) else {
        eprintln!("eval-history requires a history JSONL path");
        print_help();
        std::process::exit(2);
    };
    if let Some(unexpected) = arguments.next() {
        eprintln!("eval-history received unexpected argument: {unexpected}");
        print_help();
        std::process::exit(2);
    }
    let summary = match summarize_evaluation_history(&path) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("evaluation history failed: {error}");
            std::process::exit(1);
        }
    };
    println!("Tokenmill evaluation history");
    println!("runs: {}", summary.run_count);
    println!("accepted: {}", summary.accepted_count);
    println!("rejected: {}", summary.rejected_count);
    println!("unknown: {}", summary.unknown_count);
    println!("estimated tokens saved: {}", summary.estimated_tokens_saved);
    println!(
        "average reduction: {:.1}%",
        summary.average_reduction_percent
    );
}

fn run_live_context_variant(
    process: &mut AcpProcess,
    cwd: &Path,
    context: &ContextPackage,
    policy: RunPolicy,
    route_status: RouteStatus,
    provider: Option<String>,
    max_estimated_tokens: usize,
    run_id: &str,
) -> Result<LiveVariant, String> {
    let mut request = AcpRequest::new(run_id, context.clone());
    request.provider = provider;
    request.route_status = route_status;
    let transformed = AcpAdapter::new(policy, max_estimated_tokens).process(request);
    let transformed_context = transformed
        .transformed_context
        .ok_or_else(|| format!("context saver rejected the run: {:?}", transformed.failure))?;
    let session_id = process
        .new_session(cwd)
        .map_err(|error| error.to_string())?;
    let result = process
        .prompt_context(&session_id, &transformed_context)
        .map_err(|error| error.to_string())?;
    let usage = result.usage_summary();
    Ok(LiveVariant {
        name: if policy.saver_enabled {
            "saver-on"
        } else {
            "saver-off"
        },
        observation: transformed.observation,
        stop_reason: result.stop_reason,
        update_count: result.updates.len(),
        usage,
    })
}

fn print_live_variant_summary(variant: &LiveVariant) {
    println!(
        "{}: route={}, outcome={}, stop_reason={}, updates={}",
        variant.name,
        route_status_label(variant.observation.route_status),
        observation_outcome_label(variant.observation.outcome),
        variant.stop_reason,
        variant.update_count
    );
    print_usage_summary(&variant.usage);
}

fn read_context_package(path: &Path) -> Result<ContextPackage, String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_context_package(&source)
}

fn parse_context_package(source: &str) -> Result<ContextPackage, String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let document: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let items = document
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| document.as_array())
        .ok_or_else(|| "expected an object with an items array".to_owned())?;
    let mut context_items = Vec::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        let item = item
            .as_object()
            .ok_or_else(|| format!("items[{index}] must be an object"))?;
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("items[{index}].id must be a string"))?;
        let kind_name = item
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("items[{index}].kind must be a string"))?
            .to_ascii_lowercase();
        let kind = parse_context_kind(&kind_name)
            .ok_or_else(|| format!("items[{index}].kind is unsupported: {kind_name}"))?;
        let content = item
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("items[{index}].content must be a string"))?;
        let protected = item
            .get("protected")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("items[{index}].protected must be a boolean"))?;
        context_items.push(ContextItem::new(id, kind, content, protected));
    }

    Ok(ContextPackage::new(context_items))
}

fn parse_context_kind(kind_name: &str) -> Option<ContextKind> {
    match kind_name {
        "instruction" => Some(ContextKind::Instruction),
        "prompt" => Some(ContextKind::Prompt),
        "conversation" => Some(ContextKind::Conversation),
        "repository" => Some(ContextKind::Repository),
        "command-output" => Some(ContextKind::CommandOutput),
        "tool-output" => Some(ContextKind::ToolOutput),
        _ => None,
    }
}

fn route_status_for_agent(program: &Path, agent_name: Option<&str>) -> RouteStatus {
    let explicit_github_identity = agent_name.is_some_and(|name| {
        matches!(
            name.trim().to_ascii_lowercase().as_str(),
            "github copilot" | "github copilot cli"
        )
    });
    let generic_copilot_identity = agent_name.is_some_and(|name| {
        name.trim().eq_ignore_ascii_case("copilot") && github_copilot_path_marker(program)
    });
    if explicit_github_identity || generic_copilot_identity {
        RouteStatus::Verified
    } else {
        RouteStatus::Unverified
    }
}

fn github_copilot_path_marker(program: &Path) -> bool {
    let normalized = program
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    [
        "/github-copilot-sdk/",
        "/github cli/copilot/",
        "/node_modules/@github/copilot/",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

#[derive(Debug, Eq, PartialEq)]
struct ContextPromptOptions {
    policy: RunPolicy,
    report_path: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
struct PairedContextOptions {
    mode: IntegrationMode,
    report_path: Option<PathBuf>,
    history_path: Option<PathBuf>,
    task_success: Option<bool>,
}

#[derive(Debug, PartialEq)]
struct EvaluationHistorySummary {
    run_count: usize,
    accepted_count: usize,
    rejected_count: usize,
    unknown_count: usize,
    estimated_tokens_saved: usize,
    average_reduction_percent: f64,
}

struct LiveVariant {
    name: &'static str,
    observation: tokenmill_core::Observation,
    stop_reason: String,
    update_count: usize,
    usage: AcpUsageSummary,
}

fn parse_context_prompt_options(
    arguments: impl Iterator<Item = String>,
) -> Result<ContextPromptOptions, String> {
    let arguments = arguments.collect::<Vec<_>>();
    let mut policy = RunPolicy::default();
    let mut report_path = None;
    let mut index = 0;

    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("{option} requires a value"))?;
        match option {
            "--saver" => policy.saver_enabled = parse_toggle(option, value)?,
            "--routing" => policy.routing_enabled = parse_toggle(option, value)?,
            "--mode" => {
                policy.mode = match value.as_str() {
                    "strict" => IntegrationMode::Strict,
                    "compatible" => IntegrationMode::Compatible,
                    _ => {
                        return Err(format!(
                            "{option} must be strict or compatible, got {value}"
                        ));
                    }
                };
            }
            "--report" => report_path = Some(PathBuf::from(value)),
            _ => return Err(format!("unexpected argument: {option}")),
        }
        index += 2;
    }

    Ok(ContextPromptOptions {
        policy,
        report_path,
    })
}

fn parse_paired_context_options(
    arguments: impl Iterator<Item = String>,
) -> Result<PairedContextOptions, String> {
    let arguments = arguments.collect::<Vec<_>>();
    let mut mode = IntegrationMode::Strict;
    let mut report_path = None;
    let mut history_path = None;
    let mut task_success = None;
    let mut index = 0;

    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| format!("{option} requires a value"))?;
        match option {
            "--mode" => {
                mode = match value.as_str() {
                    "strict" => IntegrationMode::Strict,
                    "compatible" => IntegrationMode::Compatible,
                    _ => {
                        return Err(format!(
                            "{option} must be strict or compatible, got {value}"
                        ));
                    }
                };
            }
            "--report" => report_path = Some(PathBuf::from(value)),
            "--history" => history_path = Some(PathBuf::from(value)),
            "--task-success" => {
                task_success = match value.as_str() {
                    "pass" => Some(true),
                    "fail" => Some(false),
                    "unknown" => None,
                    _ => {
                        return Err(format!(
                            "{option} must be pass, fail, or unknown, got {value}"
                        ));
                    }
                };
            }
            _ => return Err(format!("unexpected argument: {option}")),
        }
        index += 2;
    }

    Ok(PairedContextOptions {
        mode,
        report_path,
        history_path,
        task_success,
    })
}

fn parse_toggle(option: &str, value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err(format!("{option} must be on or off, got {value}")),
    }
}

fn print_usage_summary(summary: &AcpUsageSummary) {
    println!("usage updates: {}", summary.update_count);
    match (summary.latest_used, summary.latest_size) {
        (Some(used), Some(size)) => println!("reported context usage: {used}/{size}"),
        (Some(used), None) => println!("reported context usage: {used}/unknown"),
        (None, Some(size)) => println!("reported context usage: unknown/{size}"),
        (None, None) => {}
    }
}

fn task_success_label(task_success: Option<bool>) -> &'static str {
    match task_success {
        Some(true) => "pass",
        Some(false) => "fail",
        None => "unknown",
    }
}

fn accepted_label(accepted: Option<bool>) -> &'static str {
    match accepted {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn write_observation_report(
    path: &Path,
    observation: &tokenmill_core::Observation,
    usage: &AcpUsageSummary,
) -> Result<(), String> {
    let report = json!({
        "schema_version": 1,
        "created_at_unix_seconds": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        "observation": {
            "run_id": observation.run_id,
            "adapter": observation.adapter,
            "route_status": route_status_label(observation.route_status),
            "provider": observation.provider,
            "model": observation.model,
            "mode": integration_mode_label(observation.mode),
            "saver_enabled": observation.saver_enabled,
            "routing_enabled": observation.routing_enabled,
            "saver_name": observation.saver_name,
            "before_estimated_tokens": observation.before_estimated_tokens,
            "after_estimated_tokens": observation.after_estimated_tokens,
            "measurement": measurement_status_label(observation.measurement),
            "latency_millis": observation.latency_millis,
            "failure_reason": observation.failure_reason,
            "task_success": observation.task_success,
            "outcome": observation_outcome_label(observation.outcome),
        },
        "usage_update_count": usage.update_count,
        "usage": {
            "latest_used": usage.latest_used,
            "latest_size": usage.latest_size,
        },
    });
    let mut encoded = serde_json::to_string(&report).map_err(|error| error.to_string())?;
    encoded.push('\n');
    fs::write(path, encoded).map_err(|error| error.to_string())
}

fn write_paired_observation_report(
    path: &Path,
    saver_off: &LiveVariant,
    saver_on: &LiveVariant,
    task_success: Option<bool>,
) -> Result<(), String> {
    let report = paired_observation_report(saver_off, saver_on, task_success)?;
    let mut encoded = serde_json::to_string(&report).map_err(|error| error.to_string())?;
    encoded.push('\n');
    fs::write(path, encoded).map_err(|error| error.to_string())
}

fn append_paired_observation_report(
    path: &Path,
    saver_off: &LiveVariant,
    saver_on: &LiveVariant,
    task_success: Option<bool>,
) -> Result<(), String> {
    let report = paired_observation_report(saver_off, saver_on, task_success)?;
    let mut encoded = serde_json::to_string(&report).map_err(|error| error.to_string())?;
    encoded.push('\n');
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(encoded.as_bytes())
        .map_err(|error| error.to_string())
}

fn paired_observation_report(
    saver_off: &LiveVariant,
    saver_on: &LiveVariant,
    task_success: Option<bool>,
) -> Result<Value, String> {
    let baseline_tokens = saver_off.observation.after_estimated_tokens;
    let transformed_tokens = saver_on.observation.after_estimated_tokens;
    let estimated_tokens_saved = baseline_tokens.saturating_sub(transformed_tokens);
    let reduction_percent = if baseline_tokens == 0 {
        0.0
    } else {
        estimated_tokens_saved as f64 / baseline_tokens as f64 * 100.0
    };
    let accepted = task_success.map(|success| success && estimated_tokens_saved > 0);
    let report = json!({
        "schema_version": 1,
        "report_type": "paired_live_evaluation",
        "created_at_unix_seconds": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        "task_success": task_success,
        "accepted": accepted,
        "comparison": {
            "baseline": "saver-off",
            "transformed": "saver-on",
            "baseline_after_estimated_tokens": baseline_tokens,
            "transformed_after_estimated_tokens": transformed_tokens,
            "estimated_tokens_saved": estimated_tokens_saved,
            "reduction_percent": reduction_percent,
        },
        "variants": [
            live_variant_json(saver_off),
            live_variant_json(saver_on),
        ],
    });
    Ok(report)
}

fn summarize_evaluation_history(path: &Path) -> Result<EvaluationHistorySummary, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut summary = EvaluationHistorySummary {
        run_count: 0,
        accepted_count: 0,
        rejected_count: 0,
        unknown_count: 0,
        estimated_tokens_saved: 0,
        average_reduction_percent: 0.0,
    };
    let mut reduction_total = 0.0;

    for (line_index, line) in reader.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.map_err(|error| format!("line {line_number}: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let report: Value = serde_json::from_str(&line)
            .map_err(|error| format!("line {line_number}: invalid JSON: {error}"))?;
        if report.get("schema_version").and_then(Value::as_u64) != Some(1) {
            return Err(format!("line {line_number}: unsupported schema_version"));
        }
        if report.get("report_type").and_then(Value::as_str) != Some("paired_live_evaluation") {
            return Err(format!("line {line_number}: not a paired live evaluation"));
        }

        let accepted = report
            .get("accepted")
            .ok_or_else(|| format!("line {line_number}: missing accepted"))?;
        match accepted.as_bool() {
            Some(true) => summary.accepted_count += 1,
            Some(false) => summary.rejected_count += 1,
            None if accepted.is_null() => summary.unknown_count += 1,
            None => {
                return Err(format!(
                    "line {line_number}: accepted must be boolean or null"
                ));
            }
        }

        let comparison = report
            .get("comparison")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("line {line_number}: missing comparison"))?;
        let saved = comparison
            .get("estimated_tokens_saved")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("line {line_number}: invalid estimated_tokens_saved"))?;
        summary.estimated_tokens_saved = summary
            .estimated_tokens_saved
            .checked_add(
                usize::try_from(saved)
                    .map_err(|_| format!("line {line_number}: token count is too large"))?,
            )
            .ok_or_else(|| format!("line {line_number}: token total is too large"))?;

        let reduction = comparison
            .get("reduction_percent")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("line {line_number}: invalid reduction_percent"))?;
        reduction_total += reduction;
        summary.run_count += 1;
    }

    if summary.run_count > 0 {
        summary.average_reduction_percent = reduction_total / summary.run_count as f64;
    }
    Ok(summary)
}

fn live_variant_json(variant: &LiveVariant) -> Value {
    json!({
        "name": variant.name,
        "observation": {
            "run_id": variant.observation.run_id,
            "adapter": variant.observation.adapter,
            "route_status": route_status_label(variant.observation.route_status),
            "provider": variant.observation.provider,
            "model": variant.observation.model,
            "mode": integration_mode_label(variant.observation.mode),
            "saver_enabled": variant.observation.saver_enabled,
            "routing_enabled": variant.observation.routing_enabled,
            "saver_name": variant.observation.saver_name,
            "before_estimated_tokens": variant.observation.before_estimated_tokens,
            "after_estimated_tokens": variant.observation.after_estimated_tokens,
            "measurement": measurement_status_label(variant.observation.measurement),
            "latency_millis": variant.observation.latency_millis,
            "failure_reason": variant.observation.failure_reason,
            "task_success": variant.observation.task_success,
            "outcome": observation_outcome_label(variant.observation.outcome),
        },
        "stop_reason": variant.stop_reason,
        "update_count": variant.update_count,
        "usage": {
            "update_count": variant.usage.update_count,
            "latest_used": variant.usage.latest_used,
            "latest_size": variant.usage.latest_size,
        },
    })
}

fn route_status_label(status: RouteStatus) -> &'static str {
    match status {
        RouteStatus::Verified => "verified",
        RouteStatus::Unverified => "unverified",
        RouteStatus::Bypassed => "bypassed",
        RouteStatus::Unavailable => "unavailable",
    }
}

fn integration_mode_label(mode: IntegrationMode) -> &'static str {
    match mode {
        IntegrationMode::Strict => "strict",
        IntegrationMode::Compatible => "compatible",
    }
}

fn measurement_status_label(status: tokenmill_core::MeasurementStatus) -> &'static str {
    match status {
        tokenmill_core::MeasurementStatus::Estimated => "estimated",
        tokenmill_core::MeasurementStatus::Exact => "exact",
        tokenmill_core::MeasurementStatus::Unmeasured => "unmeasured",
    }
}

fn observation_outcome_label(outcome: tokenmill_core::ObservationOutcome) -> &'static str {
    match outcome {
        tokenmill_core::ObservationOutcome::Completed => "completed",
        tokenmill_core::ObservationOutcome::Bypassed => "bypassed",
        tokenmill_core::ObservationOutcome::Rejected => "rejected",
        tokenmill_core::ObservationOutcome::Failed => "failed",
    }
}

fn run_acp_check(mut arguments: impl Iterator<Item = String>, create_session: bool) {
    let Some(program) = arguments.next() else {
        eprintln!("acp-check requires an ACP agent executable path");
        print_help();
        std::process::exit(2);
    };
    let cwd = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("current directory should resolve"));
    let mut config = AcpProcessConfig::new(program.clone()).with_arg("--acp");
    for argument in arguments {
        config = config.with_arg(argument);
    }

    let mut process = match AcpProcess::spawn(&config) {
        Ok(process) => process,
        Err(error) => {
            eprintln!("ACP check failed: {error}");
            std::process::exit(1);
        }
    };
    let initialization = match process.initialize() {
        Ok(initialization) => initialization,
        Err(error) => {
            eprintln!("ACP initialize failed: {error}");
            std::process::exit(1);
        }
    };
    println!("ACP check succeeded");
    println!("protocol version: {}", initialization.protocol_version);
    println!("agent: {:?}", initialization.agent_name);
    println!("agent version: {:?}", initialization.agent_version);
    println!(
        "GitHub Copilot identity: {}",
        route_status_label(route_status_for_agent(
            Path::new(&program),
            initialization.agent_name.as_deref()
        ))
    );
    println!("auth methods: {:?}", initialization.auth_method_ids);
    if create_session {
        let session_id = match process.new_session(&cwd) {
            Ok(session_id) => session_id,
            Err(error) => {
                eprintln!("ACP session/new failed: {error}");
                std::process::exit(1);
            }
        };
        println!("session id: {session_id}");
    }
}

fn replay_context() -> ContextPackage {
    ContextPackage::new([
        ContextItem::new(
            "instructions",
            ContextKind::Instruction,
            "Follow repository instructions and preserve correctness.",
            true,
        ),
        ContextItem::new(
            "prompt",
            ContextKind::Prompt,
            "Explain how to reduce the context sent for this task.",
            false,
        ),
        ContextItem::new(
            "repository",
            ContextKind::Repository,
            "The repository contains a Rust core and an ACP adapter for local evaluation.",
            false,
        ),
        ContextItem::new(
            "tool-output",
            ContextKind::ToolOutput,
            "Verbose command output that is not needed for this particular task.",
            false,
        ),
    ])
}

fn run_replay() {
    let mut request = AcpRequest::new("replay-compatible", replay_context());
    request.route_status = RouteStatus::Unverified;
    request.provider = Some("copilot".to_owned());
    request.model = Some("host-selected".to_owned());

    let compatible = ReplayHarness::new(AcpAdapter::new(
        RunPolicy {
            mode: IntegrationMode::Compatible,
            ..RunPolicy::default()
        },
        35,
    ))
    .run(ReplayCase::new("compatible-route", request.clone(), true));
    let strict = AcpAdapter::new(RunPolicy::default(), 35).process(request);

    println!("Tokenmill ACP replay harness");
    println!("compatible outcome: {:?}", compatible.observation.outcome);
    println!(
        "compatible route: {:?}",
        compatible.observation.route_status
    );
    println!(
        "compatible reduction: {:.1}%",
        compatible.evaluation.reduction_percent
    );
    println!("compatible accepted: {}", compatible.evaluation.accepted());
    println!("strict outcome: {:?}", strict.observation.outcome);
    println!("strict failure: {:?}", strict.failure);
}

fn run_demo() {
    let baseline = ContextPackage::new([
        ContextItem::new(
            "instructions",
            ContextKind::Instruction,
            "Follow repository instructions and preserve correctness.",
            true,
        ),
        ContextItem::new(
            "prompt",
            ContextKind::Prompt,
            "Explain how to reduce the context sent for this task.",
            false,
        ),
        ContextItem::new(
            "repository",
            ContextKind::Repository,
            "The repository contains a Rust core and a CLI adapter for local evaluation.",
            false,
        ),
        ContextItem::new(
            "tool-output",
            ContextKind::ToolOutput,
            "Verbose command output that is not needed for this particular task.",
            false,
        ),
    ]);
    let saver = DeterministicPruner::new(35);
    let transformed = saver.apply(&baseline);
    let evaluation = evaluate(&baseline, &transformed.package, true);

    println!("Tokenmill provisional context-saver demo");
    println!("mode: {:?}", IntegrationMode::Compatible);
    println!("measurement: estimated, local-only");
    println!(
        "before estimated tokens: {}",
        evaluation.baseline_estimated_tokens
    );
    println!(
        "after estimated tokens: {}",
        evaluation.transformed_estimated_tokens
    );
    println!(
        "estimated tokens saved: {}",
        evaluation.estimated_tokens_saved
    );
    println!("reduction: {:.1}%", evaluation.reduction_percent);
    println!("dropped items: {:?}", transformed.report.dropped_item_ids);
    println!("task success: {}", evaluation.task_success);
    println!("accepted: {}", evaluation.accepted());
}

fn print_help() {
    println!("Tokenmill provisional CLI");
    println!();
    println!("Usage:");
    println!("  tokenmill demo    Run the local deterministic saver evaluation");
    println!("  tokenmill replay  Run the ACP-compatible replay harness");
    println!(
        "  tokenmill acp-check <github-copilot-acp-executable> [cwd]  Probe GitHub Copilot ACP over stdio"
    );
    println!(
        "  tokenmill acp-session-check <github-copilot-acp-executable> [cwd]  Create a GitHub Copilot ACP session"
    );
    println!(
        "  tokenmill acp-prompt <github-copilot-acp-executable> <cwd> <prompt>  Send one GitHub Copilot ACP prompt"
    );
    println!(
        "  tokenmill acp-context-prompt <github-copilot-acp-executable> <cwd> <context.json> <max-tokens> [--saver on|off] [--routing on|off] [--mode strict|compatible] [--report <path>]"
    );
    println!(
        "  tokenmill acp-paired-context-prompt <github-copilot-acp-executable> <cwd> <context.json> <max-tokens> [--mode strict|compatible] [--task-success pass|fail|unknown] [--report <path>] [--history <path>]"
    );
    println!("  tokenmill eval-history <history.jsonl>  Summarize redacted paired evaluations");
    println!("  tokenmill help    Show this help");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        AcpUsageSummary, LiveVariant, append_paired_observation_report, fs, parse_context_package,
        parse_context_prompt_options, parse_paired_context_options, route_status_for_agent,
        summarize_evaluation_history, write_observation_report, write_paired_observation_report,
    };
    use serde_json::Value;
    use tokenmill_core::{ContextKind, IntegrationMode, RouteStatus};

    #[test]
    fn verifies_only_the_github_copilot_agent_identity() {
        let github_copilot_path =
            Path::new(r"C:\Users\user\AppData\Local\github-copilot-sdk\cli\1.0.86\copilot.exe");
        let microsoft_copilot_path = Path::new(r"C:\Program Files\Microsoft\copilot.exe");

        assert_eq!(
            route_status_for_agent(microsoft_copilot_path, Some("GitHub Copilot")),
            RouteStatus::Verified
        );
        assert_eq!(
            route_status_for_agent(microsoft_copilot_path, Some("GitHub Copilot CLI")),
            RouteStatus::Verified
        );
        assert_eq!(
            route_status_for_agent(github_copilot_path, Some("Copilot")),
            RouteStatus::Verified
        );
        assert_eq!(
            route_status_for_agent(microsoft_copilot_path, Some("Copilot")),
            RouteStatus::Unverified
        );
        assert_eq!(
            route_status_for_agent(github_copilot_path, Some("Other agent")),
            RouteStatus::Unverified
        );
        assert_eq!(
            route_status_for_agent(github_copilot_path, None),
            RouteStatus::Unverified
        );
    }

    #[test]
    fn parses_context_prompt_policy_options() {
        let options = parse_context_prompt_options(
            [
                "--saver",
                "off",
                "--routing",
                "off",
                "--mode",
                "compatible",
                "--report",
                "observation.jsonl",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("policy options should parse");

        assert!(!options.policy.saver_enabled);
        assert!(!options.policy.routing_enabled);
        assert_eq!(options.policy.mode, IntegrationMode::Compatible);
        assert_eq!(
            options.report_path,
            Some(PathBuf::from("observation.jsonl"))
        );
    }

    #[test]
    fn parses_paired_context_options() {
        let options = parse_paired_context_options(
            [
                "--mode",
                "compatible",
                "--task-success",
                "pass",
                "--report",
                "paired.jsonl",
                "--history",
                "history.jsonl",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("paired options should parse");

        assert_eq!(options.mode, IntegrationMode::Compatible);
        assert_eq!(options.report_path, Some(PathBuf::from("paired.jsonl")));
        assert_eq!(options.history_path, Some(PathBuf::from("history.jsonl")));
        assert_eq!(options.task_success, Some(true));

        let defaults = parse_paired_context_options(std::iter::empty::<String>())
            .expect("paired defaults should parse");
        assert_eq!(defaults.task_success, None);
    }

    #[test]
    fn parses_context_package_json() {
        let context = parse_context_package(
            r#"{
                "items": [
                    {"id":"instructions","kind":"instruction","content":"Be correct.","protected":true},
                    {"id":"output","kind":"tool-output","content":"Verbose output.","protected":false}
                ]
            }"#,
        )
        .expect("context JSON should parse");

        assert_eq!(context.items.len(), 2);
        assert_eq!(context.items[0].kind, ContextKind::Instruction);
        assert!(context.items[0].protected);
        assert_eq!(context.items[1].kind, ContextKind::ToolOutput);
    }

    #[test]
    fn parses_context_package_json_with_utf8_bom() {
        let context = parse_context_package(
            "\u{feff}{\"items\":[{\"id\":\"prompt\",\"kind\":\"prompt\",\"content\":\"Hello.\",\"protected\":true}]}",
        )
        .expect("context JSON should parse");

        assert_eq!(context.items[0].content, "Hello.");
    }

    #[test]
    fn observation_report_excludes_raw_context() {
        let path = std::env::temp_dir().join("tokenmill-observation-test.json");
        let observation = tokenmill_core::Observation {
            run_id: "test-run".to_owned(),
            adapter: "acp".to_owned(),
            route_status: tokenmill_core::RouteStatus::Verified,
            provider: Some("copilot".to_owned()),
            model: None,
            mode: tokenmill_core::IntegrationMode::Strict,
            saver_enabled: true,
            routing_enabled: true,
            saver_name: Some("deterministic-pruner".to_owned()),
            before_estimated_tokens: 20,
            after_estimated_tokens: 10,
            measurement: tokenmill_core::MeasurementStatus::Estimated,
            latency_millis: Some(3),
            failure_reason: None,
            task_success: None,
            outcome: tokenmill_core::ObservationOutcome::Completed,
        };

        write_observation_report(
            &path,
            &observation,
            &AcpUsageSummary {
                update_count: 1,
                latest_used: Some(12),
                latest_size: Some(100),
            },
        )
        .expect("report should write");
        let report = fs::read_to_string(&path).expect("report should read");
        fs::remove_file(path).expect("report should be removed");
        let report: Value = serde_json::from_str(report.trim()).expect("report should be JSON");

        assert_eq!(report["observation"]["before_estimated_tokens"], 20);
        assert_eq!(report["usage_update_count"], 1);
        assert_eq!(report["usage"]["latest_used"], 12);
        assert_eq!(report["usage"]["latest_size"], 100);
        assert!(report.get("content").is_none());
    }

    #[test]
    fn paired_observation_report_excludes_raw_context() {
        let path = std::env::temp_dir().join("tokenmill-paired-observation-test.json");
        let observation = tokenmill_core::Observation {
            run_id: "paired-run".to_owned(),
            adapter: "acp".to_owned(),
            route_status: tokenmill_core::RouteStatus::Verified,
            provider: Some("copilot".to_owned()),
            model: None,
            mode: tokenmill_core::IntegrationMode::Strict,
            saver_enabled: false,
            routing_enabled: true,
            saver_name: None,
            before_estimated_tokens: 20,
            after_estimated_tokens: 20,
            measurement: tokenmill_core::MeasurementStatus::Estimated,
            latency_millis: Some(3),
            failure_reason: None,
            task_success: None,
            outcome: tokenmill_core::ObservationOutcome::Completed,
        };
        let saver_off = LiveVariant {
            name: "saver-off",
            observation: observation.clone(),
            stop_reason: "end_turn".to_owned(),
            update_count: 1,
            usage: AcpUsageSummary {
                update_count: 1,
                latest_used: Some(20),
                latest_size: Some(100),
            },
        };
        let saver_on = LiveVariant {
            name: "saver-on",
            observation: tokenmill_core::Observation {
                saver_enabled: true,
                after_estimated_tokens: 10,
                ..observation
            },
            stop_reason: "end_turn".to_owned(),
            update_count: 1,
            usage: AcpUsageSummary {
                update_count: 1,
                latest_used: Some(10),
                latest_size: Some(100),
            },
        };

        write_paired_observation_report(&path, &saver_off, &saver_on, Some(true))
            .expect("paired report should write");
        let report = fs::read_to_string(&path).expect("paired report should read");
        fs::remove_file(path).expect("paired report should be removed");
        let report: Value =
            serde_json::from_str(report.trim()).expect("paired report should be JSON");

        assert_eq!(report["report_type"], "paired_live_evaluation");
        assert_eq!(report["task_success"], true);
        assert_eq!(report["accepted"], true);
        assert_eq!(report["comparison"]["estimated_tokens_saved"], 10);
        assert_eq!(report["variants"][1]["usage"]["latest_used"], 10);
        assert!(report.get("content").is_none());

        let history_path = std::env::temp_dir().join("tokenmill-paired-history-test.jsonl");
        let _ = fs::remove_file(&history_path);
        append_paired_observation_report(&history_path, &saver_off, &saver_on, Some(true))
            .expect("history should append");
        append_paired_observation_report(&history_path, &saver_off, &saver_on, None)
            .expect("history should append");
        let summary = summarize_evaluation_history(&history_path).expect("history should parse");
        fs::remove_file(history_path).expect("history should be removed");

        assert_eq!(summary.run_count, 2);
        assert_eq!(summary.accepted_count, 1);
        assert_eq!(summary.rejected_count, 0);
        assert_eq!(summary.unknown_count, 1);
        assert_eq!(summary.estimated_tokens_saved, 20);
        assert_eq!(summary.average_reduction_percent, 50.0);
    }

    #[test]
    fn evaluation_history_rejects_unrelated_records() {
        let path = std::env::temp_dir().join("tokenmill-invalid-history-test.jsonl");
        fs::write(
            &path,
            "{\"schema_version\":1,\"report_type\":\"observation\"}\n",
        )
        .expect("history should write");

        let error = summarize_evaluation_history(&path).expect_err("unrelated record must fail");
        fs::remove_file(path).expect("history should be removed");

        assert!(error.contains("line 1: not a paired live evaluation"));
    }
}
