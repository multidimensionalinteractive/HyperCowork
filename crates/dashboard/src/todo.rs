//! Todo list module for task tracking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

/// Task item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: super::inbox::Priority,
    pub project: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Todo list collection
pub struct TodoList {
    tasks: RwLock<HashMap<String, Task>>,
    order: RwLock<Vec<String>>,
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            order: RwLock::new(Vec::new()),
        }
    }

    /// Create a new task
    pub fn create_task(&self, title: &str, description: Option<&str>, project: Option<&str>) -> Task {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            status: TaskStatus::Todo,
            priority: super::inbox::Priority::Normal,
            project: project.map(String::from),
            agent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        let mut tasks = self.tasks.write();
        let mut order = self.order.write();

        tasks.insert(task.id.clone(), task.clone());
        order.push(task.id.clone());

        task
    }

    /// Get task by ID
    pub fn get_task(&self, id: &str) -> Option<Task> {
        let tasks = self.tasks.read();
        tasks.get(id).cloned()
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read();
        tasks.values().cloned().collect()
    }

    /// Get tasks by status
    pub fn get_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let tasks = self.tasks.read();
        let mut result: Vec<Task> = tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    /// Get tasks by project
    pub fn get_by_project(&self, project: &str) -> Vec<Task> {
        let tasks = self.tasks.read();
        tasks
            .values()
            .filter(|t| t.project.as_deref() == Some(project))
            .cloned()
            .collect()
    }

    /// Get tasks count by status
    pub fn count_by_status(&self) -> HashMap<TaskStatus, usize> {
        let tasks = self.tasks.read();
        let mut counts = HashMap::new();
        for task in tasks.values() {
            *counts.entry(task.status.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Update task status
    pub fn update_status(&self, id: &str, status: TaskStatus) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.status = status.clone();
            task.updated_at = Utc::now();
            if status == TaskStatus::Done {
                task.completed_at = Some(Utc::now());
            }
        }
    }

    /// Assign task to agent
    pub fn assign_to_agent(&self, id: &str, agent_id: &str) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.agent_id = Some(agent_id.to_string());
            task.updated_at = Utc::now();
        }
    }

    /// Update task priority
    pub fn update_priority(&self, id: &str, priority: super::inbox::Priority) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.priority = priority;
            task.updated_at = Utc::now();
        }
    }

    /// Delete task
    pub fn delete(&self, id: &str) {
        let mut tasks = self.tasks.write();
        let mut order = self.order.write();
        tasks.remove(id);
        order.retain(|i| i != id);
    }

    /// Get todo summary
    pub fn get_summary(&self) -> TodoSummary {
        let tasks = self.tasks.read();
        let mut summary = TodoSummary::default();

        for task in tasks.values() {
            match task.status {
                TaskStatus::Todo => summary.todo += 1,
                TaskStatus::InProgress => summary.in_progress += 1,
                TaskStatus::Review => summary.review += 1,
                TaskStatus::Done => summary.done += 1,
            }
        }

        summary.total = tasks.len();
        summary
    }
}

/// Todo summary stats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoSummary {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub review: usize,
    pub done: usize,
}
