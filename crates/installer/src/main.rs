//! Hermes Agent Installer
//!
//! A user-friendly interactive installer that guides users through:
//! 1. Setting up a new Hermes agent, OR
//! 2. Connecting to an existing Hermes instance

use anyhow::Result;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::PathBuf;

/// Agent configuration template
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub hermes_endpoint: Option<String>,
    pub telegram_enabled: bool,
    pub telegram_token: Option<String>,
    pub github_enabled: bool,
    pub github_token: Option<String>,
    pub schedule_morning: String,
    pub schedule_evening: String,
}

/// Installation mode
#[derive(Debug, Clone, Copy)]
pub enum InstallMode {
    NewAgent,
    ConnectExisting,
}

fn print_header(title: &str) {
    println!("\n{}", console::style("═══════════════════════════════════════════").cyan());
    println!("  {}", console::style(title).bold().cyan());
    println!("{}", console::style("═══════════════════════════════════════════").cyan());
}

fn print_info(msg: &str) {
    println!("  {} {}", console::style("ℹ").cyan(), msg);
}

fn print_success(msg: &str) {
    println!("  {} {}", console::style("✓").green(), msg);
}

fn print_error(msg: &str) {
    println!("  {} {}", console::style("✗").red(), msg);
}

fn get_providers() -> Vec<&'static str> {
    vec!["OpenRouter", "OpenAI", "Anthropic", "Local (llama-server)"]
}

fn get_models_for_provider(provider: &str) -> Vec<&'static str> {
    match provider {
        "OpenRouter" => vec![
            "xiaomi/mimo-v2-pro",
            "xiaomi/mimo-v2-flash",
            "xiaomi/mimo-v2-omni",
            "minimax/minimax-m2.7",
            "qwen/qwen3-25-32b",
            "google/gemma-4-31b",
            "google/gemma-4-26b",
            "meta-llama/llama-4-maverick",
        ],
        "OpenAI" => vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
        "Anthropic" => vec!["claude-opus-4", "claude-sonnet-4", "claude-3-5-sonnet", "claude-3-haiku"],
        "Local" => vec!["llama-3.1-70b", "mistral-7b", "codellama-34b"],
        _ => vec![],
    }
}

fn get_model_pricing(model: &str) -> &'static str {
    match model {
        "xiaomi/mimo-v2-pro" => "$1.00/1M tokens",
        "xiaomi/mimo-v2-flash" => "$0.09/1M tokens",
        "xiaomi/mimo-v2-omni" => "$0.40/1M tokens",
        "minimax/minimax-m2.7" => "$0.30/1M tokens",
        "qwen/qwen3-25-32b" => "$0.22/1M tokens",
        "google/gemma-4-31b" => "$0.14/1M tokens",
        "google/gemma-4-26b" => "$0.10/1M tokens",
        "meta-llama/llama-4-maverick" => "$0.15/1M tokens",
        "gpt-4o" => "$5.00/1M input, $15.00/1M output",
        "gpt-4o-mini" => "$0.15/1M input, $0.60/1M output",
        "claude-opus-4" => "$15.00/1M input, $75.00/1M output",
        "claude-sonnet-4" => "$3.00/1M input, $15.00/1M output",
        _ => "Varies",
    }
}

/// Try to auto-discover Hermes on common ports
async fn try_discover_hermes() -> Option<String> {
    use opencowork_hermes::HermesClient;
    
    let ports = [(8080, "localhost"), (8081, "localhost"), (3000, "localhost")];
    
    for (port, host) in ports {
        let endpoint = format!("http://{}:{}", host, port);
        let client = HermesClient::connect(&endpoint, None);
        
        match client.health_check().await {
            Ok((name, model, _)) => {
                println!("  {} Discovered: {} at {} (model: {})", 
                    console::style("✓").green(), name, endpoint, model);
                return Some(endpoint);
            }
            Err(_) => continue,
        }
    }
    None
}

/// Run the mode selection step
fn select_install_mode() -> InstallMode {
    print_header("Setup Mode");
    println!("\n  How would you like to get started?\n");
    println!("  [0] 🚀 Connect to existing Hermes");
    println!("      Already have a Hermes bot? Just point us at it.");
    println!("  [1] ✨ Create a new Hermes agent");
    println!("      Set up a fresh agent with a new model.\n");
    
    let idx = Select::new()
        .items(&["Connect to existing Hermes", "Create new agent"])
        .default(0)
        .interact()
        .unwrap_or(0);
    
    if idx == 0 {
        InstallMode::ConnectExisting
    } else {
        InstallMode::NewAgent
    }
}

