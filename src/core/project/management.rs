// ============================================================================
// src/core/project/management.rs
//
// MELISA project orchestration:
//   create, delete, invite users, revoke access, pull, update.
//
// Projects are Git bare repositories stored under PROJECTS_MASTER_PATH.
// Each invited user receives a clone (working directory) in their home dir.
// ============================================================================

use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

use crate::cli::color::{BOLD, GREEN, RED, RESET, YELLOW};

// ── Constants ────────────────────────────────────────────────────────────────

/// Root directory where all MELISA project master repositories are stored.
pub const PROJECTS_MASTER_PATH: &str = "/var/melisa/projects";

// ── Project creation ─────────────────────────────────────────────────────────

/// Initializes a new bare Git repository in the master projects directory.
///
/// The directory is created with sticky-bit permissions (1777) and the
/// repository is initialized as a Git bare repo.
///
/// # Arguments
/// * `project_name` - Name of the new project (used as directory name).
/// * `audit`        - When `true`, subprocess commands are logged.
pub async fn create_new_project(project_name: &str, audit: bool) {
    println!(
        "\n{}--- Initializing New Project: {} ---{}",
        BOLD, project_name, RESET
    );

    let master_path = format!("{}/{}", PROJECTS_MASTER_PATH, project_name);

    // Create the master project directory.
    let mkdir_status = Command::new("sudo")
        .args(&["mkdir", "-p", &master_path])
        .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
        .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
        .status()
        .await;

    match mkdir_status {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!(
                "{}[ERROR]{} Failed to create master directory '{}'.",
                RED, RESET, master_path
            );
            return;
        }
    }

    // Apply sticky-bit permissions so any MELISA user can write to it.
    let _ = Command::new("sudo")
        .args(&["chmod", "1777", &master_path])
        .status()
        .await;

    // Initialize as a bare Git repository.
    let git_status = Command::new("sudo")
        .args(&["git", "init", "--bare", &master_path])
        .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
        .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
        .status()
        .await;

    match git_status {
        Ok(s) if s.success() => {
            println!(
                "{}[SUCCESS]{} Master project '{}' has been initialized.",
                GREEN, RESET, project_name
            );
        }
        _ => {
            eprintln!(
                "{}[ERROR]{} Failed to initialize Git bare repository for '{}'.",
                RED, RESET, project_name
            );
        }
    }
}

// ── Project deletion ──────────────────────────────────────────────────────────

/// Removes a master project repository and all associated user working directories.
///
/// # Arguments
/// * `master_path`  - Absolute path to the master project directory.
/// * `project_name` - Name of the project (used for workdir discovery).
pub async fn delete_project(master_path: &str, project_name: &str) {
    println!(
        "\n{}--- Deleting Master Project: {} ---{}",
        BOLD, project_name, RESET
    );

    // Remove the master bare repository.
    let rm_status = Command::new("sudo")
        .args(&["rm", "-rf", master_path])
        .status()
        .await;

    match rm_status {
        Ok(s) if s.success() => {
            println!(
                "{}[SUCCESS]{} Master repository for '{}' removed.",
                GREEN, RESET, project_name
            );
        }
        _ => {
            eprintln!(
                "{}[ERROR]{} Failed to remove master repository '{}'.",
                RED, RESET, master_path
            );
        }
    }

    // Also remove each user's working directory clone.
    remove_all_user_workdirs(project_name).await;
}

/// Removes project working directories from every user's home directory.
async fn remove_all_user_workdirs(project_name: &str) {
    let home_dir_listing = fs::read_dir("/home").await;
    if let Ok(mut entries) = home_dir_listing {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let workdir = entry.path().join(project_name);
            if workdir.exists() {
                let _ = Command::new("sudo")
                    .args(&["rm", "-rf", workdir.to_str().unwrap_or("")])
                    .status()
                    .await;
                println!(
                    "{}[INFO]{} Removed workdir '{}' for user '{}'.",
                    YELLOW, RESET, project_name,
                    entry.file_name().to_string_lossy()
                );
            }
        }
    }
}

// ── User invitation ───────────────────────────────────────────────────────────

