//! Hermes Agent Installer
//!
//! A user-friendly interactive installer that guides users through
//! setting up and configuring their Hermes AI agents.

use anyhow::Result;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::fs;
use std::path::PathBuf;

/// Agent configuration template
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub telegram_enabled: bool,
    pub telegram_token: Option<String>,
    pub github_enabled: bool,
    pub github_token: Option<String>,
    pub schedule_morning: String,
    pub schedule_evening: String,
}

/// Installation step
#[derive(Debug, Clone)]
pub enum InstallStep {
    Welcome,
    AgentName,
    ChooseModel,
    ChooseProvider,
    ConfigureTelegram,
    ConfigureGitHub,
    ConfigureSchedule,
    Finalize,
}

/// Pretty print header
fn print_header(title: &str) {
    println!("\n{}", console::style("═══════════════════════════════════════════").cyan());
    println!("  {}", console::style(title).bold().cyan());
    println!("{}", console::style("═══════════════════════════════════════════").cyan());
}

/// Print info message
fn print_info(msg: &str) {
    println!("  {} {}", console::style("ℹ").cyan(), msg);
}

/// Print success message
fn print_success(msg: &str) {
    println!("  {} {}", console::style("✓").green(), msg);
}

/// Print error message
fn print_error(msg: &str) {
    println!("  {} {}", console::style("✗").red(), msg);
}

/// Available AI providers
fn get_providers() -> Vec<&'static str> {
    vec!["OpenRouter", "OpenAI", "Anthropic", "Local (llama-server)"]
}

/// Available models per provider
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
        "OpenAI" => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
        ],
        "Anthropic" => vec![
            "claude-opus-4",
            "claude-sonnet-4",
            "claude-3-5-sonnet",
            "claude-3-haiku",
        ],
        "Local" => vec![
            "llama-3.1-70b",
            "mistral-7b",
            "codellama-34b",
        ],
        _ => vec![],
    }
}

/// Model pricing info for display
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

/// Get default port for local models
fn get_default_port(model: &str) -> u16 {
    if model.contains("llama") || model.contains("mistral") || model.contains("codellama") {
        8080
    } else {
        11434
    }
}

/// Main interactive installer
pub async fn run_interactive_install() -> Result<AgentConfig> {
    print_header("Welcome to Hermes Agent Installer");
    println!();
    println!("  This wizard will help you set up your AI agent.");
    println!("  Press Ctrl+C at any time to cancel.\n");

    // Step 1: Agent name
    print_header("Step 1: Agent Identity");
    let name: String = Input::new()
        .with_prompt("What would you like to call your agent?")
        .default("hermes".to_string())
        .interact_text()?;

    // Step 2: Provider selection
    print_header("Step 2: Choose AI Provider");
    println!("  Select the AI provider you want to use:\n");
    
    let providers = get_providers();
    let provider_idx = Select::new()
        .items(&providers)
        .with_description(&[
            "Unified access to 100+ models",
            "OpenAI GPT models",
            "Anthropic Claude models", 
            "Run locally on your machine",
        ])
        .default(0)
        .interact()?;

    let provider = providers[provider_idx];

    // Step 3: Model selection
    print_header("Step 3: Choose AI Model");
    println!("  Select a model for your agent:\n");

    let models = get_models_for_provider(provider);
    let model_idx = Select::new()
        .items(&models)
        .default(0)
        .interact()?;

    let model = models[model_idx];
    
    println!();
    print_info(&format!("Model: {}", model));
    print_info(&format!("Pricing: {}", get_model_pricing(model)));

    // Step 4: API key (if not local)
    let api_key = if provider != "Local" {
        print_header("Step 4: API Configuration");
        Some(Input::new()
            .with_prompt("Enter your API key (leave empty to skip)")
            .allow_empty(true)
            .interact_text()?)
    } else {
        print_header("Step 4: Local Server Configuration");
        let port: u16 = Input::new()
            .with_prompt(&format!("Local server port (default: {})", get_default_port(model)))
            .default(get_default_port(model))
            .interact_text()?;
        None
    };

    // Step 5: Telegram
    print_header("Step 5: Telegram Integration");
    let telegram_enabled = Confirm::new()
        .with_prompt("Enable Telegram bot for this agent?")
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

    // Step 6: GitHub
    print_header("Step 6: GitHub Integration");
    let github_enabled = Confirm::new()
        .with_prompt("Enable GitHub integration for this agent?")
        .default(false)
        .interact()?;

    let github_token = if github_enabled {
        println!();
        Some(Input::new()
            .with_prompt("Enter your GitHub Personal Access Token")
            .interact_text()?)
    } else {
        None
    };

    // Step 7: Schedule
    print_header("Step 7: Briefing Schedule");
    println("  Set when you want to receive daily updates:\n");

    let morning_hour: String = Input::new()
        .with_prompt("Morning brief time (24h format, e.g. 08)")
        .default("08".to_string())
        .interact_text()?;

    let evening_hour: String = Input::new()
        .with_prompt("Evening brief time (24h format, e.g. 18)")
        .default("18".to_string())
        .interact_text()?;

    // Step 8: Finalize
    print_header("Installation Summary");
    println!();
    println!("  Agent Name:      {}", console::style(&name).bold());
    println!("  Provider:        {}", provider);
    println!("  Model:           {}", model);
    println!("  Telegram:        {}", if telegram_enabled { "Enabled" } else { "Disabled" });
    println!("  GitHub:          {}", if github_enabled { "Enabled" } else { "Disabled" });
    println!("  Morning Brief:   {}:00", morning_hour);
    println!("  Evening Brief:   {}:00", evening_hour);
    println!();

    let confirm = Confirm::new()
        .with_prompt("Ready to install?")
        .default(true)
        .interact()?;

    if !confirm {
        anyhow::bail!("Installation cancelled by user");
    }

    Ok(AgentConfig {
        name,
        model: model.to_string(),
        provider: provider.to_string(),
        api_key: api_key.filter(|k| !k.is_empty()),
        telegram_enabled,
        telegram_token,
        github_enabled,
        github_token,
        schedule_morning: morning_hour,
        schedule_evening: evening_hour,
    })
}

