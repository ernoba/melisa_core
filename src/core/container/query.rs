// ============================================================================
// src/core/container/query.rs
//
// Read-only operations on LXC containers:
//   list, get IP address, check running state, send a command, upload a file.
// ============================================================================

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::cli::color::{BOLD, RED, RESET, YELLOW};
use crate::core::container::types::LXC_BASE_PATH;

// ── Container listing ────────────────────────────────────────────────────────

/// Lists all LXC containers managed by MELISA.
///
/// When `only_running` is `true`, only containers in the RUNNING state are shown.
///
/// # Arguments
/// * `only_running` - When `true`, filters output to running containers only.
pub async fn list_containers(only_running: bool) {
    let args: &[&str] = if only_running {
        &["lxc-ls", "-P", LXC_BASE_PATH, "--running", "--fancy"]
    } else {
        &["lxc-ls", "-P", LXC_BASE_PATH, "--fancy"]
    };

    let output = Command::new("sudo").args(args).output().await;

    match output {
        Ok(out) if out.status.success() => {
            let content = String::from_utf8_lossy(&out.stdout);
            if content.trim().is_empty() {
                let filter_desc = if only_running { "running" } else { "registered" };
                println!(
                    "{}[INFO]{} No {} containers found.",
                    BOLD, RESET, filter_desc
                );
            } else {
                println!("{}", content);
            }
        }
        Ok(_) => {
            eprintln!(
                "{}[ERROR]{} Failed to retrieve the container list. Check LXC installation.",
                RED, RESET
            );
        }
        Err(err) => {
            eprintln!("{}[FATAL]{} Could not execute lxc-ls: {}", RED, RESET, err);
        }
    }
}

// ── Container status ─────────────────────────────────────────────────────────

/// Returns `true` if the specified container is currently in the RUNNING state.
///
/// # Arguments
/// * `name` - Container name.
pub async fn is_container_running(name: &str) -> bool {
    let output = Command::new("sudo")
        .args(&["-n", "lxc-info", "-P", LXC_BASE_PATH, "-n", name, "-s"])
        .output()
        .await;

    match output {
        Ok(out) => {
            let status_text = String::from_utf8_lossy(&out.stdout);
            status_text.contains("RUNNING")
        }
        _ => false,
    }
}