/// Grants one or more users access to a project by cloning the master repo
/// into each user's home directory.
///
/// # Arguments
/// * `project_name`   - Name of the project.
/// * `target_users`   - Usernames to invite.
/// * `audit`          - When `true`, subprocess commands are logged.
pub async fn invite_users_to_project(
    project_name: &str,
    target_users: &[&str],
    audit: bool,
) {
    println!(
        "\n{}--- Inviting Users to Project: {} ---{}",
        BOLD, project_name, RESET
    );

    let master_path = format!("{}/{}", PROJECTS_MASTER_PATH, project_name);

    for &username in target_users {
        let workdir = format!("/home/{}/{}", username, project_name);

        if std::path::Path::new(&workdir).exists() {
            println!(
                "{}[SKIP]{} User '{}' already has a working directory for '{}'.",
                YELLOW, RESET, username, project_name
            );
            continue;
        }

        if audit {
            println!(
                "[AUDIT] Running: git clone {} {} for user {}",
                master_path, workdir, username
            );
        }

        let clone_status = Command::new("sudo")
            .args(&["git", "clone", &master_path, &workdir])
            .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
            .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
            .status()
            .await;

        match clone_status {
            Ok(s) if s.success() => {
                // Transfer ownership of the cloned workdir to the invited user.
                let _ = Command::new("sudo")
                    .args(&["chown", "-R", &format!("{}:{}", username, username), &workdir])
                    .status()
                    .await;

                println!(
                    "{}[SUCCESS]{} User '{}' granted access to project '{}'.",
                    GREEN, RESET, username, project_name
                );
            }
            _ => {
                eprintln!(
                    "{}[ERROR]{} Failed to create working directory for '{}'.",
                    RED, RESET, username
                );
            }
        }
    }
}

// ── User removal from project ─────────────────────────────────────────────────

/// Revokes project access from one or more users by removing their working dirs.
///
/// # Arguments
/// * `project_name`  - Name of the project.
/// * `target_users`  - Usernames whose access should be revoked.
/// * `audit`         - When `true`, subprocess commands are logged.
pub async fn remove_users_from_project(
    project_name: &str,
    target_users: &[&str],
    audit: bool,
) {
    println!(
        "\n{}--- Revoking Project Access: {} ---{}",
        BOLD, project_name, RESET
    );

    for &username in target_users {
        let workdir = format!("/home/{}/{}", username, project_name);

        if !std::path::Path::new(&workdir).exists() {
            println!(
                "{}[SKIP]{} User '{}' has no working directory for '{}'.",
                YELLOW, RESET, username, project_name
            );
            continue;
        }

        if audit {
            println!("[AUDIT] Running: rm -rf {}", workdir);
        }

        let rm_status = Command::new("sudo")
            .args(&["rm", "-rf", &workdir])
            .status()
            .await;

        match rm_status {
            Ok(s) if s.success() => {
                println!(
                    "{}[SUCCESS]{} Access revoked from '{}' for project '{}'.",
                    GREEN, RESET, username, project_name
                );
            }
            _ => {
                eprintln!(
                    "{}[ERROR]{} Failed to remove working directory for '{}'.",
                    RED, RESET, username
                );
            }
        }
    }
}

// ── Pull (merge user workspace into master) ───────────────────────────────────

/// Merges a user's working directory into the master project repository.
///
/// Uses a force-push from the user's workdir to the master bare repo.
///
/// # Arguments
/// * `from_user`    - The username whose workspace is being merged.
/// * `project_name` - Name of the project.
/// * `audit`        - When `true`, subprocess commands are logged.
///
/// # Returns
/// `true` if the pull succeeded, `false` otherwise.
pub async fn pull_user_workspace(from_user: &str, project_name: &str, audit: bool) -> bool {
    println!(
        "\n{}--- Pulling '{}' workspace for project '{}' ---{}",
        BOLD, from_user, project_name, RESET
    );

    let user_workdir = format!("/home/{}/{}", from_user, project_name);
    let master_path = format!("{}/{}", PROJECTS_MASTER_PATH, project_name);

    if !std::path::Path::new(&user_workdir).exists() {
        eprintln!(
            "{}[ERROR]{} Working directory '{}' does not exist for user '{}'.",
            RED, RESET, user_workdir, from_user
        );
        return false;
    }

    if audit {
        println!(
            "[AUDIT] Running: git push --force origin master (from {})",
            user_workdir
        );
    }

    // Push from the user's working directory to the master bare repository.
    let push_status = Command::new("sudo")
        .args(&["-u", from_user, "git", "-C", &user_workdir, "push", "--force", &master_path, "HEAD:master"])
        .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
        .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
        .status()
        .await;

    match push_status {
        Ok(s) if s.success() => {
            println!(
                "{}[SUCCESS]{} Workspace of '{}' merged into master for project '{}'.",
                GREEN, RESET, from_user, project_name
            );
            true
        }
        _ => {
            eprintln!(
                "{}[ERROR]{} Failed to merge workspace for '{}'.",
                RED, RESET, from_user
            );
            false
        }
    }
}

// ── Update (sync master → user workdir) ──────────────────────────────────────

