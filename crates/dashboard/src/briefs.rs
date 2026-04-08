//! Brief generator for morning and evening summaries

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike, Local};

/// Brief type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BriefType {
    Morning,
    Evening,
}

/// A generated brief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brief {
    pub brief_type: BriefType,
    pub generated_at: DateTime<Utc>,
    pub agents_summary: Vec<AgentSummary>,
    pub tasks_summary: TasksSummary,
    pub token_cost: CostSummary,
    pub inbox_count: usize,
    pub custom_messages: Vec<String>,
}

/// Agent summary for brief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub name: String,
    pub status: String,
    pub model: String,
    pub tasks_completed: usize,
    pub tokens_used: u64,
}

/// Tasks summary for brief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksSummary {
    pub completed: usize,
    pub in_progress: usize,
    pub new: usize,
    pub blocked: usize,
}

/// Cost summary for brief
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub today_cost: f64,
    pub monthly_projection: f64,
    pub by_model: Vec<ModelCost>,
}

/// Cost per model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub model: String,
    pub cost: f64,
    pub percentage: f64,
}

/// Brief generator
pub struct BriefGenerator {
    schedule_morning_hour: u32,
    schedule_evening_hour: u32,
}

impl Default for BriefGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl BriefGenerator {
    pub fn new() -> Self {
        Self {
            schedule_morning_hour: 8,  // 8 AM
            schedule_evening_hour: 18, // 6 PM
        }
    }

    /// Set morning brief time (hour in 24h format)
    pub fn set_morning_time(&mut self, hour: u32) {
        self.schedule_morning_hour = hour;
    }

    /// Set evening brief time (hour in 24h format)
    pub fn set_evening_time(&mut self, hour: u32) {
        self.schedule_evening_hour = hour;
    }

    /// Check if it's morning brief time
    pub fn is_morning_time(&self) -> bool {
        let now = Local::now();
        now.hour() == self.schedule_morning_hour
    }

    /// Check if it's evening brief time
    pub fn is_evening_time(&self) -> bool {
        let now = Local::now();
        now.hour() == self.schedule_evening_hour
    }

    /// Generate a morning brief
    pub fn generate_morning_brief(&self, agents: Vec<AgentSummary>, tasks: TasksSummary, costs: CostSummary, inbox_count: usize) -> Brief {
        Brief {
            brief_type: BriefType::Morning,
            generated_at: Utc::now(),
            agents_summary: agents,
            tasks_summary: tasks,
            token_cost: costs,
            inbox_count,
            custom_messages: vec![
                "Good morning!".to_string(),
                "Here's your morning briefing.".to_string(),
            ],
        }
    }

    /// Generate an evening brief
    pub fn generate_evening_brief(&self, agents: Vec<AgentSummary>, tasks: TasksSummary, costs: CostSummary, inbox_count: usize) -> Brief {
        Brief {
            brief_type: BriefType::Evening,
            generated_at: Utc::now(),
            agents_summary: agents,
            tasks_summary: tasks,
            token_cost: costs,
            inbox_count,
            custom_messages: vec![
                "Good evening!".to_string(),
                "Here's your daily summary.".to_string(),
            ],
        }
    }

    /// Helper to format cost
    fn format_cost(cost: f64) -> String {
        format!("{:.2}", cost)
    }

    /// Format brief as Telegram message
    pub fn format_for_telegram(&self, brief: &Brief) -> String {
        let mut msg = String::new();
        
        match brief.brief_type {
            BriefType::Morning => {
                msg.push_str("🌅 *Morning Brief*\n\n");
            },
            BriefType::Evening => {
                msg.push_str("🌙 *Evening Review*\n\n");
            },
        }
        
        // Agents
        msg.push_str("*Agents:*\n");
        for agent in &brief.agents_summary {
            let emoji = if agent.status == "running" { "🟢" } else { "⚪" };
            msg.push_str(&format!(
                "{} {} - {} ({} tokens)\n",
                emoji, agent.name, agent.status, agent.tokens_used
            ));
        }
        msg.push_str("\n");
        
        // Tasks
        msg.push_str("*Tasks:*\n");
        msg.push_str(&format!(
            "✅ Completed: {}\n🔄 In Progress: {}\n📋 New: {}\n⛔ Blocked: {}\n",
            brief.tasks_summary.completed,
            brief.tasks_summary.in_progress,
            brief.tasks_summary.new,
            brief.tasks_summary.blocked
        ));
        msg.push_str("\n");
        
        // Costs
        msg.push_str("*Token Costs:*\n");
        msg.push_str(&format!(
            "💰 Today: ${}\n📊 Monthly: ${}\n",
            Self::format_cost(brief.token_cost.today_cost),
            Self::format_cost(brief.token_cost.monthly_projection)
        ));
        
        // Inbox
        if brief.inbox_count > 0 {
            msg.push_str(&format!("\n📬 Unread: {}\n", brief.inbox_count));
        }
        
        msg
    }

    /// Format brief as HTML
    pub fn format_as_html(&self, brief: &Brief) -> String {
        let mut html = String::new();
        
        html.push_str("<div class='brief'>\n");
        
        match brief.brief_type {
            BriefType::Morning => html.push_str("<h2>🌅 Morning Brief</h2>\n"),
            BriefType::Evening => html.push_str("<h2>🌙 Evening Review</h2>\n"),
        }
        
        html.push_str("<div class='section'>\n<h3>Agents</h3>\n<ul>\n");
        for agent in &brief.agents_summary {
            html.push_str(&format!(
                "<li><span class='status {}'>{}</span> {} - {}</li>\n",
                agent.status, agent.status, agent.name, agent.model
            ));
        }
        html.push_str("</ul>\n</div>\n");
        
        html.push_str("<div class='section'>\n<h3>Tasks</h3>\n");
        html.push_str(&format!(
            "<p>✅ Done: {} | 🔄 In Progress: {} | 📋 New: {} | ⛔ Blocked: {}</p>\n",
            brief.tasks_summary.completed,
            brief.tasks_summary.in_progress,
            brief.tasks_summary.new,
            brief.tasks_summary.blocked
        ));
        html.push_str("</div>\n");
        
        html.push_str("<div class='section'>\n<h3>Costs</h3>\n");
        html.push_str(&format!(
            "<p>Today: <strong>${}</strong> | Monthly: <strong>${}</strong></p>\n",
            Self::format_cost(brief.token_cost.today_cost),
            Self::format_cost(brief.token_cost.monthly_projection)
        ));
        html.push_str("</div>\n");
        
        if brief.inbox_count > 0 {
            html.push_str(&format!("<p>📬 {} unread messages</p>\n", brief.inbox_count));
        }
        
        html.push_str("</div>\n");
        html
    }
}
