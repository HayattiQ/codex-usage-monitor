# cxusage

`cxusage` is a local terminal monitor for Codex usage.

It polls Codex session event logs from `~/.codex/sessions/**/*.jsonl`, extracts usage and rate-limit events, and keeps the current values visible in a terminal UI.

## Status

This is an early v1 implementation. It is local-only, single-user, and does not scrape the interactive Codex `/status` screen.

## Install

From source:

```sh
cargo install --path .
```

Planned Homebrew install:

```sh
brew tap <owner>/tools
brew install cxusage
```

## Usage

Check whether local Codex usage events are readable:

```sh
cxusage doctor
```

Start the live monitor:

```sh
cxusage watch
```

Override paths or polling interval:

```sh
cxusage --codex-dir ~/.codex --data-dir ~/.local/share/cxusage --interval 30s watch
```

`watch` exits with `q` or `Esc`.

## What It Reads

`cxusage` looks for Codex session JSONL files under:

```text
~/.codex/sessions/**/*.jsonl
```

It extracts `token_count` events and normalizes:

- 5h limit usage, remaining percent, and reset time
- weekly limit usage, remaining percent, and reset time
- plan type
- model context window
- observed timestamp

The tool stores its own history and checkpoints under the app data directory. By default this is the platform data directory for `cxusage`.

## Commands

```text
cxusage watch
cxusage doctor
```

Global flags:

```text
--codex-dir <path>    Codex config/data directory, defaults to ~/.codex
--data-dir <path>     cxusage app data directory
--interval <duration> Poll interval, defaults to 30s
```

Durations support `s`, `m`, and `h` suffixes.

## Development

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Homebrew Formula

A formula template is kept at `packaging/homebrew/cxusage.rb`. Release automation should replace the version and SHA256 values before publishing to `<owner>/homebrew-tools`.
