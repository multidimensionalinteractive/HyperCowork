//! Hermes Agent Integration for HyperCoWork
//!
//! Provides client functionality for connecting to Hermes agents,
//! monitoring their status, and managing tasks across the fleet.

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Common Hermes endpoints to try when auto-discovering
const COMMON_HERMES_PORTS: &[(&str, u16)] = &[
    ("localhost", 8080),
    ("localhost", 8081),
    ("localhost", 8082),
    ("localhost", 3000),
    ("127.0.0.1", 8080),
    ("127.0.0.1", 8081),
];

/// Hermes agent client for API communication
#[derive(Clone)]
pub struct HermesClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    name: Option<String>,
}

/// Agent status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Idle,
    Busy,
    Error,
    Offline,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Busy => write!(f, "busy"),
            AgentStatus::Error => write!(f, "error"),
            AgentStatus::Offline => write!(f, "offline"),
        }
    }
}

/// Connected Hermes agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesAgent {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub status: AgentStatus,
    pub model: String,
    pub platform: String,
    pub tokens_today: TokenCount,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub project: Option<String>,
}

impl HermesAgent {
    /// Create a new Hermes agent entry
    pub fn new(id: &str, name: &str, endpoint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            status: AgentStatus::Offline,
            model: String::new(),
            platform: String::new(),
            tokens_today: TokenCount::default(),
            last_seen: chrono::Utc::now(),
            project: None,
        }
    }
}

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
    pub cache: u64,
}

impl TokenCount {
    /// Total tokens used
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache
    }
}

/// Token cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCost {
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_cost: f64,
    pub total_cost: f64,
}

/// Hermes API response for agent status
#[derive(Debug, Deserialize)]
pub struct HermesStatusResponse {
    pub status: String,
    pub model: String,
    pub tokens_used: HermesTokens,
}

#[derive(Debug, Deserialize)]
pub struct HermesTokens {
    pub input: u64,
    pub output: u64,
    pub cache: u64,
}

/// Hermes chat message
#[derive(Debug, Serialize)]
pub struct HermesMessage {
    pub message: String,
}

/// Hermes chat response
#[derive(Debug, Deserialize)]
pub struct HermesChatResponse {
    pub response: String,
}

/// Discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub endpoint: String,
    pub name: String,
    pub model: String,
    pub status: String,
}

impl HermesClient {
    /// Create a new Hermes client - simple one-liner
    ///
    /// # Example
    /// ```
    /// let client = HermesClient::connect("http://localhost:8080", None)?;
    /// ```
    pub fn connect(endpoint: &str, api_key: Option<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: endpoint.trim_end_matches('/').to_string(),
            api_key,
            name: None,
        }
    }

    /// Create with a friendly name for the dashboard
    pub fn connect_named(endpoint: &str, api_key: Option<String>, name: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: endpoint.trim_end_matches('/').to_string(),
            api_key,
            name: Some(name.to_string()),
        }
    }

    /// Auto-discover Hermes on common ports
    ///
    /// Tries localhost on common ports until it finds one that responds.
    /// Returns the first discovered endpoint.
    pub async fn auto_discover() -> Result<Option<DiscoveryResult>> {
        for (host, port) in COMMON_HERMES_PORTS {
            let endpoint = format!("http://{}:{}", host, port);
            let client = Self::connect(&endpoint, None);
            
            match client.health_check().await {
                Ok(info) => {
                    return Ok(Some(DiscoveryResult {
                        endpoint,
                        name: info.0,
                        model: info.1,
                        status: info.2,
                    }));
                }
                Err(_) => continue,
            }
        }
        Ok(None)
    }

    /// Health check - verifies connection and returns basic info
    ///
    /// Returns (name, model, status) tuple if Hermes is reachable.
    pub async fn health_check(&self) -> Result<(String, String, String)> {
        let url = format!("{}/api/status", self.base_url);
        let mut req = self.http.get(&url);
        
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        
        let resp = req.send().await
            .context(format!("Failed to connect to Hermes at {}", self.base_url))?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Hermes returned error status: {}", resp.status());
        }
        
        let status: HermesStatusResponse = resp.json().await
            .context("Failed to parse Hermes status response")?;
        
        let name = self.name.clone().unwrap_or_else(|| "Unknown".to_string());
        Ok((name, status.model, status.status))
    }

    /// Get agent status from Hermes
    pub async fn get_status(&self) -> Result<HermesStatusResponse> {
        let url = format!("{}/api/status", self.base_url);
        let mut req = self.http.get(&url);
        
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        
        let resp = req.send().await?;
        let status: HermesStatusResponse = resp.json().await?;
        Ok(status)
    }

    /// Send a message to Hermes and get response
    pub async fn chat(&self, message: &str) -> Result<HermesChatResponse> {
        let url = format!("{}/api/chat", self.base_url);
        let body = HermesMessage {
            message: message.to_string(),
        };
        
        let mut req = self.http.post(&url).json(&body);
        
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        
        let resp = req.send().await?;
        let chat_resp: HermesChatResponse = resp.json().await?;
        Ok(chat_resp)
    }

    /// Get token usage stats
    pub async fn get_token_usage(&self) -> Result<TokenCount> {
        let status = self.get_status().await?;
        Ok(TokenCount {
            input: status.tokens_used.input,
            output: status.tokens_used.output,
            cache: status.tokens_used.cache,
        })
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.base_url
    }

    /// Get the assigned name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Fleet manager for tracking multiple Hermes agents
