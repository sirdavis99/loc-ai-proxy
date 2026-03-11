use crate::Cli;
use tracing::{info, error};
use crate::config::Config;

pub async fn configure_provider(provider: crate::ProviderChoice) {
    use dialoguer::{Select, Input, Confirm};
    use console::style;
    
    match provider {
        crate::ProviderChoice::Opencode => {
            println!("{}", style("Configuring opencode provider").bold().green());
            
            // Auto-detect opencode
            let opencode_path = which::which("opencode")
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| {
                    println!("{} opencode not found in PATH", style("⚠").yellow());
                    None
                });
            
            let url = if let Some(path) = opencode_path {
                // Test connection to default port
                let test_url = "http://127.0.0.1:4096".to_string();
                match test_opencode_connection(&test_url).await {
                    true => {
                        println!("{} Found opencode running at {}", style("✓").green(), test_url);
                        test_url
                    }
                    false => {
                        let start = Confirm::new()
                            .with_prompt("opencode not running. Auto-start it?")
                            .default(true)
                            .interact()
                            .unwrap();
                        
                        if start {
                            match start_opencode(&path).await {
                                Ok(url) => {
                                    println!("{} Started opencode at {}", style("✓").green(), url);
                                    url
                                }
                                Err(e) => {
                                    error!("Failed to start opencode: {}", e);
                                    Input::new()
                                        .with_prompt("Enter opencode URL")
                                        .default("http://127.0.0.1:4096".to_string())
                                        .interact()
                                        .unwrap()
                                }
                            }
                        } else {
                            Input::new()
                                .with_prompt("Enter opencode URL")
                                .default("http://127.0.0.1:4096".to_string())
                                .interact()
                                .unwrap()
                        }
                    }
                }
            } else {
                Input::new()
                    .with_prompt("Enter opencode URL")
                    .default("http://127.0.0.1:4096".to_string())
                    .interact()
                    .unwrap()
            };
            
            let auto_start = Confirm::new()
                .with_prompt("Auto-start opencode if not running?")
                .default(true)
                .interact()
                .unwrap();
            
            // Save configuration
            let mut config = Config::load(None).await.unwrap_or_default();
            use crate::config::{ProviderConfig, ProviderSettings, OpencodeConfig};
            
            config.providers.insert(
                "opencode".to_string(),
                ProviderConfig {
                    enabled: true,
                    settings: ProviderSettings::Opencode(OpencodeConfig {
                        url,
                        auto_start,
                        ..Default::default()
                    }),
                },
            );
            
            if let Err(e) = config.save(None).await {
                error!("Failed to save configuration: {}", e);
            } else {
                println!("{} Configuration saved successfully!", style("✓").green());
                println!("\nYou can now start the proxy with: locaiproxy");
            }
        }
    }
}

async fn test_opencode_connection(url: &str) -> bool {
    match reqwest::get(format!("{}/global/health", url)).await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

async fn start_opencode(_path: &str) -> anyhow::Result<String> {
    // Start opencode in serve mode
    use std::process::Command;
    
    let mut child = Command::new("opencode")
        .args(["serve"])
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start opencode: {}", e))?;
    
    // Wait a bit for it to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Verify it's running
    let url = "http://127.0.0.1:4096".to_string();
    if test_opencode_connection(&url).await {
        // Detach the process
        let _ = child.wait();
        Ok(url)
    } else {
        let _ = child.kill();
        Err(anyhow::anyhow!("opencode failed to start properly"))
    }
}

pub async fn show_status() {
    use console::style;
    
    println!("{}", style("loc-ai-proxy Status").bold().underlined());
    
    let config = match Config::load(None).await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", style("No configuration found. Run 'locaiproxy configure opencode' first.").yellow());
            return;
        }
    };
    
    for (name, provider) in &config.providers {
        let status = if provider.enabled {
            match name.as_str() {
                "opencode" => {
                    if let crate::config::ProviderSettings::Opencode(cfg) = &provider.settings {
                        if test_opencode_connection(&cfg.url).await {
                            style("✓ Connected").green()
                        } else {
                            style("✗ Not connected").red()
                        }
                    } else {
                        style("✗ Invalid config").red()
                    }
                }
                _ => style("? Unknown").yellow(),
            }
        } else {
            style("⊘ Disabled").dim()
        };
        
        println!("  {}: {}", name, status);
    }
    
    println!("\n{}", style("Server Configuration:").bold());
    println!("  Port: {}", config.server.port);
    println!("  Host: {}", config.server.host);
}

pub async fn list_models() {
    use console::style;
    
    println!("{}", style("Available Models").bold().underlined());
    println!("\n{}", style("opencode models:").bold());
    
    // Try to fetch from running proxy
    match reqwest::get("http://127.0.0.1:9110/v1/models").await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                    for model in data {
                        if let Some(id) = model.get("id").and_then(|i| i.as_str()) {
                            println!("  • {}", id);
                        }
                    }
                }
            } else {
                println!("  (Proxy not running - start with 'locaiproxy')");
            }
        }
        Err(_) => {
            println!("  (Proxy not running - start with 'locaiproxy')");
        }
    }
}

pub async fn run_diagnostics() {
    use console::style;
    
    println!("{}", style("Running Diagnostics").bold().underlined());
    
    let mut all_ok = true;
    
    // Check 1: Rust version
    print!("  Rust version... ");
    if let Ok(output) = std::process::Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", style("✓").green(), version.trim());
        } else {
            println!("{}", style("✗ Not found").red());
            all_ok = false;
        }
    } else {
        println!("{}", style("✗ Not found").red());
        all_ok = false;
    }
    
    // Check 2: opencode installed
    print!("  opencode... ");
    match which::which("opencode") {
        Ok(path) => {
            println!("{} at {}", style("✓").green(), path.display());
            
            // Check if running
            print!("  opencode running... ");
            if test_opencode_connection("http://127.0.0.1:4096").await {
                println!("{}", style("✓ Yes").green());
            } else {
                println!("{}", style("✗ No (run 'opencode serve')").red());
            }
        }
        Err(_) => {
            println!("{}", style("✗ Not found in PATH").red());
            all_ok = false;
        }
    }
    
    // Check 3: Config exists
    print!("  Configuration... ");
    match Config::load(None).await {
        Ok(_) => println!("{}", style("✓ Found").green()),
        Err(_) => {
            println!("{}", style("✗ Not found (run 'locaiproxy configure opencode')").red());
            all_ok = false;
        }
    }
    
    // Check 4: Port available
    print!("  Port 9110... ");
    match tokio::net::TcpListener::bind("127.0.0.1:9110").await {
        Ok(_) => {
            println!("{}", style("✓ Available").green());
        }
        Err(_) => {
            println!("{}", style("✗ In use (is locaiproxy already running?)").yellow());
        }
    }
    
    if all_ok {
        println!("\n{}", style("All checks passed! Ready to use.").bold().green());
    } else {
        println!("\n{}", style("Some issues found. Please fix above.").bold().yellow());
    }
}
