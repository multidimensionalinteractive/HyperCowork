//! Token Cost Estimator for OpenCoWork
//!
//! Tracks token usage across agents and calculates costs based on
//! model pricing. Supports per-agent, per-model, and aggregate reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Datelike};

/// Model pricing info (price per 1M tokens)
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_price: f64,    // per 1M input tokens
    pub output_price: f64,    // per 1M output tokens
    pub cache_price: f64,    // per 1M cached tokens
    pub currency: &'static str,
}

impl ModelPricing {
    pub fn new(input: f64, output: f64, cache: f64) -> Self {
        Self {
            input_price: input,
            output_price: output,
            cache_price: cache,
            currency: "USD",
        }
    }
}

/// Default model pricing (can be overridden)
pub fn default_pricing() -> HashMap<String, ModelPricing> {
    let mut pricing = HashMap::new();
    
    // Xiaomi MiMo models
    pricing.insert("xiaomi/mimo-v2-pro".to_string(), ModelPricing::new(1.00, 1.00, 0.10));
    pricing.insert("xiaomi/mimo-v2-flash".to_string(), ModelPricing::new(0.09, 0.09, 0.01));
    pricing.insert("xiaomi/mimo-v2-omni".to_string(), ModelPricing::new(0.40, 0.40, 0.04));
    
    // MiniMax
    pricing.insert("minimax/minimax-m2.7".to_string(), ModelPricing::new(0.30, 0.30, 0.03));
    
    // Qwen models
    pricing.insert("qwen/qwen3-25-32b".to_string(), ModelPricing::new(0.22, 0.22, 0.02));
    pricing.insert("qwen/qwen3-1m".to_string(), ModelPricing::new(0.07, 0.07, 0.007));
    
    // Gemma
    pricing.insert("google/gemma-4-31b".to_string(), ModelPricing::new(0.14, 0.14, 0.01));
    pricing.insert("google/gemma-4-26b".to_string(), ModelPricing::new(0.10, 0.10, 0.01));
    
    // Llama
    pricing.insert("meta-llama/llama-4-maverick".to_string(), ModelPricing::new(0.15, 0.15, 0.015));
    
    // OpenAI (for reference)
    pricing.insert("openai/gpt-4o".to_string(), ModelPricing::new(5.00, 15.00, 0.00));
    pricing.insert("openai/gpt-4o-mini".to_string(), ModelPricing::new(0.15, 0.60, 0.00));
    
    // Anthropic (for reference)
    pricing.insert("anthropic/claude-opus-4".to_string(), ModelPricing::new(15.00, 75.00, 0.00));
    pricing.insert("anthropic/claude-sonnet-4".to_string(), ModelPricing::new(3.00, 15.00, 0.00));
    
    pricing
}

/// Token usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    pub agent_id: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub timestamp: DateTime<Utc>,
}

/// Daily cost summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub total_tokens: u64,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_cost: f64,
    pub total_cost: f64,
    pub by_agent: HashMap<String, AgentCost>,
}

/// Per-agent cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCost {
    pub agent_id: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_cost: f64,
    pub total_cost: f64,
}

/// Monthly projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyProjection {
    pub month: String,
    pub daily_avg_cost: f64,
    pub projected_monthly: f64,
    pub daily_records: Vec<DailyCost>,
}

/// Cost estimator core
pub struct CostEstimator {
    pricing: RwLock<HashMap<String, ModelPricing>>,
    daily_records: RwLock<HashMap<String, Vec<TokenRecord>>>,
}

impl Default for CostEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl CostEstimator {
    pub fn new() -> Self {
        Self {
            pricing: RwLock::new(default_pricing()),
            daily_records: RwLock::new(HashMap::new()),
        }
    }

    /// Set custom pricing for a model
    pub fn set_pricing(&self, model: &str, pricing: ModelPricing) {
        let mut p = self.pricing.write();
        p.insert(model.to_lowercase(), pricing);
    }

    /// Record token usage for an agent
    pub fn record_tokens(&self, agent_id: &str, model: &str, input: u64, output: u64, cache: u64) {
        let record = TokenRecord {
            agent_id: agent_id.to_string(),
            model: model.to_string(),
            input_tokens: input,
            output_tokens: output,
            cache_tokens: cache,
            timestamp: Utc::now(),
        };
        
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut records = self.daily_records.write();
        records
            .entry(today)
            .or_insert_with(Vec::new)
            .push(record);
    }

    /// Calculate cost for given token counts
    pub fn calculate_cost(&self, model: &str, input: u64, output: u64, cache: u64) -> AgentCost {
        let pricing = self.pricing.read();
        let p = pricing.get(&model.to_lowercase()).cloned().unwrap_or_else(|| {
            // Default pricing if model not found
            ModelPricing::new(0.10, 0.10, 0.01)
        });
        
        let input_cost = (input as f64 / 1_000_000.0) * p.input_price;
        let output_cost = (output as f64 / 1_000_000.0) * p.output_price;
        let cache_cost = (cache as f64 / 1_000_000.0) * p.cache_price;
        
        AgentCost {
            agent_id: String::new(), // Filled by caller
            model: model.to_string(),
            input_tokens: input,
            output_tokens: output,
            cache_tokens: cache,
            input_cost,
            output_cost,
            cache_cost,
            total_cost: input_cost + output_cost + cache_cost,
        }
    }

