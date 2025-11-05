# Quick Setup Guide

This guide will help you set up the Pop MCP Server for use with Claude Desktop and other MCP clients.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Step 1: Clone and Build the Server

```bash
# Clone the repository
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp

# Install dependencies and build
npm install
npm run build
```

The build artifacts will be in the `build/` directory.

## Step 2: Configure Claude Desktop

1. Locate your Claude Desktop configuration file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%/Claude/claude_desktop_config.json`
   - **Linux**: `~/.config/Claude/claude_desktop_config.json`

2. Edit the file and add the Pop MCP server configuration:

```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "node",
      "args": [
        "/Users/tomas/Programacion/POP MCP/build/index.js"
      ],
      "env": {}
    }
  }
}
```

**Important**: Use the absolute path shown above (or adjust if you moved the project).

3. Save the file and restart Claude Desktop.

## Step 3: Verify Installation

In Claude Desktop, start a new conversation and try:

```
Do you have access to Pop CLI tools?
```

Claude should respond that it has access to the Pop MCP server with tools for creating, building, testing, and deploying ink! smart contracts.

## Step 4: Test the Server

Try these example prompts:

1. **Check Pop CLI**:
   ```
   Is Pop CLI installed on my system?
   ```

2. **List Templates**:
   ```
   What contract templates are available?
   ```

3. **Access Documentation**:
   ```
   How do I create storage variables in ink!?
   ```

4. **Create a Contract** (only if you want to test):
   ```
   Create a new ERC20 contract called TestToken in /tmp
   ```

## Troubleshooting

### Server Not Showing Up

1. Check the configuration file path is correct
2. Verify the path to `build/index.js` is absolute and correct
3. Restart Claude Desktop completely (quit and reopen)
4. Check Claude Desktop logs (Help > Debug Info)

### Build Errors

If `npm install` or `npm run build` fails:

1. Ensure Node.js 18+ is installed: `node --version`
2. Delete `node_modules` and try again:
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```

### Pop CLI Not Found

If the MCP server reports Pop CLI is not installed:

1. Install Pop CLI:
   ```bash
   brew install r0gue-io/pop-cli/pop
   ```

2. Or use the `install_pop_instructions` tool through Claude

### Permission Errors

Ensure the server script is executable:

```bash
chmod +x "/Users/tomas/Programacion/POP MCP/build/index.js"
```

## Using with Other MCP Clients

### Cline (VS Code Extension)

Add to Cline's MCP settings:

```json
{
  "pop-cli": {
    "command": "node",
    "args": ["/Users/tomas/Programacion/POP MCP/build/index.js"]
  }
}
```

### Zed Editor

Add to Zed's settings:

```json
{
  "context_servers": {
    "pop-cli": {
      "command": {
        "path": "node",
        "args": ["/Users/tomas/Programacion/POP MCP/build/index.js"]
      }
    }
  }
}
```

## Development Mode

To test the server interactively during development:

```bash
npm run inspector
```

This opens a web UI where you can test tools and view responses.

## Publishing (Optional)

To make this server available globally via npm:

1. Update `package.json` with your details (author, repository, etc.)
2. Create an npm account if you don't have one
3. Login: `npm login`
4. Publish: `npm publish --access public`

Then anyone can install it with:
```bash
npm install -g @pop-cli/mcp-server
```

And use it in their config:
```json
{
  "mcpServers": {
    "pop-cli": {
      "command": "pop-mcp-server"
    }
  }
}
```

## Next Steps

- Read the full [README.md](./README.md) for detailed documentation
- Check out the [Pop CLI documentation](https://learn.onpop.io)
- Explore the `.claude/docs/` folder for comprehensive guides
- Join the Polkadot developer community

Happy building! 🚀
