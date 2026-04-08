//! Hermes Agent Integration for OpenCoWork
//!
//! Provides client functionality for connecting to Hermes agents,
//! monitoring their status, and managing tasks across the fleet.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Hermes agent client for API communication
#[derive(Clone)]
pub struct HermesClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

/// Agent status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Idle,
    Busy,
    Error,
    Offline,
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

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
    pub cache: u64,
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

impl HermesClient {
    /// Create a new Hermes client
    pub fn new(endpoint: &str, api_key: Option<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: endpoint.trim_end_matches('/').to_string(),
            api_key,
        }
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

    /// Update agent status
    pub async fn update_status(&self, agent_id: &str, status: AgentStatus) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = status;
            agent.last_seen = chrono::Utc::now();
        }
    }
}

/// Shared fleet state
pub type SharedFleet = Arc<HermesFleet>;

/// Create a new shared fleet
pub fn create_fleet() -> SharedFleet {
    Arc::new(HermesFleet::new())
}
