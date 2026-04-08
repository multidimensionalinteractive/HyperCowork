//! Inbox module for unified message/review collection

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use parking_lot::RwLock;

/// Message type in inbox
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Brief,
    Notification,
    AgentMessage,
    UserMessage,
    System,
}

/// Priority level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Normal,
    Low,
}

/// Inbox message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxMessage {
    pub id: String,
    pub title: String,
    pub content: String,
    pub msg_type: MessageType,
    pub priority: Priority,
    pub from_agent: Option<String>,
    pub from_user: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub starred: bool,
}

/// Inbox collection
pub struct Inbox {
    messages: RwLock<VecDeque<InboxMessage>>,
    max_size: usize,
}

impl Default for Inbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Inbox {
    pub fn new() -> Self {
        Self {
            messages: RwLock::new(VecDeque::new()),
            max_size: 100,
        }
    }

    /// Add a message to inbox
    pub fn add_message(&self, mut message: InboxMessage) {
        message.timestamp = Utc::now();
        message.id = uuid::Uuid::new_v4().to_string();

        let mut messages = self.messages.write();
        messages.push_front(message);

        // Trim to max size
        while messages.len() > self.max_size {
            messages.pop_back();
        }
    }

    /// Add a brief message
    pub fn add_brief(&self, title: &str, content: &str, from_agent: &str) {
        let msg = InboxMessage {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            msg_type: MessageType::Brief,
            priority: Priority::Normal,
            from_agent: Some(from_agent.to_string()),
            from_user: None,
            timestamp: Utc::now(),
            read: false,
            starred: false,
        };
        self.add_message(msg);
    }

    /// Add an agent notification
    pub fn add_notification(&self, title: &str, content: &str, from_agent: &str, priority: Priority) {
        let msg = InboxMessage {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            msg_type: MessageType::Notification,
            priority,
            from_agent: Some(from_agent.to_string()),
            from_user: None,
            timestamp: Utc::now(),
            read: false,
            starred: false,
        };
        self.add_message(msg);
    }

    /// Get all messages
    pub fn get_messages(&self) -> Vec<InboxMessage> {
        let messages = self.messages.read();
        messages.iter().cloned().collect()
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        let messages = self.messages.read();
        messages.iter().filter(|m| !m.read).count()
    }

    /// Get messages by type
    pub fn get_by_type(&self, msg_type: MessageType) -> Vec<InboxMessage> {
        let messages = self.messages.read();
        messages
            .iter()
            .filter(|m| m.msg_type == msg_type)
            .cloned()
            .collect()
    }

    /// Mark message as read
    pub fn mark_read(&self, id: &str) {
        let mut messages = self.messages.write();
        for msg in messages.iter_mut() {
            if msg.id == id {
                msg.read = true;
                break;
            }
        }
    }

    /// Toggle star
    pub fn toggle_star(&self, id: &str) {
        let mut messages = self.messages.write();
        for msg in messages.iter_mut() {
            if msg.id == id {
                msg.starred = !msg.starred;
                break;
            }
        }
    }

    /// Delete message
    pub fn delete(&self, id: &str) {
        let mut messages = self.messages.write();
        messages.retain(|m| m.id != id);
    }

    /// Clear all messages
    pub fn clear(&self) {
        let mut messages = self.messages.write();
        messages.clear();
    }
}