    /// Get today's cost summary
    pub fn get_today_cost(&self) -> DailyCost {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.get_daily_cost(&today)
    }

    /// Get daily cost by date string
    pub fn get_daily_cost(&self, date: &str) -> DailyCost {
        let records = self.daily_records.read();
        let day_records = records.get(date).cloned().unwrap_or_default();
        
        let mut by_agent: HashMap<String, Vec<&TokenRecord>> = HashMap::new();
        for record in &day_records {
            by_agent
                .entry(record.agent_id.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        let mut agent_costs = HashMap::new();
        let mut total_input: u64 = 0;
        let mut total_output: u64 = 0;
        let mut total_cache: u64 = 0;
        let mut total_input_cost: f64 = 0.0;
        let mut total_output_cost: f64 = 0.0;
        let mut total_cache_cost: f64 = 0.0;
        
        let pricing = self.pricing.read();
        
        for (agent_id, recs) in &by_agent {
            let mut input: u64 = 0;
            let mut output: u64 = 0;
            let mut cache: u64 = 0;
            
            for rec in recs {
                input += rec.input_tokens;
                output += rec.output_tokens;
                cache += rec.cache_tokens;
            }
            
            let p = pricing.get(&recs.first().unwrap().model.to_lowercase()).cloned().unwrap_or_else(|| ModelPricing::new(0.10, 0.10, 0.01));
            
            let input_cost = (input as f64 / 1_000_000.0) * p.input_price;
            let output_cost = (output as f64 / 1_000_000.0) * p.output_price;
            let cache_cost = (cache as f64 / 1_000_000.0) * p.cache_price;
            
            total_input += input;
            total_output += output;
            total_cache += cache;
            total_input_cost += input_cost;
            total_output_cost += output_cost;
            total_cache_cost += cache_cost;
            
            agent_costs.insert(agent_id.clone(), AgentCost {
                agent_id: agent_id.clone(),
                model: recs.first().unwrap().model.clone(),
                input_tokens: input,
                output_tokens: output,
                cache_tokens: cache,
                input_cost,
                output_cost,
                cache_cost,
                total_cost: input_cost + output_cost + cache_cost,
            });
        }
        
        let total_cost = total_input_cost + total_output_cost + total_cache_cost;
        
        DailyCost {
            date: date.to_string(),
            total_tokens: total_input + total_output + total_cache,
            input_cost: total_input_cost,
            output_cost: total_output_cost,
            cache_cost: total_cache_cost,
            total_cost,
            by_agent: agent_costs,
        }
    }

    /// Get monthly projection
    pub fn get_monthly_projection(&self) -> MonthlyProjection {
        let now = Utc::now();
        let month = now.format("%Y-%m").to_string();
        
        let records = self.daily_records.read();
        let days_in_month = now.num_days_from_ce() % 30 + 1;
        
        let mut daily_costs: Vec<DailyCost> = Vec::new();
        let mut total_cost: f64 = 0.0;
        
        for record in records.values() {
            if !record.is_empty() {
                let date = record.first().unwrap().timestamp.format("%Y-%m-%d").to_string();
                let cost = self.get_daily_cost(&date);
                total_cost += cost.total_cost;
                daily_costs.push(cost);
            }
        }
        
        let daily_avg = if days_in_month > 0 {
            total_cost / days_in_month as f64
        } else {
            0.0
        };
        
        MonthlyProjection {
            month,
            daily_avg_cost: daily_avg,
            projected_monthly: daily_avg * 30.0,
            daily_records: daily_costs,
        }
    }
}

/// Shared cost estimator
pub type SharedCostEstimator = Arc<CostEstimator>;

/// Create a new shared cost estimator
pub fn create_cost_estimator() -> SharedCostEstimator {
    Arc::new(CostEstimator::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost() {
        let estimator = CostEstimator::new();
        
        // 1M input tokens at $1/M = $1
        let cost = estimator.calculate_cost("xiaomi/mimo-v2-pro", 1_000_000, 0, 0);
        assert!((cost.input_cost - 1.0).abs() < 0.01);
        
        // Mixed tokens
        let cost = estimator.calculate_cost("xiaomi/mimo-v2-flash", 1_000_000, 500_000, 2_000_000);
        // MiMo-V2-Flash: $0.09/M input, $0.09/M output, $0.01/M cache
        assert!((cost.input_cost - 0.09).abs() < 0.001);
        assert!((cost.output_cost - 0.045).abs() < 0.001);
        assert!((cost.cache_cost - 0.02).abs() < 0.001);
    }
}
