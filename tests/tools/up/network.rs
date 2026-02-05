use crate::common::{is_error, text, texts};
use anyhow::{anyhow, Result};
use pop_mcp_server::executor::PopExecutor;
use pop_mcp_server::tools::clean::{clean_network, CleanNetworkParams};
use pop_mcp_server::tools::up::network::{up_network, UpNetworkParams};
use std::time::{Duration, Instant};

struct NetworkRun {
    relay_url: String,
    chain_url: String,
    zombie_json: String,
    pop_pid: Option<u32>,
}

impl Drop for NetworkRun {
    fn drop(&mut self) {
        let executor = PopExecutor::new();
        let _ = clean_network(
            &executor,
            CleanNetworkParams {
                path: Some(self.zombie_json.clone()),
                all: None,
                keep_state: Some(false),
            },
        );
        if let Some(pid) = self.pop_pid.take() {
            let _ = std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status();
        }
    }
}

struct CleanupGuard {
    zombie_json: Option<String>,
    pop_pid: Option<u32>,
}

impl CleanupGuard {
    fn new() -> Self {
        Self {
            zombie_json: None,
            pop_pid: None,
        }
    }

    fn arm(&mut self, zombie_json: String) {
        self.zombie_json = Some(zombie_json);
    }

    fn arm_pid(&mut self, pop_pid: Option<u32>) {
        self.pop_pid = pop_pid;
    }

    fn disarm(&mut self) {
        self.zombie_json = None;
        self.pop_pid = None;
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some(path) = self.zombie_json.take() {
            let executor = PopExecutor::new();
            let _ = clean_network(
                &executor,
                CleanNetworkParams {
                    path: Some(path),
                    all: None,
                    keep_state: Some(false),
                },
            );
        }
        if let Some(pid) = self.pop_pid.take() {
            let _ = std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .status();
        }
    }
}

fn parse_ws_urls(texts: &[String]) -> Result<(String, String)> {
    let mut relay = None;
    let mut chain = None;

    for text in texts {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("relay_ws:") {
            relay = Some(rest.trim().to_owned());
        } else if let Some(rest) = trimmed.strip_prefix("chain_ws:") {
            chain = Some(rest.trim().to_owned());
        }
    }

    let relay = relay.ok_or_else(|| anyhow!("Missing relay ws url in output"))?;
    let chain = chain.ok_or_else(|| anyhow!("Missing chain ws url in output"))?;
    Ok((relay, chain))
}

fn parse_zombie_json(texts: &[String]) -> Result<String> {
    for text in texts {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("zombie_json:") {
            let path = rest.trim();
            if !path.is_empty() {
                return Ok(path.to_owned());
            }
        }
    }
    Err(anyhow!("Missing zombie_json in output"))
}

fn parse_base_dir(texts: &[String]) -> Option<String> {
    for text in texts {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("base_dir:") {
            let path = rest.trim();
            if !path.is_empty() {
                return Some(path.to_owned());
            }
        }
    }
    None
}

fn parse_pop_pid(texts: &[String]) -> Option<u32> {
    for text in texts {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("pop_pid:") {
            let pid = rest.trim().parse::<u32>().ok()?;
            return Some(pid);
        }
    }
    None
}

fn wait_for_port_open(port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if crate::common::is_port_in_use(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!("Timed out waiting for port {}", port))
}

fn wait_for_ws(url: &str, timeout: Duration) -> Result<()> {
    let port = crate::common::ws_port_from_url(url)?;
    wait_for_port_open(port, timeout)
}

fn launch_network() -> Result<NetworkRun> {
    let executor = PopExecutor::new();
    let result = up_network(
        &executor,
        UpNetworkParams {
            path: "tests/networks/paseo+asset-hub.toml".to_owned(),
            verbose: Some(true),
        },
    )?;

    if is_error(&result) {
        return Err(anyhow!("up_network failed: {}", text(&result)?));
    }

    let all_texts = texts(&result);
    let mut cleanup = CleanupGuard::new();
    cleanup.arm_pid(parse_pop_pid(&all_texts));

    let zombie_json = match parse_zombie_json(&all_texts) {
        Ok(path) => path,
        Err(_) => {
            if let Some(base_dir) = parse_base_dir(&all_texts) {
                format!("{}/zombie.json", base_dir)
            } else {
                return Err(anyhow!("Missing zombie_json in output"));
            }
        }
    };
    cleanup.arm(zombie_json.clone());

    let urls = parse_ws_urls(&all_texts)?;

    wait_for_ws(&urls.0, Duration::from_secs(60))?;
    wait_for_ws(&urls.1, Duration::from_secs(60))?;

    let run = NetworkRun {
        relay_url: urls.0.clone(),
        chain_url: urls.1.clone(),
        zombie_json,
        pop_pid: parse_pop_pid(&all_texts),
    };
    cleanup.disarm();
    Ok(run)
}

#[test]
fn up_network_launches_and_outputs_endpoints() -> Result<()> {
    let network = launch_network()?;
    assert!(!network.relay_url.is_empty());
    assert!(!network.chain_url.is_empty());
    Ok(())
}

#[test]
fn up_network_invalid_path_fails() -> Result<()> {
    let executor = PopExecutor::new();
    let result = up_network(
        &executor,
        UpNetworkParams {
            path: "./does-not-exist.toml".to_owned(),
            verbose: Some(false),
        },
    )?;

    assert!(is_error(&result));
    let output = text(&result)?;
    assert!(
        output.contains("Could not launch local network")
            || output.contains("No such file")
            || output.contains("not found")
            || output.contains("Failed to parse")
    );
    Ok(())
}