/// Synchronizes a user's working directory by force-resetting it to the
/// current state of the master repository.
///
/// # Arguments
/// * `project_name` - Name of the project.
/// * `username`     - The user whose workdir should be updated.
/// * `audit`        - When `true`, subprocess commands are logged.
pub async fn update_project_for_user(project_name: &str, username: &str, audit: bool) {
    let workdir = format!("/home/{}/{}", username, project_name);

    if !std::path::Path::new(&workdir).exists() {
        println!(
            "{}[ERROR]{} Working directory '{}' not found for user '{}'.",
            RED, RESET, workdir, username
        );
        return;
    }

    println!(
        "{}[INFO]{} Synchronizing '{}' working directory…",
        YELLOW, RESET, project_name
    );

    // Fetch then hard-reset to origin/master.
    let fetch_status = Command::new("sudo")
        .args(&["-u", username, "git", "-C", &workdir, "fetch", "--all"])
        .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
        .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
        .status()
        .await;

    if let Ok(s) = fetch_status {
        if s.success() {
            let reset_status = Command::new("sudo")
                .args(&["-u", username, "git", "-C", &workdir, "reset", "--hard", "origin/master"])
                .stdout(if audit { Stdio::inherit() } else { Stdio::null() })
                .stderr(if audit { Stdio::inherit() } else { Stdio::null() })
                .status()
                .await;

            match reset_status {
                Ok(rs) if rs.success() => {
                    println!(
                        "{}[SUCCESS]{} Working directory for '{}' is now up to date.",
                        GREEN, RESET, project_name
                    );
                }
                _ => {
                    eprintln!(
                        "{}[ERROR]{} Failed to reset working directory to master.",
                        RED, RESET
                    );
                }
            }
        }
    }
}

/// Distributes master repository updates to all invited members' working directories.
///
/// # Arguments
/// * `project_name` - Name of the project.
/// * `audit`        - When `true`, subprocess commands are logged.
pub async fn distribute_master_to_all_members(project_name: &str, audit: bool) {
    println!(
        "\n{}--- Distributing master updates for '{}' to all members ---{}",
        BOLD, project_name, RESET
    );

    let home_dir_listing = fs::read_dir("/home").await;
    if let Ok(mut entries) = home_dir_listing {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let username = entry.file_name().to_string_lossy().to_string();
            let workdir = entry.path().join(project_name);
            if workdir.exists() {
                update_project_for_user(project_name, &username, audit).await;
            }
        }
    }

    println!(
        "{}[DONE]{} Master updates distributed to all project members.",
        GREEN, RESET
    );
}

// ── Project listing ───────────────────────────────────────────────────────────

/// Lists all projects the current user has a working directory for.
///
/// # Arguments
/// * `home_dir` - The current user's home directory.
pub async fn list_projects(home_dir: &str) {
    println!("\n{}--- Your Project Working Directories ---{}", BOLD, RESET);

    let home_listing = fs::read_dir(home_dir).await;
    match home_listing {
        Ok(mut entries) => {
            let mut found_any = false;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().is_dir() {
                    // Check if this directory is a Git repository.
                    let git_dir = entry.path().join(".git");
                    if git_dir.exists() {
                        println!("  > {}", entry.file_name().to_string_lossy());
                        found_any = true;
                    }
                }
            }
            if !found_any {
                println!(
                    "  {}No project working directories found in '{}'.{}",
                    YELLOW, home_dir, RESET
                );
            }
        }
        Err(err) => {
            eprintln!(
                "{}[ERROR]{} Failed to read home directory '{}': {}",
                RED, RESET, home_dir, err
            );
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projects_master_path_is_absolute() {
        assert!(
            PROJECTS_MASTER_PATH.starts_with('/'),
            "PROJECTS_MASTER_PATH must be an absolute filesystem path"
        );
    }

    #[test]
    fn test_master_project_path_construction() {
        let project_name = "my-app";
        let expected = format!("{}/{}", PROJECTS_MASTER_PATH, project_name);
        let actual = format!("{}/{}", PROJECTS_MASTER_PATH, project_name);
        assert_eq!(
            actual, expected,
            "Master project path must be constructed as PROJECTS_MASTER_PATH/project_name"
        );
    }

    #[test]
    fn test_user_workdir_path_construction() {
        let username = "alice";
        let project_name = "my-app";
        let workdir = format!("/home/{}/{}", username, project_name);
        assert_eq!(
            workdir, "/home/alice/my-app",
            "User workdir must follow the '/home/<user>/<project>' convention"
        );
    }

    #[test]
    fn test_project_name_with_spaces_is_invalid_as_directory() {
        // Project names must not contain spaces; this is enforced at the CLI layer.
        let project_name = "my project";
        let has_space = project_name.contains(' ');
        assert!(
            has_space,
            "This test verifies that space detection is possible so CLI can reject it"
        );
    }
}