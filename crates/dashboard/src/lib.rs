//! Dashboard Components for OpenCoWork
//!
//! Provides inbox, todo list, kanban board, and brief generation
//! for the Hermes agent control dashboard.

use std::sync::Arc;

mod inbox;
mod todo;
mod kanban;
mod briefs;

pub use inbox::{Inbox, InboxMessage, MessageType, Priority};
pub use todo::{TodoList, Task, TaskStatus, TodoSummary};
pub use kanban::{KanbanBoard, KanbanCard, KanbanColumn, KanbanColumnData};
pub use briefs::{BriefGenerator, Brief, BriefType, AgentSummary, TasksSummary, CostSummary};

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