/// Run the connect-existing flow
async fn run_connect_existing() -> Result<AgentConfig> {
    print_header("Connect to Existing Hermes");
    println!("\n  Let's find your Hermes instance.\n");
    
    // Try auto-discovery first
    print_info("Trying to auto-discover Hermes on common ports...");
    
    let discovered_endpoint = try_discover_hermes().await;
    
    let endpoint = if let Some(ep) = discovered_endpoint {
        println!();
        let use_discovered = Confirm::new()
            .with_prompt(&format!("Found Hermes at {}. Use this?", ep))
            .default(true)
            .interact()?;
        
        if use_discovered {
            ep
        } else {
            Input::new()
                .with_prompt("Enter your Hermes endpoint URL")
                .default("http://localhost:8080".to_string())
                .interact_text()?
        }
    } else {
        print_info("No Hermes found on common ports.");
        Input::new()
            .with_prompt("Enter your Hermes endpoint URL")
            .default("http://localhost:8080".to_string())
            .interact_text()?
    };
    
    // Verify connection
    print_info("Verifying connection...");
    let client = opencowork_hermes::HermesClient::connect(&endpoint, None);
    
    match client.health_check().await {
        Ok((name, model, status)) => {
            print_success(&format!("Connected! Agent: {} ({}), Status: {}", name, model, status));
        }
        Err(e) => {
            print_error(&format!("Could not connect: {}", e));
            println!("  Make sure Hermes is running and the URL is correct.");
        }
    }
    
    // Name this connection
    let name: String = Input::new()
        .with_prompt("Give this agent a friendly name")
        .default("my-hermes".to_string())
        .interact_text()?;
    
    // Telegram setup
    print_header("Telegram Integration");
    let telegram_enabled = Confirm::new()
        .with_prompt("Enable Telegram bot for notifications?")
        .default(true)
        .interact()?;
    
    let telegram_token = if telegram_enabled {
        println!();
        Some(Input::new()
            .with_prompt("Enter your Telegram bot token")
            .interact_text()?)
    } else {
        None
    };
    
    // Schedule
    print_header("Briefing Schedule");
    let morning_hour: String = Input::new()
        .with_prompt("Morning brief time (24h format)")
        .default("08".to_string())
        .interact_text()?;
    
    let evening_hour: String = Input::new()
        .with_prompt("Evening brief time (24h format)")
        .default("18".to_string())
        .interact_text()?;
    
    // Summary
    print_header("Connection Summary");
    println!();
    println!("  Hermes Endpoint:  {}", console::style(&endpoint).bold());
    println!("  Agent Name:       {}", console::style(&name).bold());
    println!("  Telegram:         {}", if telegram_enabled { "Enabled" } else { "Disabled" });
    println!("  Morning Brief:    {}:00", morning_hour);
    println!("  Evening Brief:    {}:00", evening_hour);
    println!();
    
    let confirm = Confirm::new()
        .with_prompt("Save this configuration?")
        .default(true)
        .interact()?;
    
    if !confirm {
        anyhow::bail!("Setup cancelled");
    }
    
    Ok(AgentConfig {
        name,
        model: String::new(), // Will be fetched from Hermes
        provider: String::new(),
        api_key: None,
        hermes_endpoint: Some(endpoint),
        telegram_enabled,
        telegram_token,
        github_enabled: false,
        github_token: None,
        schedule_morning: morning_hour,
        schedule_evening: evening_hour,
    })
}

/// Run the new agent setup flow
async fn run_new_agent_setup() -> Result<AgentConfig> {
    print_header("Create New Hermes Agent");
    println!();
    println!("  Set up a fresh agent with your choice of model.\n");
    
    // Agent name
    let name: String = Input::new()
        .with_prompt("What would you like to call your agent?")
        .default("hermes".to_string())
        .interact_text()?;
    
    // Provider
    print_header("Choose AI Provider");
    let providers = get_providers();
    println!("  [0] OpenRouter - Unified access to 100+ models (recommended)");
    println!("  [1] OpenAI - OpenAI GPT models");
    println!("  [2] Anthropic - Anthropic Claude models");
    println!("  [3] Local - Run locally on your machine\n");
    
    let provider_idx = Select::new()
        .items(&providers)
        .default(0)
        .interact()?;
    let provider = providers[provider_idx];
    
    // Model
    print_header("Choose AI Model");
    let models = get_models_for_provider(provider);
    let model_idx = Select::new()
        .items(&models)
        .default(0)
        .interact()?;
    let model = models[model_idx];
    
    println!();
    print_info(&format!("Model: {}", model));
    print_info(&format!("Pricing: {}", get_model_pricing(model)));
    
    // API key
    let api_key = if provider != "Local" {
        print_header("API Configuration");
        Some(Input::new()
            .with_prompt("Enter your API key")
            .interact_text()?)
    } else {
        print_header("Local Server Configuration");
        Input::new()
            .with_prompt("Local server port (default: 8080)")
            .default("8080".to_string())
            .interact_text()?;
        None
    };
    
    // Telegram
    print_header("Telegram Integration");
    let telegram_enabled = Confirm::new()
        .with_prompt("Enable Telegram bot?")
        .default(true)
        .interact()?;
    
    let telegram_token = if telegram_enabled {
        Some(Input::new()
            .with_prompt("Enter your Telegram bot token")
            .interact_text()?)
    } else {
        None
    };
    
    // GitHub
    print_header("GitHub Integration");
    let github_enabled = Confirm::new()
        .with_prompt("Enable GitHub integration?")
        .default(false)
        .interact()?;
    
    let github_token = if github_enabled {
        Some(Input::new()
            .with_prompt("Enter your GitHub Personal Access Token")
            .interact_text()?)
    } else {
        None
    };
    
    // Schedule
    print_header("Briefing Schedule");
    let morning_hour: String = Input::new()
        .with_prompt("Morning brief time (24h format)")
        .default("08".to_string())
        .interact_text()?;
    
    let evening_hour: String = Input::new()
        .with_prompt("Evening brief time (24h format)")
        .default("18".to_string())
        .interact_text()?;
    
    // Summary
    print_header("Setup Summary");
    println!();
    println!("  Agent Name:       {}", console::style(&name).bold());
    println!("  Provider:         {}", provider);
    println!("  Model:            {}", model);
    println!("  Telegram:         {}", if telegram_enabled { "Enabled" } else { "Disabled" });
    println!("  GitHub:           {}", if github_enabled { "Enabled" } else { "Disabled" });
    println!("  Morning Brief:    {}:00", morning_hour);
    println!("  Evening Brief:   {}:00", evening_hour);
    println!();
    
    let confirm = Confirm::new()
        .with_prompt("Ready to install?")
        .default(true)
        .interact()?;
    
    if !confirm {
        anyhow::bail!("Setup cancelled");
    }
    
    Ok(AgentConfig {
        name,
        model: model.to_string(),
        provider: provider.to_string(),
        api_key,
        hermes_endpoint: None,
        telegram_enabled,
        telegram_token,
        github_enabled,
        github_token,
        schedule_morning: morning_hour,
        schedule_evening: evening_hour,
    })
}

