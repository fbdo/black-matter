# Black Matter

Black Matter is a small, from-scratch AI coding harness written in Rust as a learning project. It is built on [Infinity](https://github.com/hydro-project/infinity), the agent runtime from the Hydro project, and talks to models through Amazon Bedrock. Today it is only a hello agent: it sends one message and prints the events. The goal is to grow it, step by step, into a real coding harness.

## Prerequisites

- Rust via [rustup](https://rustup.rs). `rust-toolchain.toml` selects stable with `rustfmt` and `clippy` automatically.
- AWS credentials with [Bedrock model access](https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html) for the Anthropic model you use (any standard AWS credential source and region setting works, for example `AWS_PROFILE` and `AWS_REGION`).

## Build and run

```sh
cargo build
cargo run -- "Write a haiku about distributed systems"
```

`BLACK_MATTER_MODEL` overrides the Bedrock model id (default `global.anthropic.claude-sonnet-4-6`). Running the agent calls Bedrock and costs money. Tests and CI never do.

Checks, the same as CI:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Infinity is pinned to an exact git commit in the root `Cargo.toml`. To upgrade, change the `rev` there and run `cargo update -p infinity-agent-core`.

## Developing with AI-DLC

The project uses [AI-DLC](https://github.com/fbdo/aidlc-workflows) (the `fbdo` fork) on the [jcode](https://github.com/1jehuang/jcode) harness. It is already installed in `.jcode/` and `aidlc/`.

```sh
jcode trust grant   # once, to approve the project's hooks
jcode               # start a session in this folder
/aidlc <what you want to build>
```

Run `/aidlc --doctor` to check the setup.

## Learning path

- [ ] Read the Infinity quickstart and trace the events printed by the hello agent.
- [ ] Add a custom tool (Infinity "adding tools") and watch the model call it.
- [ ] Add a file-read tool, so the agent can look at the code it works on.
- [ ] Add a shell tool that runs commands through Infinity's sandbox crates.
- [ ] Connect external tool servers through the RAP/MCP bridge.
- [ ] Persist conversations with a durable store instead of the in-memory ones.
- [ ] Add a terminal UI (for example with ratatui) with streaming output.
- [ ] Add a file-edit tool and a permission prompt for risky actions.

## License

Apache-2.0, see [LICENSE](LICENSE).
