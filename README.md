# Pop MCP Server

MCP server for Polkadot development with [Pop CLI](https://github.com/r0gue-io/pop-cli). Smart contracts, parachains, chain interaction, and documentation — through your AI assistant.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Installation

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

### Build from Source

```bash
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp
cargo build --release
# Binary: target/release/pop-mcp-server
```

## Configuration

Replace `/path/to/pop-mcp-server` with the actual path to your binary, or just `pop-mcp-server` if it's in your PATH.

`PRIVATE_KEY` is used to sign on-chain transactions (deploy, call, transfer). Use dev keys (`//Alice`, `//Bob`) for local networks only. Never use keys with mainnet funds.

### Claude Desktop

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "pop-mcp": {
      "type": "stdio",
      "command": "/path/to/pop-mcp-server",
      "args": [],
      "env": {
        "PRIVATE_KEY": "//Alice"
      }
    }
  }
}
```

### Claude Code

Global (`~/.claude.json`) or project-specific (`.mcp.json`):

```json
{
  "mcpServers": {
    "pop-mcp": {
      "type": "stdio",
      "command": "/path/to/pop-mcp-server",
      "args": [],
      "env": {
        "PRIVATE_KEY": "//Alice"
      }
    }
  }
}
```

### Cursor

**Settings** > **Tools & MCP** > **Add new MCP Server**:

```json
{
  "pop-mcp": {
    "type": "stdio",
    "command": "/path/to/pop-mcp-server",
    "args": [],
    "env": {
      "PRIVATE_KEY": "//Alice"
    }
  }
}
```

### Other MCP Clients

Provide `command: "pop-mcp-server"` (or full path) with `args: []` and set `PRIVATE_KEY` in `env`.

## Resources

- [Pop CLI Documentation](https://learn.onpop.io)
- [Pop CLI GitHub](https://github.com/r0gue-io/pop-cli)
- [ink! Documentation](https://use.ink)
- [Model Context Protocol](https://modelcontextprotocol.io)

## License

MIT
