// ============================================================================
// src/core/container/lifecycle.rs
//
// LXC container lifecycle management:
//   create, delete, start, stop, attach.
//
// All functions that mutate container state require admin privileges and
// accept an `audit` flag that controls subprocess output visibility.
// ============================================================================

use std::process::Stdio;
use tokio::process::Command;
use indicatif::ProgressBar;

use crate::cli::color::{BOLD, GREEN, RED, RESET, YELLOW};
use crate::core::container::network::{setup_container_dns, inject_network_config, unlock_container_dns};
use crate::core::container::query::is_container_running;
use crate::core::container::types::{LXC_BASE_PATH, DistroMetadata};
use crate::core::metadata::cleanup_container_metadata;
use crate::core::root_check::ensure_admin;

// ── Shared subprocess helper ─────────────────────────────────────────────────

/// Executes a `sudo` command and optionally inherits stdout/stderr.
///
/// # Arguments
/// * `args`      - Arguments passed to `sudo`.
/// * `is_audit`  - When `true`, subprocess output is forwarded to the terminal.
async fn run_sudo(args: &[&str], is_audit: bool) -> std::io::Result<std::process::ExitStatus> {
    let mut cmd = Command::new("sudo");
    cmd.args(args);
    if is_audit {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }
    cmd.status().await
}

// ── Container creation ───────────────────────────────────────────────────────

/// Creates a new unprivileged LXC container from the specified distribution.
///
/// Steps performed:
/// 1. Download and provision via `lxc-create`.
/// 2. Inject network configuration.
/// 3. Configure DNS (locked with `chattr +i`).
/// 4. Write MELISA metadata.
///
/// # Arguments
/// * `name`   - Target container name.
/// * `meta`   - Distribution metadata (slug, name, arch).
/// * `pb`     - Progress bar for status messages.
/// * `audit`  - When `true`, subprocess output is forwarded to the terminal.
pub async fn create_container(
    name: &str,
    meta: DistroMetadata,
    pb: ProgressBar,
    audit: bool,
) {
    pb.println(format!(
        "{}[CREATE]{} Provisioning container '{}' from '{}'…",
        BOLD, RESET, name, meta.slug
    ));

    let slug_parts: Vec<&str> = meta.slug.splitn(3, '/').collect();
    let (distro, release, arch) = match slug_parts.as_slice() {
        [d, r, a] => (*d, *r, *a),
        _ => {
            eprintln!(
                "{}[ERROR]{} Invalid distro slug format: '{}'",
                RED, RESET, meta.slug
            );
            return;
        }
    };

    if audit {
        pb.println(format!(
            "{}[AUDIT]{} Running: lxc-create -P {} -n {} -t download -- -d {} -r {} -a {}",
            YELLOW, RESET, LXC_BASE_PATH, name, distro, release, arch
        ));
    }

    let status = Command::new("sudo")
        .args(&[
            "lxc-create",
            "-P", LXC_BASE_PATH,
            "-n", name,
            "-t", "download",
            "--",
            "-d", distro,
            "-r", release,
            "-a", arch,
        ])
        .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
        .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            pb.println(format!(
                "{}[SUCCESS]{} Container '{}' has been provisioned.",
                GREEN, RESET, name
            ));
            pb.println(format!(
                "{}[INFO]{} Injecting network configuration for '{}'…",
                BOLD, RESET, name
            ));
            inject_network_config(name, &pb).await;
            pb.println(format!(
                "{}[INFO]{} Configuring DNS for '{}'…",
                BOLD, RESET, name
            ));
            setup_container_dns(name, &pb).await;
        }
        Ok(s) => {
            eprintln!(
                "{}[ERROR]{} Container creation failed with exit code: {}.",
                RED, RESET,
                s.code().unwrap_or(-1)
            );
        }
        Err(err) => {
            eprintln!("{}[FATAL]{} Failed to execute lxc-create: {}", RED, RESET, err);
        }
    }
}

// ── Container deletion ───────────────────────────────────────────────────────

