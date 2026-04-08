//! Multi-Instance Management for Hermes Agents
//!
//! Manages multiple Hermes instances with environment tabs,
//! each showing its own context, status, and conversation.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Environment type for visual differentiation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Remote,
    Production,
    Development,
    Staging,
}

impl Environment {
    pub fn color(&self) -> &'static str {
        match self {
            Environment::Local => "#5ce0d8",      // teal
            Environment::Remote => "#7c5cfc",      // purple
            Environment::Production => "#fc7c5c",  // red/orange
            Environment::Development => "#5cfc7c", // green
            Environment::Staging => "#fcfc5c",    // yellow
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Environment::Local => "Local",
            Environment::Remote => "Remote",
            Environment::Production => "Production",
            Environment::Development => "Development",
            Environment::Staging => "Staging",
        }
    }
}

/// Hardware/environment info for an instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentContext {
    pub os: Option<String>,
    pub hostname: Option<String>,
    pub gpu: Option<String>,
    pub vram_gb: Option<f32>,
    pub memory_gb: Option<f32>,
    pub storage_gb: Option<f32>,
    pub cuda_version: Option<String>,
    pub uptime_seconds: Option<u64>,
}

impl EnvironmentContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Format as display string
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref os) = self.os {
            parts.push(os.clone());
        }
        if let Some(ref gpu) = self.gpu {
            parts.push(gpu.clone());
        }
        if let Some(vram) = self.vram_gb {
            parts.push(format!("{:.0}GB VRAM", vram));
        }
        if let Some(uptime) = self.uptime_seconds {
            let days = uptime / 86400;
            let hours = (uptime % 86400) / 3600;
            if days > 0 {
                parts.push(format!("{}d {}h uptime", days, hours));
            } else if hours > 0 {
                parts.push(format!("{}h uptime", hours));
            }
        }
        parts.join(" │ ")
    }
    
    /// Full detail string for tooltip
    pub fn full_detail(&self) -> String {
        let mut lines = Vec::new();
        if let Some(ref os) = self.os { lines.push(format!("OS: {}", os)); }
        if let Some(ref hostname) = self.hostname { lines.push(format!("Host: {}", hostname)); }
        if let Some(ref gpu) = self.gpu { lines.push(format!("GPU: {}", gpu)); }
        if let Some(vram) = self.vram_gb { lines.push(format!("VRAM: {:.0}GB", vram)); }
        if let Some(mem) = self.memory_gb { lines.push(format!("RAM: {:.0}GB", mem)); }
        if let Some(disk) = self.storage_gb { lines.push(format!("Disk: {:.0}GB", disk)); }
        if let Some(ref cuda) = self.cuda_version { lines.push(format!("CUDA: {}", cuda)); }
        if let Some(uptime) = self.uptime_seconds {
            let days = uptime / 86400;
            let hours = (uptime % 86400) / 3600;
            lines.push(format!("Uptime: {}d {}h", days, hours));
        }
        lines.join("\n")
    }
}

/// A single Hermes instance tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceTab {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub environment: Environment,
    pub color: String,
    pub is_active: bool,
    pub status: crate::AgentStatus,
    pub model: String,
    pub tokens_today: u64,
    pub last_seen: DateTime<Utc>,
    pub context: EnvironmentContext,
    pub project: Option<String>,
    pub unread_count: usize,
}

impl InstanceTab {
    pub fn new(id: &str, name: &str, endpoint: &str, env: Environment) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            environment: env.clone(),
            color: env.color().to_string(),
            is_active: false,
            status: crate::AgentStatus::Offline,
            model: String::new(),
            tokens_today: 0,
            last_seen: Utc::now(),
            context: EnvironmentContext::new(),
            project: None,
            unread_count: 0,
        }
    }
    
    /// Get display label (name + environment indicator)
    pub fn label(&self) -> String {
        format!("{}:{}", self.environment.label(), self.name)
    }
    
    /// Get status emoji
    pub fn status_emoji(&self) -> &'static str {
        match self.status {
            crate::AgentStatus::Running => "🟢",
            crate::AgentStatus::Idle => "⚪",
            crate::AgentStatus::Busy => "🟡",
            crate::AgentStatus::Error => "🔴",
            crate::AgentStatus::Offline => "⚫",
        }
    }
}

