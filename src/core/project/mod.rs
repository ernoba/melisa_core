// ============================================================================
// src/core/project/mod.rs
// ============================================================================

pub mod management;

pub use management::{
    PROJECTS_MASTER_PATH,
    create_new_project, delete_project,
    invite_users_to_project, remove_users_from_project,
    pull_user_workspace, update_project_for_user,
    distribute_master_to_all_members, list_projects,
};