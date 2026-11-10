//! Trusted AgentModule manifests and descriptor producers.
//!
//! A module is selected by exact identity (module id, pinned version, OS and
//! architecture), never by a brand match. Every module here is compiled into the
//! daemon; nothing is loaded from the Workspace or from a native agent.

pub(crate) mod opencode;

use super::module_registry::AgentModuleRegistry;

/// The Runtime's trusted module registry. Registration failure is a programming
/// error caught by the module tests, so it fails closed to an empty registry.
pub(crate) fn default_module_registry() -> AgentModuleRegistry {
    let mut registry = AgentModuleRegistry::default();
    if registry.register(opencode::manifest()).is_err() {
        return AgentModuleRegistry::default();
    }
    registry
}