/// Manager for all instance tabs
pub struct InstanceManager {
    instances: RwLock<HashMap<String, InstanceTab>>,
    active_id: RwLock<Option<String>>,
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            active_id: RwLock::new(None),
        }
    }
    
    /// Add a new instance tab
    pub fn add_instance(&self, tab: InstanceTab) {
        let mut instances = self.instances.write();
        let mut active_id = self.active_id.write();
        
        // If this is the first instance, make it active
        if instances.is_empty() {
            active_id.insert(tab.id.clone());
        }
        
        // Mark the instance appropriately
        let mut tab = tab;
        if let Some(ref aid) = *active_id {
            if tab.id == *aid {
                tab.is_active = true;
            }
        }
        
        instances.insert(tab.id.clone(), tab);
    }
    
    /// Remove an instance
    pub fn remove_instance(&self, id: &str) {
        let mut instances = self.instances.write();
        let mut active_id = self.active_id.write();
        
        instances.remove(id);
        
        // If we removed the active one, activate the first remaining
        if active_id.as_deref() == Some(id) {
            *active_id = instances.keys().next().cloned();
        }
    }
    
    /// Switch to an instance
    pub fn set_active(&self, id: &str) {
        let mut active_id = self.active_id.write();
        *active_id = Some(id.to_string());
        
        // Update is_active flag
        drop(active_id);
        let mut instances = self.instances.write();
        for (iid, inst) in instances.iter_mut() {
            inst.is_active = (iid == id);
        }
    }
    
    /// Get active instance
    pub fn get_active(&self) -> Option<InstanceTab> {
        let active_id = self.active_id.read();
        let instances = self.instances.read();
        active_id.as_ref().and_then(|id| instances.get(id).cloned())
    }
    
    /// Get all instances
    pub fn get_all(&self) -> Vec<InstanceTab> {
        let instances = self.instances.read();
        instances.values().cloned().collect()
    }
    
    /// Get instance by ID
    pub fn get(&self, id: &str) -> Option<InstanceTab> {
        let instances = self.instances.read();
        instances.get(id).cloned()
    }
    
    /// Update instance status
    pub fn update_status(&self, id: &str, status: crate::AgentStatus) {
        let mut instances = self.instances.write();
        if let Some(inst) = instances.get_mut(id) {
            inst.status = status;
            inst.last_seen = Utc::now();
        }
    }
    
    /// Update environment context
    pub fn update_context(&self, id: &str, context: EnvironmentContext) {
        let mut instances = self.instances.write();
        if let Some(inst) = instances.get_mut(id) {
            inst.context = context;
        }
    }
    
    /// Update token count
    pub fn update_tokens(&self, id: &str, tokens: u64) {
        let mut instances = self.instances.write();
        if let Some(inst) = instances.get_mut(id) {
            inst.tokens_today = tokens;
        }
    }
    
    /// Increment unread count
    pub fn increment_unread(&self, id: &str) {
        let mut instances = self.instances.write();
        if let Some(inst) = instances.get_mut(id) {
            inst.unread_count += 1;
        }
    }
    
    /// Clear unread count
    pub fn clear_unread(&self, id: &str) {
        let mut instances = self.instances.write();
        if let Some(inst) = instances.get_mut(id) {
            inst.unread_count = 0;
        }
    }
    
    /// Get instances by environment
    pub fn get_by_environment(&self, env: &Environment) -> Vec<InstanceTab> {
        let instances = self.instances.read();
        instances
            .values()
            .filter(|i| i.environment == *env)
            .cloned()
            .collect()
    }
    
    /// Count instances
    pub fn count(&self) -> usize {
        let instances = self.instances.read();
        instances.len()
    }
}

/// Shared instance manager
pub type SharedInstanceManager = std::sync::Arc<InstanceManager>;

/// Create a new shared instance manager
pub fn create_instance_manager() -> SharedInstanceManager {
    std::sync::Arc::new(InstanceManager::new())
}
