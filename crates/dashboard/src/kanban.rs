//! Kanban board module

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Kanban column definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KanbanColumn {
    Inbox,
    Todo,
    InProgress,
    Review,
    Done,
}

impl KanbanColumn {
    pub fn as_str(&self) -> &'static str {
        match self {
            KanbanColumn::Inbox => "inbox",
            KanbanColumn::Todo => "todo",
            KanbanColumn::InProgress => "in_progress",
            KanbanColumn::Review => "review",
            KanbanColumn::Done => "done",
        }
    }
}

/// Kanban card (wrapper around task for board display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanCard {
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: super::inbox::Priority,
    pub project: Option<String>,
    pub agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Kanban column with cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanColumnData {
    pub column: KanbanColumn,
    pub cards: Vec<KanbanCard>,
}

/// Kanban board
pub struct KanbanBoard {
    columns: RwLock<HashMap<KanbanColumn, Vec<KanbanCard>>>,
}

impl Default for KanbanBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl KanbanBoard {
    pub fn new() -> Self {
        let columns: HashMap<KanbanColumn, Vec<KanbanCard>> = HashMap::from([
            (KanbanColumn::Inbox, Vec::new()),
            (KanbanColumn::Todo, Vec::new()),
            (KanbanColumn::InProgress, Vec::new()),
            (KanbanColumn::Review, Vec::new()),
            (KanbanColumn::Done, Vec::new()),
        ]);
        
        Self {
            columns: RwLock::new(columns),
        }
    }

    /// Add a card to a column
    pub fn add_card(&self, column: KanbanColumn, card: KanbanCard) {
        let mut cols = self.columns.write();
        if let Some(cards) = cols.get_mut(&column) {
            cards.push(card);
        }
    }

    /// Move a card between columns
    pub fn move_card(&self, task_id: &str, from: KanbanColumn, to: KanbanColumn) -> bool {
        let mut cols = self.columns.write();
        
        // Find and remove from source column
        let mut card: Option<KanbanCard> = None;
        if let Some(cards) = cols.get_mut(&from) {
            if let Some(pos) = cards.iter().position(|c| c.task_id == task_id) {
                card = Some(cards.remove(pos));
            }
        }
        
        // Add to destination column
        if let Some(c) = card {
            if let Some(cards) = cols.get_mut(&to) {
                cards.push(c);
                return true;
            }
        }
        
        false
    }

    /// Get all columns with their cards
    pub fn get_board(&self) -> Vec<KanbanColumnData> {
        let cols = self.columns.read();
        let mut result = Vec::new();
        
        for column in [
            KanbanColumn::Inbox,
            KanbanColumn::Todo,
            KanbanColumn::InProgress,
            KanbanColumn::Review,
            KanbanColumn::Done,
        ] {
            if let Some(cards) = cols.get(&column) {
                result.push(KanbanColumnData {
                    column: column.clone(),
                    cards: cards.clone(),
                });
            }
        }
        
        result
    }

    /// Get card count per column
    pub fn get_counts(&self) -> HashMap<KanbanColumn, usize> {
        let cols = self.columns.read();
        let mut counts: HashMap<KanbanColumn, usize> = HashMap::new();
        for (col, cards) in cols.iter() {
            counts.insert(col.clone(), cards.len());
        }
        counts
    }

    /// Remove card from board
    pub fn remove_card(&self, task_id: &str) {
        let mut cols = self.columns.write();
        let values: Vec<&mut Vec<KanbanCard>> = cols.values_mut().collect();
        for cards in values {
            cards.retain(|c| c.task_id != task_id);
        }
    }

    /// Clear all cards from a column
    pub fn clear_column(&self, column: &KanbanColumn) {
        let mut cols = self.columns.write();
        if let Some(cards) = cols.get_mut(column) {
            cards.clear();
        }
    }

    /// Sync with todo list - rebuild board from tasks
    pub fn sync_from_todo(&self, tasks: &super::TodoList) {
        // Clear all columns first
        {
            let mut cols = self.columns.write();
            let values: Vec<&mut Vec<KanbanCard>> = cols.values_mut().collect();
            for cards in values {
                cards.clear();
            }
        }
        
        // Get all tasks and distribute to columns
        let all_tasks = tasks.get_all_tasks();
        for task in all_tasks {
            let card = KanbanCard {
                task_id: task.id.clone(),
                title: task.title,
                description: task.description,
                priority: task.priority,
                project: task.project,
                agent: task.agent_id,
                created_at: task.created_at,
                updated_at: task.updated_at,
            };
            
            let column = match task.status {
                super::todo::TaskStatus::Todo => KanbanColumn::Todo,
                super::todo::TaskStatus::InProgress => KanbanColumn::InProgress,
                super::todo::TaskStatus::Review => KanbanColumn::Review,
                super::todo::TaskStatus::Done => KanbanColumn::Done,
            };
            
            self.add_card(column, card);
        }
    }
}