/// Generate config file content
pub fn generate_config(config: &AgentConfig) -> String {
    let mut yaml = String::new();
    yaml.push_str(&format!("# Hermes Agent Configuration\n"));
    yaml.push_str(&format!("# Generated by OpenCoWork Installer\n\n"));
    yaml.push_str(&format!("agent:\n"));
    yaml.push_str(&format!("  name: \"{}\"\n", config.name));
    yaml.push_str(&format!("  model: \"{}\"\n", config.model));
    yaml.push_str(&format!("  provider: \"{}\"\n", config.provider));
    
    if let Some(ref key) = config.api_key {
        yaml.push_str(&format!("  api_key: \"{}\"\n", key));
    }
    
    yaml.push_str(&format!("\n"));
    
    if config.telegram_enabled {
        yaml.push_str(&format!("telegram:\n"));
        yaml.push_str(&format!("  enabled: true\n"));
        if let Some(ref token) = config.telegram_token {
            yaml.push_str(&format!("  bot_token: \"{}\"\n", token));
        }
        yaml.push_str(&format!("\n"));
    }
    
    if config.github_enabled {
        yaml.push_str(&format!("github:\n"));
        yaml.push_str(&format!("  enabled: true\n"));
        if let Some(ref token) = config.github_token {
            yaml.push_str(&format!("  access_token: \"{}\"\n", token));
        }
        yaml.push_str(&format!("\n"));
    }
    
    yaml.push_str(&format!("schedule:\n"));
    yaml.push_str(&format!("  morning_brief: \"{}:00\"\n", config.schedule_morning));
    yaml.push_str(&format!("  evening_brief: \"{}:00\"\n", config.schedule_evening));
    
    yaml
}

/// Save config to file
pub fn save_config(config: &AgentConfig, path: &PathBuf) -> Result<()> {
    let yaml = generate_config(config);
    fs::write(path, yaml)?;
    Ok(())
}

/// Get default config directory
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hermes")
}

fn main() {
    println!();
    println!("{}", console::style("╔═══════════════════════════════════════════╗").cyan());
    println!("{}", console::style("║     Hermes Agent Installer v0.1.0         ║").cyan());
    println!("{}", console::style("╚═══════════════════════════════════════════╝").cyan());
    println!();
    
    // Run the interactive installer
    let config = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt.block_on(async {
            run_interactive_install().await
        }),
        Err(e) => {
            print_error(&format!("Failed to create runtime: {}", e));
            std::process::exit(1);
        }
    };

    let config = match config {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Installation failed: {}", e));
            std::process::exit(1);
        }
    };

    // Save config
    let config_dir = get_config_dir();
    let config_path = config_dir.join("config.yaml");
    
    match save_config(&config, &config_path) {
        Ok(_) => {
            print_success(&format!("Configuration saved to {:?}", config_path));
        },
        Err(e) => {
            print_error(&format!("Failed to save config: {}", e));
            println!("  Please create the directory and try again:");
            println!("  mkdir -p {:?}", config_dir);
        }
    }

    println!();
    println!("{}", console::style("Installation complete!").bold().green());
    println!();
    println!("Next steps:");
    println!("  1. Review your config at: {:?}", config_path);
    println!("  2. Run: cargo run --bin hermes-agent");
    println!();
}
