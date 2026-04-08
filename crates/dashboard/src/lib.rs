//! Dashboard Components for HyperCoWork
//!
//! Provides inbox, todo list, kanban board, brief generation,
//! and multi-instance management for Hermes agent control dashboard.

use std::sync::Arc;

mod inbox;
mod todo;
mod kanban;
mod briefs;
mod instance;

// Re-export hermes types for instance module
pub use hypercowork_hermes::{AgentStatus, HermesAgent, TokenCount};

pub use inbox::{Inbox, InboxMessage, MessageType, Priority};
pub use todo::{TodoList, Task, TaskStatus, TodoSummary};
pub use kanban::{KanbanBoard, KanbanCard, KanbanColumn, KanbanColumnData};
pub use briefs::{BriefGenerator, Brief, BriefType, AgentSummary, TasksSummary, CostSummary};
pub use instance::{
    InstanceManager, InstanceTab, Environment, EnvironmentContext,
    SharedInstanceManager, create_instance_manager,
};

/// Dashboard state shared across all components
pub struct DashboardState {
    pub inbox: Inbox,
    pub todo: TodoList,
    pub kanban: KanbanBoard,
    pub briefs: BriefGenerator,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            inbox: Inbox::new(),
            todo: TodoList::new(),
            kanban: KanbanBoard::new(),
            briefs: BriefGenerator::new(),
        }
    }
}

/// Shared dashboard state
pub type SharedDashboard = Arc<DashboardState>;

/// Create a new shared dashboard
pub fn create_dashboard() -> SharedDashboard {
    Arc::new(DashboardState::new())
}