#[derive(Default)]
pub struct HermesFleet {
    agents: RwLock<HashMap<String, HermesAgent>>,
}

impl HermesFleet {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Add an agent to the fleet by connecting to it
    pub async fn add_agent(&self, endpoint: &str, api_key: Option<String>, name: Option<String>) -> Result<HermesAgent> {
        let client = HermesClient::connect(endpoint, api_key);
        let (agent_name, model, status) = client.health_check().await?;
        
        let name = name.unwrap_or(agent_name);
        let id = uuid::Uuid::new_v4().to_string();
        
        let agent = HermesAgent {
            id: id.clone(),
            name: name.clone(),
            endpoint: endpoint.to_string(),
            status: parse_status(&status),
            model,
            platform: String::new(),
            tokens_today: TokenCount::default(),
            last_seen: chrono::Utc::now(),
            project: None,
        };
        
        self.register(agent.clone());
        Ok(agent)
    }

    /// Register a new agent with the fleet
    pub async fn register(&self, agent: HermesAgent) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), agent);
    }

    /// Unregister an agent
    pub async fn unregister(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
    }

    /// Get all agents
    pub async fn get_agents(&self) -> Vec<HermesAgent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Option<HermesAgent> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Get agents by project
    pub async fn get_agents_by_project(&self, project: &str) -> Vec<HermesAgent> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.project.as_deref() == Some(project))
            .cloned()
            .collect()
    }

    /// Get agents count
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Update agent status
    pub async fn update_status(&self, agent_id: &str, status: AgentStatus) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = status;
            agent.last_seen = chrono::Utc::now();
        }
    }
}

/// Parse status string to AgentStatus enum
fn parse_status(s: &str) -> AgentStatus {
    match s.to_lowercase().as_str() {
        "running" => AgentStatus::Running,
        "idle" => AgentStatus::Idle,
        "busy" => AgentStatus::Busy,
        "error" => AgentStatus::Error,
        _ => AgentStatus::Offline,
    }
}

/// Shared fleet state
pub type SharedFleet = Arc<HermesFleet>;

/// Create a new shared fleet
pub fn create_fleet() -> SharedFleet {
    Arc::new(HermesFleet::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status("running"), AgentStatus::Running);
        assert_eq!(parse_status("IDLE"), AgentStatus::Idle);
        assert_eq!(parse_status("unknown"), AgentStatus::Offline);
    }

    #[test]
    fn test_token_count_total() {
        let tc = TokenCount {
            input: 1000,
            output: 500,
            cache: 200,
        };
        assert_eq!(tc.total(), 1700);
    }
}
