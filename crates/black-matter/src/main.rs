//! Black Matter: a hello-world agent built on Infinity.
//!
//! Usage: `black-matter <message...>`
//!
//! Environment:
//! - `BLACK_MATTER_MODEL`: Bedrock model id (default: [`DEFAULT_MODEL`]).
//! - AWS region and credentials come from the standard AWS chain
//!   (`AWS_REGION`, `AWS_PROFILE`, `~/.aws/config`, ...), read by the provider.

use std::sync::Arc;

use infinity_agent_core::stores::{InMemoryConversationStore, InMemoryStateStore};
use infinity_agent_core::system::{AgentEvent, AgentSystemBuilder, StaticModel};
use infinity_provider_bedrock::BedrockProvider;
use tokio::task::LocalSet;

/// Used when `BLACK_MATTER_MODEL` is not set.
const DEFAULT_MODEL: &str = "global.anthropic.claude-sonnet-4-6";

/// Everything the program needs from its command line and environment.
#[derive(Debug, PartialEq, Eq)]
struct Config {
    model_id: String,
    message: String,
}

/// Why a [`Config`] could not be built.
#[derive(Debug, PartialEq, Eq)]
enum ConfigError {
    MissingMessage,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMessage => write!(f, "usage: black-matter <message...>"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Build a config from plain values.
    ///
    /// Taking an iterator and an `Option` (instead of reading `std::env`
    /// directly) keeps this function pure, so it is trivial to unit test.
    fn parse(
        args: impl IntoIterator<Item = String>,
        model_env: Option<String>,
    ) -> Result<Self, ConfigError> {
        // All remaining args form one message, so quoting is optional.
        let message = args.into_iter().collect::<Vec<_>>().join(" ");
        if message.trim().is_empty() {
            return Err(ConfigError::MissingMessage);
        }
        // An unset or empty variable falls back to the default.
        let model_id = model_env
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        Ok(Self { model_id, message })
    }
}

// `Box<dyn Error>` lets `?` convert any error type. Infinity's errors are
// `Send + Sync`, hence those bounds.
type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // `skip(1)` drops the program name.
    let config = Config::parse(
        std::env::args().skip(1),
        std::env::var("BLACK_MATTER_MODEL").ok(),
    )?;

    // Infinity's local agent tasks are not `Send`, so the whole system has to
    // run inside a `LocalSet`.
    LocalSet::new()
        .run_until(async {
            let provider = Arc::new(BedrockProvider::from_env());
            let model = StaticModel::new(provider, &config.model_id).await?;

            // In-memory stores: history lives only as long as this process.
            let system = AgentSystemBuilder::new_local(
                InMemoryConversationStore::new(),
                InMemoryStateStore::new(),
                model,
            )
            .start();

            let mut thread = system.thread_builder().launch().await;
            thread.send_user_text(config.message).await?;

            // Print every event until the first completion round finishes.
            while let Some(event) = thread.recv().await {
                println!("{event:?}");
                if matches!(event, AgentEvent::CompletionFinished { .. }) {
                    break;
                }
            }

            system.shutdown().await;
            Ok(())
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn joins_args_and_uses_default_model() {
        let config = Config::parse(args(&["hello", "world"]), None).unwrap();
        assert_eq!(config.message, "hello world");
        assert_eq!(config.model_id, DEFAULT_MODEL);
    }

    #[test]
    fn env_model_overrides_default_but_empty_does_not() {
        let set = Config::parse(args(&["hi"]), Some("my-model".into())).unwrap();
        assert_eq!(set.model_id, "my-model");
        let empty = Config::parse(args(&["hi"]), Some(String::new())).unwrap();
        assert_eq!(empty.model_id, DEFAULT_MODEL);
    }

    #[test]
    fn missing_message_is_an_error() {
        assert_eq!(
            Config::parse(args(&[]), None),
            Err(ConfigError::MissingMessage)
        );
    }
}
