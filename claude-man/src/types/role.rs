//! Role types for Claude sessions
//!
//! Defines the different roles that can be assigned to Claude sessions
//! based on the MANAGER-based orchestration pattern.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::types::error::ClaudeManError;

/// Role assigned to a Claude session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    /// MANAGER - Orchestrates and coordinates other sessions
    Manager,

    /// ARCHITECT - Designs system architecture and technical decisions
    Architect,

    /// DEVELOPER - Implements code and features
    Developer,

    /// STAKEHOLDER - Represents business requirements and validation
    Stakeholder,
}

impl Role {
    /// Returns the short prefix for session IDs (e.g., "DEV" for Developer)
    pub fn prefix(&self) -> &'static str {
        match self {
            Role::Manager => "MGR",
            Role::Architect => "ARCH",
            Role::Developer => "DEV",
            Role::Stakeholder => "STAKE",
        }
    }

    /// Returns all available roles
    pub fn all() -> &'static [Role] {
        &[Role::Manager, Role::Architect, Role::Developer, Role::Stakeholder]
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Manager => write!(f, "MANAGER"),
            Role::Architect => write!(f, "ARCHITECT"),
            Role::Developer => write!(f, "DEVELOPER"),
            Role::Stakeholder => write!(f, "STAKEHOLDER"),
        }
    }
}

impl FromStr for Role {
    type Err = ClaudeManError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MANAGER" | "MGR" => Ok(Role::Manager),
            "ARCHITECT" | "ARCH" => Ok(Role::Architect),
            "DEVELOPER" | "DEV" => Ok(Role::Developer),
            "STAKEHOLDER" | "STAKE" => Ok(Role::Stakeholder),
            _ => Err(ClaudeManError::InvalidInput(format!(
                "Invalid role '{}'. Valid roles: MANAGER, ARCHITECT, DEVELOPER, STAKEHOLDER",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_prefix() {
        assert_eq!(Role::Manager.prefix(), "MGR");
        assert_eq!(Role::Architect.prefix(), "ARCH");
        assert_eq!(Role::Developer.prefix(), "DEV");
        assert_eq!(Role::Stakeholder.prefix(), "STAKE");
    }

    #[test]
    fn test_role_display() {
        assert_eq!(Role::Manager.to_string(), "MANAGER");
        assert_eq!(Role::Developer.to_string(), "DEVELOPER");
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!("MANAGER".parse::<Role>().unwrap(), Role::Manager);
        assert_eq!("manager".parse::<Role>().unwrap(), Role::Manager);
        assert_eq!("MGR".parse::<Role>().unwrap(), Role::Manager);
        assert_eq!("DEVELOPER".parse::<Role>().unwrap(), Role::Developer);
        assert_eq!("DEV".parse::<Role>().unwrap(), Role::Developer);
        assert_eq!("dev".parse::<Role>().unwrap(), Role::Developer);
    }

    #[test]
    fn test_invalid_role() {
        assert!("INVALID".parse::<Role>().is_err());
    }

    #[test]
    fn test_role_all() {
        let all_roles = Role::all();
        assert_eq!(all_roles.len(), 4);
        assert!(all_roles.contains(&Role::Manager));
        assert!(all_roles.contains(&Role::Developer));
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::Developer;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""DEVELOPER""#);

        let deserialized: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, role);
    }
}