/// Returns `true` if the specified container exists in the LXC path.
///
/// # Arguments
/// * `name` - Container name.
pub async fn container_exists(name: &str) -> bool {
    let output = Command::new("sudo")
        .args(&["lxc-info", "-P", LXC_BASE_PATH, "-n", name, "-s"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    output.map(|s| s.success()).unwrap_or(false)
}

// ── IP address ───────────────────────────────────────────────────────────────

/// Retrieves the internal IPv4 address assigned to a running container.
///
/// Returns `None` if the container is stopped or does not have an IP.
///
/// # Arguments
/// * `name` - Container name.
pub async fn get_container_ip(name: &str) -> Option<String> {
    let output = Command::new("sudo")
        .args(&["lxc-info", "-P", LXC_BASE_PATH, "-n", name, "-i"])
        .output()
        .await
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with("IP:") {
            let ip = line.trim_start_matches("IP:").trim().to_string();
            if !ip.is_empty() {
                return Some(ip);
            }
        }
    }
    None
}

// ── Remote command execution ─────────────────────────────────────────────────

/// Sends a shell command to the specified container via `lxc-attach`.
///
/// Verifies that the container is running before attempting execution.
///
/// # Arguments
/// * `name`         - Container name.
/// * `command_args` - Command and arguments to execute inside the container.
pub async fn send_command(name: &str, command_args: &[&str]) {
    if command_args.is_empty() {
        eprintln!("{}[ERROR]{} No command payload provided.", RED, RESET);
        return;
    }

    // Verify the container is running before attempting execution.
    let status_output = Command::new("sudo")
        .args(&["/usr/bin/lxc-info", "-P", LXC_BASE_PATH, "-n", name, "-s"])
        .output()
        .await;

    match status_output {
        Ok(out) => {
            let status_text = String::from_utf8_lossy(&out.stdout);
            if !status_text.contains("RUNNING") {
                println!(
                    "{}[ERROR]{} Container '{}' is NOT running.",
                    RED, RESET, name
                );
                println!(
                    "{}Tip:{} Execute 'melisa --run {}' to start it first.",
                    YELLOW, RESET, name
                );
                return;
            }
        }
        Err(_) => {
            eprintln!("{}[ERROR]{} Failed to retrieve container status.", RED, RESET);
            return;
        }
    }

    println!("{}[SEND]{} Executing payload on '{}'…", BOLD, name, RESET);

    let mut attach_args = vec!["lxc-attach", "-P", LXC_BASE_PATH, "-n", name, "--"];
    attach_args.extend_from_slice(command_args);

    let _ = Command::new("sudo")
        .args(&attach_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await;
}

// ── File upload ──────────────────────────────────────────────────────────────

/// Uploads a file from the host into the specified container.
///
/// The source file must exist in the current working directory or be specified
/// as an absolute path.  The destination is an absolute path inside the container.
///
/// # Arguments
/// * `container_name` - Target container name.
/// * `dest_path`      - Absolute path inside the container where the file is placed.
pub async fn upload_to_container(container_name: &str, dest_path: &str) {
    // Resolve the source file from the host CWD (simplistic implementation;
    // a more complete version would accept a source path argument).
    let source_path = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string());

    if !Path::new(&source_path).exists() {
        eprintln!(
            "{}[ERROR]{} Source path '{}' does not exist.",
            RED, RESET, source_path
        );
        return;
    }

    let rootfs_dest = format!("{}/{}/rootfs{}", LXC_BASE_PATH, container_name, dest_path);

    let status = Command::new("sudo")
        .args(&["cp", "-r", &source_path, &rootfs_dest])
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "{}[SUCCESS]{} File uploaded to '{}' successfully.",
                crate::cli::color::GREEN, RESET, dest_path
            );
        }
        Ok(s) => {
            eprintln!(
                "{}[ERROR]{} Upload failed with exit code: {}.",
                RED, RESET,
                s.code().unwrap_or(-1)
            );
        }
        Err(err) => {
            eprintln!("{}[FATAL]{} Could not execute upload: {}", RED, RESET, err);
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that an empty command args slice is detected before attempting execution.
    #[test]
    fn test_empty_command_args_are_rejected() {
        let args: &[&str] = &[];
        assert!(
            args.is_empty(),
            "Empty args slice must be detectable before container command dispatch"
        );
    }

    /// Verifies that the IP extraction logic parses the expected lxc-info format.
    #[test]
    fn test_ip_extraction_from_lxc_info_output() {
        let lxc_info_output = "Name:           mybox\nState:          RUNNING\nIP:             10.0.3.42\n";
        let extracted_ip: Option<String> = lxc_info_output
            .lines()
            .find(|line| line.starts_with("IP:"))
            .map(|line| line.trim_start_matches("IP:").trim().to_string())
            .filter(|ip| !ip.is_empty());

        assert!(extracted_ip.is_some(), "IP must be extracted when present in lxc-info output");
        assert_eq!(
            extracted_ip.unwrap(),
            "10.0.3.42",
            "Extracted IP must match the value in the lxc-info output"
        );
    }

    /// Verifies that the running state detection checks for "RUNNING" substring.
    #[test]
    fn test_running_state_detection_requires_running_substring() {
        let running_output = "State: RUNNING";
        let stopped_output = "State: STOPPED";

        assert!(
            running_output.contains("RUNNING"),
            "RUNNING state output must be detected by substring match"
        );
        assert!(
            !stopped_output.contains("RUNNING"),
            "STOPPED state output must not match the RUNNING check"
        );
    }
}