/// Generate config file
pub fn generate_config(config: &AgentConfig) -> String {
    let mut yaml = String::new();
    yaml.push_str("# OpenCoWork Configuration\n");
    yaml.push_str("# Generated by opencowork-installer\n\n");
    
    if let Some(ref endpoint) = config.hermes_endpoint {
        yaml.push_str("# Connected to existing Hermes\n");
        yaml.push_str(&format!("hermes:\n"));
        yaml.push_str(&format!("  endpoint: \"{}\"\n", endpoint));
        yaml.push_str(&format!("  name: \"{}\"\n", config.name));
        yaml.push_str("\n");
    } else {
        yaml.push_str(&format!("agent:\n"));
        yaml.push_str(&format!("  name: \"{}\"\n", config.name));
        yaml.push_str(&format!("  model: \"{}\"\n", config.model));
        yaml.push_str(&format!("  provider: \"{}\"\n", config.provider));
        if let Some(ref key) = config.api_key {
            yaml.push_str(&format!("  api_key: \"{}\"\n", key));
        }
        yaml.push_str("\n");
    }
    
    if config.telegram_enabled {
        yaml.push_str("telegram:\n");
        yaml.push_str("  enabled: true\n");
        if let Some(ref token) = config.telegram_token {
            yaml.push_str(&format!("  bot_token: \"{}\"\n", token));
        }
        yaml.push_str("\n");
    }
    
    if config.github_enabled {
        yaml.push_str("github:\n");
        yaml.push_str("  enabled: true\n");
        if let Some(ref token) = config.github_token {
            yaml.push_str(&format!("  access_token: \"{}\"\n", token));
        }
        yaml.push_str("\n");
    }
    
    yaml.push_str("schedule:\n");
    yaml.push_str(&format!("  morning_brief: \"{}:00\"\n", config.schedule_morning));
    yaml.push_str(&format!("  evening_brief: \"{}:00\"\n", config.schedule_evening));
    
    yaml
}

pub fn save_config(config: &AgentConfig, path: &PathBuf) -> Result<()> {
    let yaml = generate_config(config);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, yaml)?;
    Ok(())
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencowork")
}

#[tokio::main]
async fn main() {
    println!();
    println!("{}", console::style("╔═══════════════════════════════════════════╗").cyan());
    println!("{}", console::style("║     OpenCoWork Installer v0.1.0          ║").cyan());
    println!("{}", console::style("╚═══════════════════════════════════════════╝").cyan());
    println!();
    
    let mode = select_install_mode();
    
    let config = match mode {
        InstallMode::ConnectExisting => {
            run_connect_existing().await
        }
        InstallMode::NewAgent => {
            run_new_agent_setup().await
        }
    };
    
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Setup failed: {}", e));
            std::process::exit(1);
        }
    };
    
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.yaml");
    
    match save_config(&config, &config_path) {
        Ok(_) => {
            print_success(&format!("Configuration saved to {:?}", config_path));
        }
        Err(e) => {
            print_error(&format!("Failed to save config: {}", e));
        }
    }
    
    println!();
    println!("{}", console::style("Setup complete!").bold().green());
    println!();
    println!("Next steps:");
    println!("  1. Review config: cat {:?}", config_path);
    println!("  2. Run: cargo run -p opencowork-server -- --config {:?}", config_path);
    println!();
}
