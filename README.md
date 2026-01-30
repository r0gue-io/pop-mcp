# Pop MCP Server

MCP server for Pop CLI.

## Install

### Prebuilt binary

Download the latest release for your platform and extract it from GitHub Releases:

- https://github.com/r0gue-io/pop-mcp/releases

Then:

```bash
chmod +x pop-mcp-server-*
# optional: move to PATH
sudo mv pop-mcp-server-* /usr/local/bin/pop-mcp-server
```

### Build from source

```bash
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp
cargo build --release
```

Binary output:

- `target/release/pop-mcp-server`