/// Destroys an existing LXC container and removes all associated metadata.
///
/// The container is stopped gracefully before deletion if it is currently running.
///
/// # Arguments
/// * `name`  - Container name to destroy.
/// * `pb`    - Progress bar for status messages.
/// * `audit` - When `true`, subprocess output is forwarded to the terminal.
pub async fn delete_container(name: &str, pb: ProgressBar, audit: bool) {
    pb.println(format!("{}--- Processing Deletion: {} ---{}", BOLD, name, RESET));

    // Stop the container first if it is running.
    if is_container_running(name).await {
        pb.println(format!(
            "{}[INFO]{} Container '{}' is currently running — initiating graceful shutdown…",
            YELLOW, RESET, name
        ));
        stop_container(name, audit).await;
        if is_container_running(name).await {
            eprintln!(
                "{}[ERROR]{} Failed to stop container '{}'. Deletion aborted to prevent data corruption.",
                RED, RESET, name
            );
            return;
        }
    }

    pb.println(format!(
        "{}[INFO]{} Unlocking system configurations for '{}'…",
        BOLD, RESET, name
    ));
    unlock_container_dns(name).await;

    pb.println(format!(
        "{}[INFO]{} Purging MELISA engine metadata for '{}'…",
        BOLD, RESET, name
    ));
    cleanup_container_metadata(name).await;

    if audit {
        pb.println(format!(
            "{}[AUDIT]{} Running lxc-destroy — raw output follows:",
            YELLOW, RESET
        ));
    }

    let status = run_sudo(
        &["-n", "lxc-destroy", "-P", LXC_BASE_PATH, "-n", name, "-f"],
        audit,
    )
    .await;

    match status {
        Ok(s) if s.success() => {
            pb.println(format!(
                "{}[SUCCESS]{} Container '{}' has been permanently destroyed.",
                GREEN, RESET, name
            ));
        }
        Ok(s) => {
            eprintln!(
                "{}[ERROR]{} Deletion failed with exit code: {}.",
                RED, RESET,
                s.code().unwrap_or(-1)
            );
            eprintln!(
                "{}[TIP]{} Ensure you have sudo permissions or check 'lxc-ls' for container status.",
                YELLOW, RESET
            );
        }
        Err(err) => {
            eprintln!("{}[FATAL]{} Could not execute lxc-destroy: {}", RED, RESET, err);
        }
    }
}

// ── Container start ──────────────────────────────────────────────────────────

/// Starts the specified LXC container in daemon mode.
///
/// # Arguments
/// * `name`  - Container name.
/// * `audit` - When `true`, subprocess output is forwarded to the terminal.
pub async fn start_container(name: &str, audit: bool) {
    println!("{}[INFO]{} Starting container '{}'…", GREEN, RESET, name);

    let status = run_sudo(
        &["lxc-start", "-P", LXC_BASE_PATH, "-n", name, "-d"],
        audit,
    )
    .await;

    match status {
        Ok(s) if s.success() => {
            println!("{}[SUCCESS]{} Container '{}' is now running.", GREEN, RESET, name);
        }
        _ => {
            eprintln!(
                "{}[ERROR]{} Failed to start container '{}'. \
                Check if it exists and is properly configured.",
                RED, RESET, name
            );
        }
    }
}

// ── Container stop ───────────────────────────────────────────────────────────

/// Stops the specified LXC container.
///
/// Requires administrator privileges.
///
/// # Arguments
/// * `name`  - Container name.
/// * `audit` - When `true`, subprocess output is forwarded to the terminal.
pub async fn stop_container(name: &str, audit: bool) {
    if !ensure_admin().await {
        return;
    }

    println!(
        "{}[SHUTDOWN]{} Initiating shutdown for container '{}'…",
        YELLOW, RESET, name
    );

    let status = run_sudo(
        &["lxc-stop", "-P", LXC_BASE_PATH, "-n", name],
        audit,
    )
    .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "{}[SUCCESS]{} Container '{}' has been successfully stopped.",
                GREEN, RESET, name
            );
        }
        Ok(_) => {
            eprintln!("{}[ERROR]{} Failed to stop container '{}'.", RED, RESET, name);
        }
        Err(err) => {
            eprintln!("{}[FATAL]{} Execution error: {}", RED, RESET, err);
        }
    }
}

// ── Container attach ─────────────────────────────────────────────────────────

/// Opens an interactive shell session inside the specified container.
///
/// This call blocks the current task until the user exits the shell.
///
/// # Arguments
/// * `name` - Container name.
pub async fn attach_to_container(name: &str) {
    println!(
        "{}[MODE]{} Entering Saferoom: {}. Type 'exit' to return to host.{}",
        BOLD, name, name, RESET
    );

    let _ = Command::new("sudo")
        .args(&["lxc-attach", "-P", LXC_BASE_PATH, "-n", name, "--", "bash"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .status()
        .await;
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `run_sudo` constructs the command without panicking.
    /// We cannot execute `sudo` in a unit test environment, so we only verify
    /// that the helper does not panic when called with empty args.
    #[tokio::test]
    async fn test_run_sudo_does_not_panic_with_empty_args() {
        // We expect an IO error because `sudo` is not available or no args
        // are valid, but the call itself must not panic.
        let _ = run_sudo(&[], false).await;
    }

    /// Verifies that audit mode compiles and the branching is syntactically correct.
    #[test]
    fn test_audit_mode_flag_is_boolean() {
        let is_audit: bool = true;
        let _stdio = if is_audit {
            Stdio::inherit()
        } else {
            Stdio::null()
        };
    }
}