# Pop MCP Server

MCP server for Polkadot development with [Pop CLI](https://github.com/r0gue-io/pop-cli). Smart contract (ink!) and chain (Polkadot SDK) interaction with your AI assistant.

## Setup

### Prerequisites

- Pop CLI works on your machine:

```bash
pop --version
```

If needed, install from the official docs:
`https://learn.onpop.io/v/cli/installing-pop-cli`

### Build Pop MCP

```bash
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp
cargo build --release
```

### Claude Code (CLI)

Add the server (project scope). This creates/updates `.mcp.json`:

```bash
claude mcp add pop-mcp --scope project --env PRIVATE_KEY=//Alice -- /absolute/path/to/pop-mcp/target/release/pop-mcp-server
```

Verify:

```bash
claude mcp get pop-mcp
```

### Codex CLI

Config is stored in `~/.codex/config.toml` (or project `.codex/config.toml`).

Add the server:

```bash
codex mcp add pop-mcp --env PRIVATE_KEY=//Alice -- /absolute/path/to/pop-mcp/target/release/pop-mcp-server
```

Verify:

```bash
codex mcp list
```

### Verify Pop MCP

In Claude Code or Codex, call:

```
check_pop_installation
```

Expected: Pop CLI version is returned.

### Notes

- `PRIVATE_KEY` is only required for signing transactions. Read-only calls work without it. Use dev keys (`//Alice`, `//Bob`) for local networks only.
- You do not run the MCP server manually; the client launches it.

### Switching Keys

Use the same server name (`pop-mcp`) when you update the key.

Claude Code:

```bash
claude mcp add pop-mcp --scope project --env PRIVATE_KEY=//Bob -- /absolute/path/to/pop-mcp/target/release/pop-mcp-server
```

Codex CLI:

```bash
codex mcp add pop-mcp --env PRIVATE_KEY=//Bob -- /absolute/path/to/pop-mcp/target/release/pop-mcp-server
```
## Installation (Optional)

### Pre-built Binary

Download from [GitHub Releases](https://github.com/r0gue-io/pop-mcp/releases):

```bash
# macOS ARM64 (Apple Silicon)
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-aarch64-apple-darwin.tar.gz | tar xz

# macOS Intel
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-x86_64-apple-darwin.tar.gz | tar xz

# Linux
curl -L https://github.com/r0gue-io/pop-mcp/releases/latest/download/pop-mcp-server-x86_64-unknown-linux-gnu.tar.gz | tar xz
```

Optionally move to your PATH:

```bash
sudo mv pop-mcp-server-* /usr/local/bin/pop-mcp-server
```

## Resources

- [Pop CLI Documentation](https://learn.onpop.io)
- [Pop CLI GitHub](https://github.com/r0gue-io/pop-cli)
- [ink! Documentation](https://use.ink)
- [Model Context Protocol](https://modelcontextprotocol.io)

## License

MIT
