# tf-config

Configuration management crate for the test-framework project.

## Overview

This crate provides YAML-based configuration loading and validation for the test-framework CLI tool. It handles project configuration, integration endpoints (Jira, Squash), template paths, and LLM settings.

## Usage

```rust
use std::path::Path;
use tf_config::load_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(Path::new("config.yaml"))?;

    println!("Project: {}", config.project_name);
    println!("Output folder: {}", config.output_folder);

    // Check for optional warnings using pattern matching
    match config.check_output_folder_exists() {
        Some(warning) => eprintln!("Warning: {}", warning),
        None => println!("Output folder exists"),
    }

    Ok(())
}
```

## Configuration Schema

```yaml
# Required fields
project_name: "my-project"
output_folder: "./output"

# Optional: Jira integration
jira:
  endpoint: "https://jira.example.com"
  token: "your-api-token"  # Sensitive - redacted in logs

# Optional: Squash integration
squash:
  endpoint: "https://squash.example.com"
  username: "user"
  password: "pass"  # Sensitive - redacted in logs

# Optional: Template paths
templates:
  cr: "./templates/cr.md"
  ppt: "./templates/report.pptx"
  anomaly: "./templates/anomaly.md"

# Optional: LLM configuration
llm:
  mode: "auto"  # auto | local | cloud
  local_endpoint: "http://localhost:11434"  # Required when mode is "local"
  local_model: "mistral:7b-instruct"  # Optional: model name for local LLM
  cloud_enabled: true  # Required when mode is "cloud"
  cloud_endpoint: "https://api.openai.com/v1"  # Required when mode is "cloud"
  cloud_model: "gpt-4o-mini"  # Required when mode is "cloud"
  api_key: "sk-your-key"  # Required when mode is "cloud" - Sensitive, redacted in logs
  timeout_seconds: 120  # Optional: request timeout (default: 120)
  max_tokens: 4096  # Optional: max response tokens (default: 4096)
```

### Cloud Mode Requirements

When using `mode: "cloud"`, the following fields are **required**:
- `cloud_enabled: true` - Must be explicitly enabled
- `cloud_endpoint` - The cloud LLM API endpoint URL
- `cloud_model` - The model name to use
- `api_key` - Your API key for authentication

### Auto Mode with Cloud Enabled

When using `mode: "auto"` with `cloud_enabled: true`, the same cloud fields are **required**:
- `cloud_endpoint` - Required for cloud fallback
- `cloud_model` - Required for cloud fallback
- `api_key` - Required for cloud authentication

This ensures that when auto mode decides to use cloud LLM (e.g., when local is unavailable), all necessary configuration is present.

## Error Handling

The crate provides detailed error messages with field names and correction hints:

```rust
use tf_config::{load_config, ConfigError};

match load_config(Path::new("config.yaml")) {
    Ok(config) => { /* use config */ }
    Err(ConfigError::FileNotFound { path }) => {
        eprintln!("Config not found: {}", path.display());
    }
    Err(ConfigError::MissingField { field, hint }) => {
        eprintln!("Missing field '{}'. Expected: {}", field, hint);
    }
    Err(ConfigError::InvalidValue { field, reason, hint }) => {
        eprintln!("Invalid '{}': {}. Expected: {}", field, reason, hint);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security

Sensitive fields (tokens, passwords, API keys) are automatically redacted in debug output:

```rust
let config = load_config(path)?;
println!("{:?}", config);  // Shows [REDACTED] for sensitive fields
```

Use the `Redact` trait for explicit redaction in logging:

```rust
use tf_config::Redact;

if let Some(jira) = &config.jira {
    // redacted() returns a String with sensitive fields masked as [REDACTED]
    println!("Jira config: {}", jira.redacted());
}
```

## Validation

The crate validates:
- Required fields are present and non-empty
- URLs have valid format (scheme + host)
- LLM mode is one of: auto, local, cloud
- Local LLM endpoint is provided when mode is "local"
- Template paths have valid format (non-empty, no null bytes)

## License

MIT
