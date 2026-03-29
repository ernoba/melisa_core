// ============================================================================
// src/deployment/dependency.rs
//
// Dependency installation for MELISA deployments.
//
// Executes package installation commands inside an LXC container via
// `lxc-attach`.  Supports system package managers and language-specific
// installers (pip, npm, cargo, gem, composer).
// ============================================================================

use std::process::Stdio;
use tokio::process::Command;

use crate::cli::color::{BOLD, GREEN, RED, RESET, YELLOW};
use crate::core::container::types::LXC_BASE_PATH;
use crate::deployment::manifest::types::DependencySection;

// ── Container command execution ───────────────────────────────────────────────

/// Executes a shell command inside the specified container, inheriting output.
///
/// # Arguments
/// * `container` - Target container name.
/// * `shell_cmd` - Shell command string passed to `sh -c`.
///
/// # Returns
/// `true` if the command exited with status 0, `false` otherwise.
pub async fn lxc_exec(container: &str, shell_cmd: &str) -> bool {
    let status = Command::new("sudo")
        .args(&[
            "lxc-attach", "-P", LXC_BASE_PATH,
            "-n", container,
            "--", "sh", "-c", shell_cmd,
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await;
    status.map(|s| s.success()).unwrap_or(false)
}

/// Executes a shell command inside the specified container, suppressing output.
///
/// Used for probing operations (e.g., `which <pm>`) where output is irrelevant.
///
/// # Arguments
/// * `container` - Target container name.
/// * `shell_cmd` - Shell command string passed to `sh -c`.
///
/// # Returns
/// `true` if the command exited with status 0, `false` otherwise.
pub async fn lxc_exec_silent(container: &str, shell_cmd: &str) -> bool {
    let status = Command::new("sudo")
        .args(&[
            "lxc-attach", "-P", LXC_BASE_PATH,
            "-n", container,
            "--", "sh", "-c", shell_cmd,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    status.map(|s| s.success()).unwrap_or(false)
}

// ── Package manager detection ─────────────────────────────────────────────────

/// Probes the container to detect its system package manager.
///
/// Checks for each known package manager in order; returns the first one found.
///
/// # Arguments
/// * `container` - Target container name.
///
/// # Returns
/// `Some(pm_name)` if a supported package manager was found, `None` otherwise.
pub async fn detect_package_manager(container: &str) -> Option<String> {
    for pm in &["apt-get", "pacman", "dnf", "apk", "zypper"] {
        if lxc_exec_silent(container, &format!("which {}", pm)).await {
            return Some(pm.to_string());
        }
    }
    None
}

// ── System package installation ───────────────────────────────────────────────

/// Installs system-level packages inside the container using the detected
/// package manager.
///
/// Updates the package index before installing.  Returns `true` even when
/// there are no packages to install (no-op).
///
/// # Arguments
/// * `container`   - Target container name.
/// * `deps`        - The `[dependencies]` section from the manifest.
/// * `pkg_manager` - Package manager name as returned by [`detect_package_manager`].
///
/// # Returns
/// `true` if installation succeeded (or was skipped), `false` on error.
pub async fn install_system_deps(
    container: &str,
    deps: &DependencySection,
    pkg_manager: &str,
) -> bool {
    let packages: &Vec<String> = match pkg_manager {
        "apt-get" | "apt" => &deps.apt,
        "pacman"          => &deps.pacman,
        "dnf" | "yum"     => &deps.dnf,
        "apk"             => &deps.apk,
        "zypper"          => &deps.zypper,
        _ => {
            println!(
                "{}[WARNING]{} Unknown package manager '{}' — skipping system deps.",
                YELLOW, RESET, pkg_manager
            );
            return true;
        }
    };

    if packages.is_empty() {
        println!(
            "{}[INFO]{} No system dependencies defined for '{}'.",
            YELLOW, RESET, pkg_manager
        );
        return true;
    }

    println!(
        "{}[DEPLOY]{} Installing {} system package(s) via {}…{}",
        BOLD, RESET, packages.len(), pkg_manager, RESET
    );

    // Update the package index first.
    let update_cmd = build_update_cmd(pkg_manager);
    let _ = lxc_exec_silent(container, &update_cmd).await;

    // Install the packages.
    let install_cmd = match build_system_install_cmd(pkg_manager, deps) {
        Some(cmd) => cmd,
        None => return true, // nothing to install
    };

    let success = lxc_exec(container, &install_cmd).await;
    if success {
        println!("{}[OK]{} System dependencies installed successfully.", GREEN, RESET);
    } else {
        println!("{}[ERROR]{} Failed to install system dependencies.", RED, RESET);
    }
    success
}

// ── Language package installation ─────────────────────────────────────────────

/// Installs language-specific packages inside the container.
///
/// Handles pip, npm, cargo, gem, and composer in sequence.
/// Returns `true` only if all non-empty installer groups succeed.
///
/// # Arguments
/// * `container` - Target container name.
/// * `deps`      - The `[dependencies]` section from the manifest.
///
/// # Returns
/// `true` if all language installers succeeded, `false` if any failed.
pub async fn install_lang_deps(container: &str, deps: &DependencySection) -> bool {
    let mut all_succeeded = true;

    // ── pip ──────────────────────────────────────────────────────────────────
    if !deps.pip.is_empty() {
        println!(
            "{}[DEPLOY]{} Installing {} pip package(s)…{}",
            BOLD, RESET, deps.pip.len(), RESET
        );
        let pkgs = deps.pip.join(" ");
        let cmd = format!("pip3 install --break-system-packages {}", pkgs);
        if lxc_exec(container, &cmd).await {
            println!("{}[OK]{} pip packages installed.", GREEN, RESET);
        } else {
            println!("{}[ERROR]{} pip install failed.", RED, RESET);
            all_succeeded = false;
        }
    }

    // ── npm ──────────────────────────────────────────────────────────────────
    if !deps.npm.is_empty() {
        println!(
            "{}[DEPLOY]{} Installing {} npm package(s) globally…{}",
            BOLD, RESET, deps.npm.len(), RESET
        );
        let pkgs = deps.npm.join(" ");
        let cmd = format!("npm install -g {}", pkgs);
        if lxc_exec(container, &cmd).await {
            println!("{}[OK]{} npm packages installed.", GREEN, RESET);
        } else {
            println!("{}[ERROR]{} npm install failed.", RED, RESET);
            all_succeeded = false;
        }
    }

    // ── cargo ─────────────────────────────────────────────────────────────────
    for crate_name in &deps.cargo {
        println!(
            "{}[DEPLOY]{} cargo install '{}'…{}",
            BOLD, RESET, crate_name, RESET
        );
        let cmd = format!("cargo install {}", crate_name);
        if !lxc_exec(container, &cmd).await {
            println!("{}[ERROR]{} cargo install '{}' failed.", RED, RESET, crate_name);
            all_succeeded = false;
        }
    }

    // ── gem ───────────────────────────────────────────────────────────────────
    if !deps.gem.is_empty() {
        println!(
            "{}[DEPLOY]{} Installing {} gem package(s)…{}",
            BOLD, RESET, deps.gem.len(), RESET
        );
        let pkgs = deps.gem.join(" ");
        let cmd = format!("gem install {}", pkgs);
        if lxc_exec(container, &cmd).await {
            println!("{}[OK]{} gem packages installed.", GREEN, RESET);
        } else {
            println!("{}[ERROR]{} gem install failed.", RED, RESET);
            all_succeeded = false;
        }
    }

    // ── composer ──────────────────────────────────────────────────────────────
    if !deps.composer.is_empty() {
        println!(
            "{}[DEPLOY]{} Installing {} composer package(s)…{}",
            BOLD, RESET, deps.composer.len(), RESET
        );
        let pkgs = deps.composer.join(" ");
        let cmd = format!("composer global require {}", pkgs);
        if lxc_exec(container, &cmd).await {
            println!("{}[OK]{} composer packages installed.", GREEN, RESET);
        } else {
            println!("{}[ERROR]{} composer install failed.", RED, RESET);
            all_succeeded = false;
        }
    }

    all_succeeded
}

// ── Pure command builders (also used in unit tests) ───────────────────────────

/// Builds the package index update command for the given package manager.
///
/// # Arguments
/// * `pkg_manager` - Package manager name.
///
/// # Returns
/// Shell command string.
pub fn build_update_cmd(pkg_manager: &str) -> String {
    match pkg_manager {
        "pacman"       => "pacman -Sy --noconfirm".into(),
        "apk"          => "apk update".into(),
        "zypper"       => "zypper --non-interactive refresh".into(),
        "dnf" | "yum"  => "dnf makecache".into(),
        _              => "apt-get update -y".into(),
    }
}

/// Builds the package installation command for the given package manager.
///
/// Returns `None` when the relevant package list in `deps` is empty.
///
/// # Arguments
/// * `pkg_manager` - Package manager name.
/// * `deps`        - Dependency section containing package lists.
///
/// # Returns
/// `Some(cmd)` if there are packages to install, `None` otherwise.
pub fn build_system_install_cmd(pkg_manager: &str, deps: &DependencySection) -> Option<String> {
    let packages: &Vec<String> = match pkg_manager {
        "apt-get" | "apt" => &deps.apt,
        "pacman"          => &deps.pacman,
        "dnf" | "yum"     => &deps.dnf,
        "apk"             => &deps.apk,
        "zypper"          => &deps.zypper,
        _                 => return None,
    };

    if packages.is_empty() {
        return None;
    }

    let pkg_list = packages.join(" ");
    let cmd = match pkg_manager {
        "pacman"      => format!("pacman -S --noconfirm {}", pkg_list),
        "apk"         => format!("apk add {}", pkg_list),
        "zypper"      => format!("zypper --non-interactive install {}", pkg_list),
        "dnf" | "yum" => format!("dnf install -y {}", pkg_list),
        _             => format!("apt-get install -y {}", pkg_list),
    };
    Some(cmd)
}

/// Returns `true` if any language-specific dependency is defined.
///
/// Used by the deployer to decide whether the language installation step
/// is necessary.
///
/// # Arguments
/// * `deps` - Dependency section to inspect.
pub fn has_lang_deps(deps: &DependencySection) -> bool {
    !deps.pip.is_empty()
        || !deps.npm.is_empty()
        || !deps.cargo.is_empty()
        || !deps.gem.is_empty()
        || !deps.composer.is_empty()
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_update_cmd ─────────────────────────────────────────────────────

    #[test]
    fn test_build_update_cmd_apt_produces_apt_get_update() {
        let cmd = build_update_cmd("apt-get");
        assert!(cmd.contains("apt-get"), "Update command must use apt-get");
        assert!(cmd.contains("update"), "Update command must include the update subcommand");
    }

    #[test]
    fn test_build_update_cmd_pacman_produces_sy_noconfirm() {
        let cmd = build_update_cmd("pacman");
        assert!(cmd.contains("pacman"), "Update command must use pacman");
        assert!(cmd.contains("-Sy"), "pacman update must use -Sy flag");
    }

    #[test]
    fn test_build_update_cmd_apk_produces_apk_update() {
        let cmd = build_update_cmd("apk");
        assert_eq!(cmd, "apk update", "apk update command must be exactly 'apk update'");
    }

    #[test]
    fn test_build_update_cmd_dnf_produces_dnf_makecache() {
        let cmd = build_update_cmd("dnf");
        assert!(cmd.contains("dnf"), "Update command must use dnf");
        assert!(
            cmd.contains("makecache") || cmd.contains("update"),
            "dnf update command must use makecache or update"
        );
    }

    #[test]
    fn test_build_update_cmd_unknown_pm_falls_back_to_apt_get() {
        let cmd = build_update_cmd("chocolatey");
        assert!(
            cmd.contains("apt-get"),
            "Unknown package manager must fall back to apt-get update"
        );
    }

    // ── build_system_install_cmd ─────────────────────────────────────────────

    #[test]
    fn test_build_system_install_cmd_apt_with_packages_produces_install_command() {
        let mut deps = DependencySection::default();
        deps.apt = vec!["curl".into(), "git".into(), "vim".into()];
        let cmd = build_system_install_cmd("apt-get", &deps);
        assert!(cmd.is_some(), "Must produce a command when apt packages are defined");
        let cmd = cmd.unwrap();
        assert!(cmd.contains("apt-get install -y"), "Must use 'apt-get install -y'");
        assert!(cmd.contains("curl"), "Command must include 'curl'");
        assert!(cmd.contains("git"), "Command must include 'git'");
        assert!(cmd.contains("vim"), "Command must include 'vim'");
    }

    #[test]
    fn test_build_system_install_cmd_returns_none_when_package_list_is_empty() {
        let deps = DependencySection::default();
        let cmd = build_system_install_cmd("apt-get", &deps);
        assert!(
            cmd.is_none(),
            "Must return None when no packages are defined for the given PM"
        );
    }

    #[test]
    fn test_build_system_install_cmd_pacman_uses_noconfirm_flag() {
        let mut deps = DependencySection::default();
        deps.pacman = vec!["nodejs".into(), "npm".into()];
        let cmd = build_system_install_cmd("pacman", &deps).unwrap();
        assert!(cmd.contains("pacman -S --noconfirm"), "pacman install must use -S --noconfirm");
        assert!(cmd.contains("nodejs"));
        assert!(cmd.contains("npm"));
    }

    #[test]
    fn test_build_system_install_cmd_apk_uses_add_subcommand() {
        let mut deps = DependencySection::default();
        deps.apk = vec!["python3".into()];
        let cmd = build_system_install_cmd("apk", &deps).unwrap();
        assert!(cmd.contains("apk add"), "apk install must use 'apk add'");
        assert!(cmd.contains("python3"));
    }

    #[test]
    fn test_build_system_install_cmd_unknown_pm_returns_none() {
        let mut deps = DependencySection::default();
        deps.apt = vec!["curl".into()];
        let cmd = build_system_install_cmd("chocolatey", &deps);
        assert!(
            cmd.is_none(),
            "Unknown package manager must return None instead of crashing"
        );
    }

    // ── has_lang_deps ────────────────────────────────────────────────────────

    #[test]
    fn test_has_lang_deps_returns_true_when_pip_is_filled() {
        let mut deps = DependencySection::default();
        deps.pip = vec!["flask".into()];
        assert!(
            has_lang_deps(&deps),
            "has_lang_deps must return true when pip packages are defined"
        );
    }

    #[test]
    fn test_has_lang_deps_returns_false_when_all_lang_lists_are_empty() {
        let deps = DependencySection::default();
        assert!(
            !has_lang_deps(&deps),
            "has_lang_deps must return false when all language dependency lists are empty"
        );
    }

    #[test]
    fn test_has_lang_deps_returns_true_when_cargo_is_filled() {
        let mut deps = DependencySection::default();
        deps.cargo = vec!["ripgrep".into()];
        assert!(
            has_lang_deps(&deps),
            "has_lang_deps must return true when cargo crates are defined"
        );
    }

    #[test]
    fn test_has_lang_deps_returns_true_when_npm_is_filled() {
        let mut deps = DependencySection::default();
        deps.npm = vec!["typescript".into()];
        assert!(
            has_lang_deps(&deps),
            "has_lang_deps must return true when npm packages are defined"
        );
    }

    #[test]
    fn test_has_lang_deps_returns_true_when_gem_is_filled() {
        let mut deps = DependencySection::default();
        deps.gem = vec!["rails".into()];
        assert!(has_lang_deps(&deps), "has_lang_deps must return true when gems are defined");
    }

    #[test]
    fn test_has_lang_deps_returns_true_when_composer_is_filled() {
        let mut deps = DependencySection::default();
        deps.composer = vec!["laravel/framework".into()];
        assert!(
            has_lang_deps(&deps),
            "has_lang_deps must return true when composer packages are defined"
        );
    }
}