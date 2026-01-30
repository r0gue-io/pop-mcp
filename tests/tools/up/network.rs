use crate::common::{is_error, is_success, text, texts, DEFAULT_SURI};
use anyhow::{anyhow, Result};
use pop_mcp_server::executor::PopExecutor;
use pop_mcp_server::tools::call::chain::{call_chain, CallChainParams};
use pop_mcp_server::tools::up::network::{up_network, UpNetworkParams};
use std::sync::OnceLock;

static NETWORK_INFO: OnceLock<NetworkInfo> = OnceLock::new();

struct NetworkInfo {
    relay_url: String,
    para_url: String,
    pid: u32,
}

impl Drop for NetworkInfo {
    fn drop(&mut self) {
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(self.pid.to_string())
            .status();
    }
}

fn parse_ws_urls(output: &str) -> Result<(String, String)> {
    let mut relay = None;
    let mut para = None;

    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('│').trim();
        if let Some(rest) = trimmed.strip_prefix("endpoint:") {
            let url = rest.trim();
            if relay.is_none() {
                relay = Some(url.to_string());
            } else if para.is_none() {
                para = Some(url.to_string());
                break;
            }
        }
    }

    let relay = relay.ok_or_else(|| anyhow!("Missing relay endpoint in output"))?;
    let para = para.ok_or_else(|| anyhow!("Missing parachain endpoint in output"))?;
    Ok((relay, para))
}

fn parse_pid(texts: &[String]) -> Result<u32> {
    for text in texts {
        let trimmed = text.trim();
        if let Some(rest) = trimmed.strip_prefix("pid:") {
            let pid = rest.trim().parse::<u32>()?;
            return Ok(pid);
        }
    }
    Err(anyhow!("Missing pid in output"))
}

fn ensure_network() -> Result<(String, String)> {
    if let Some(info) = NETWORK_INFO.get() {
        return Ok((info.relay_url.clone(), info.para_url.clone()));
    }

    let executor = PopExecutor::new();
    let result = up_network(
        &executor,
        UpNetworkParams {
            path: "../pop-cli/tests/networks/paseo+asset-hub.toml".to_owned(),
            verbose: Some(true),
            skip_confirm: Some(true),
            rm: Some(false),
        },
    )?;

    if is_error(&result) {
        return Err(anyhow!("up_network failed: {}", text(&result)?));
    }

    let output = text(&result)?;
    let urls = parse_ws_urls(&output)?;
    let all_texts = texts(&result);
    let pid = parse_pid(&all_texts)?;

    let info = NetworkInfo {
        relay_url: urls.0.clone(),
        para_url: urls.1.clone(),
        pid,
    };

    let _ = NETWORK_INFO.set(info);
    Ok(urls)
}

#[test]
fn up_network_launches_and_outputs_endpoints() -> Result<()> {
    let (relay_url, para_url) = ensure_network()?;
    assert!(!relay_url.is_empty());
    assert!(!para_url.is_empty());
    Ok(())
}

#[test]
fn up_network_enables_relay_and_parachain_calls() -> Result<()> {
    let (relay_url, para_url) = ensure_network()?;
    let executor = PopExecutor::new();

    let relay_result = call_chain(
        &executor,
        CallChainParams {
            url: relay_url,
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x1234".to_string()]),
            suri: Some(DEFAULT_SURI.to_string()),
            sudo: None,
            metadata: None,
        },
    )?;
    assert!(is_success(&relay_result));

    let para_result = call_chain(
        &executor,
        CallChainParams {
            url: para_url,
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x1234".to_string()]),
            suri: Some(DEFAULT_SURI.to_string()),
            sudo: None,
            metadata: None,
        },
    )?;
    assert!(is_success(&para_result));
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
            skip_confirm: Some(true),
            rm: Some(false),
        },
    )?;

    assert!(is_error(&result));
    let output = text(&result)?;
    assert!(
        output.contains("Timed out")
            || output.contains("No such file")
            || output.contains("not found")
    );
    Ok(())
}
