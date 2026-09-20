use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tokenmill_core::{IntegrationMode, RouteStatus, RunPolicy};

pub fn directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("TOKENMILL_HOME") {
        if path.is_empty() {
            return Err("TOKENMILL_HOME must not be empty".into());
        }
        return Ok(path.into());
    }
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA");
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_CONFIG_HOME").or_else(|| {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").into())
    });
    base.map(|base| PathBuf::from(base).join("tokenmill"))
        .ok_or_else(|| "set TOKENMILL_HOME to a writable settings directory".into())
}

pub fn load_from(dir: &Path) -> Result<Option<RunPolicy>, String> {
    let file = match File::open(dir.join("settings.json")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("cannot read settings.json".into()),
    };
    let mut data = Vec::new();
    file.take(8193)
        .read_to_end(&mut data)
        .map_err(|_| "cannot read settings.json")?;
    if data.len() > 8192 {
        return Err("settings.json exceeds 8 KiB".into());
    }
    let value: Value = serde_json::from_slice(&data).map_err(|_| "invalid settings JSON")?;
    let fields = value.as_object().ok_or("settings must be an object")?;
    if fields.len() != 4 || value["schema_version"].as_u64() != Some(1) {
        return Err("unsupported settings schema or fields".into());
    }
    Ok(Some(RunPolicy {
        routing_enabled: value["routing_enabled"]
            .as_bool()
            .ok_or("invalid routing_enabled")?,
        saver_enabled: value["saver_enabled"]
            .as_bool()
            .ok_or("invalid saver_enabled")?,
        mode: match value["mode"].as_str() {
            Some("strict") => IntegrationMode::Strict,
            Some("compatible") => IntegrationMode::Compatible,
            _ => return Err("invalid mode".into()),
        },
        ..RunPolicy::default()
    }))
}

pub fn load() -> Result<RunPolicy, String> {
    Ok(load_from(&directory()?)?.unwrap_or_default())
}

pub fn lock(dir: &Path, name: &str) -> Result<File, String> {
    fs::create_dir_all(dir).map_err(|_| "cannot create settings directory")?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(name))
        .map_err(|_| "cannot open local lock")?;
    file.try_lock()
        .map_err(|_| format!("{name} is busy; retry after the active operation"))?;
    Ok(file)
}

fn policy_json(policy: RunPolicy) -> Value {
    json!({
        "schema_version": 1,
        "routing_enabled": policy.routing_enabled,
        "saver_enabled": policy.saver_enabled,
        "mode": match policy.mode { IntegrationMode::Strict => "strict", IntegrationMode::Compatible => "compatible" },
    })
}

pub fn update(dir: &Path, change: Option<(&str, &str)>) -> Result<RunPolicy, String> {
    let _lock = lock(dir, "settings.lock")?;
    let mut policy = load_from(dir)?.unwrap_or(RunPolicy {
        routing_enabled: false,
        ..RunPolicy::default()
    });
    if let Some((key, value)) = change {
        match (key, value) {
            ("routing", "on" | "off") => policy.routing_enabled = value == "on",
            ("saver", "on" | "off") => policy.saver_enabled = value == "on",
            ("mode", "strict") => policy.mode = IntegrationMode::Strict,
            ("mode", "compatible") => policy.mode = IntegrationMode::Compatible,
            _ => return Err("use routing/saver on|off or mode strict|compatible".into()),
        }
    }
    atomic_write(dir, "settings.json", &policy_json(policy))?;
    Ok(policy)
}

fn atomic_write(dir: &Path, name: &str, value: &Value) -> Result<(), String> {
    let temp = dir.join(format!("{name}.tmp"));
    let result = (|| {
        let mut file = File::create(&temp).map_err(|_| "cannot create temporary settings")?;
        file.write_all(value.to_string().as_bytes())
            .map_err(|_| "cannot write settings")?;
        file.sync_all().map_err(|_| "cannot flush settings")?;
        drop(file);
        fs::rename(&temp, dir.join(name))
            .map_err(|_| "cannot replace settings; previous policy retained")
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result.map_err(String::from)
}

pub struct Request {
    _lock: File,
    dir: PathBuf,
    state: Value,
}

impl Request {
    pub fn begin(policy: RunPolicy) -> Result<Option<Self>, String> {
        let dir = directory()?;
        if !dir.join("settings.json").exists() {
            return Ok(None);
        }
        let request = Self {
            _lock: lock(&dir, "request.lock")?,
            dir,
            state: json!({"schema_version": 1, "outcome": "incomplete", "route_status": "unverified", "policy": policy_json(policy)}),
        };
        atomic_write(&request.dir, "last-request.json", &request.state)?;
        Ok(Some(request))
    }

    pub fn complete(&mut self, route: RouteStatus) -> Result<(), String> {
        self.state["outcome"] = json!("completed");
        self.state["route_status"] = json!(crate::route_status_label(route));
        atomic_write(&self.dir, "last-request.json", &self.state)
    }
}

fn last_request(dir: &Path) -> Result<Value, String> {
    let file = match File::open(dir.join("last-request.json")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Value::Null),
        Err(_) => return Err("cannot read last request status".into()),
    };
    let mut data = Vec::new();
    file.take(8193)
        .read_to_end(&mut data)
        .map_err(|_| "cannot read last request status")?;
    let value: Value = serde_json::from_slice(&data).map_err(|_| "invalid last request status")?;
    if data.len() > 8192
        || value["schema_version"] != 1
        || !matches!(value["outcome"].as_str(), Some("completed" | "incomplete"))
        || !matches!(
            value["route_status"].as_str(),
            Some("verified" | "unverified" | "unavailable" | "bypassed")
        )
    {
        return Err("invalid last request status".into());
    }
    // Project only fixed, validated labels, never arbitrary fields from disk.
    Ok(json!({"outcome": value["outcome"], "route_status": value["route_status"]}))
}

pub fn command(args: &[String]) -> Result<(), String> {
    let dir = directory()?;
    let is_running = running(&dir)?;
    let last_request = last_request(&dir)?;
    let policy = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["init"] => Some(update(&dir, None)?),
        ["set", key, value] => Some(update(&dir, Some((key, value)))?),
        ["show"] => load_from(&dir)?,
        _ => return Err("settings show|init|set <routing|saver|mode> <value>".into()),
    };
    let mut state = policy_json(policy.unwrap_or_default());
    state["configured"] = json!(policy.is_some());
    state["running"] = json!(is_running);
    state["last_request"] = last_request;
    state["scope"] = json!("acp-context-prompt only");
    println!("{state}");
    Ok(())
}

fn running(dir: &Path) -> Result<bool, String> {
    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.join("request.lock"))
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err("cannot read request status".into()),
    };
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(_) => Err("cannot inspect request status".into()),
    }
}
