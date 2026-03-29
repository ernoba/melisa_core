// ============================================================================
// src/core/user/types.rs
//
// User-related types for the MELISA identity and access management subsystem.
// ============================================================================

/// Access level assigned to a MELISA-managed user account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    /// Full management access: users, projects, and LXC containers.
    Admin,
    /// Standard access: project and LXC container management only.
    Regular,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "Administrator"),
            UserRole::Regular => write!(f, "Standard User"),
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
    fn test_user_role_display_admin() {
        assert_eq!(
            UserRole::Admin.to_string(),
            "Administrator",
            "Admin role must display as 'Administrator'"
        );
    }

    #[test]
    fn test_user_role_display_regular() {
        assert_eq!(
            UserRole::Regular.to_string(),
            "Standard User",
            "Regular role must display as 'Standard User'"
        );
    }

    #[test]
    fn test_user_role_equality() {
        assert_eq!(UserRole::Admin, UserRole::Admin);
        assert_ne!(UserRole::Admin, UserRole::Regular);
    }

    #[test]
    fn test_user_role_clone() {
        let original = UserRole::Admin;
        let cloned = original.clone();
        assert_eq!(original, cloned, "Cloned UserRole must equal the original");
    }
}