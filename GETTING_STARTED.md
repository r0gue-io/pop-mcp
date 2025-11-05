# Getting Started with Pop MCP Server

Welcome! This guide will get you up and running with the Pop MCP Server in just a few minutes.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## What is This?

Pop MCP Server connects your AI assistant (like Claude) to Pop CLI, giving you superpowers for **complete Polkadot development**. Just chat with your AI to:

- **Smart Contracts**: Create, build, test, and deploy ink! contracts
- **Parachains**: Build and launch custom parachains/appchains
- **Pallets**: Create custom runtime modules
- **Chain Interaction**: Call extrinsics, query storage, read constants
- **Documentation**: Access comprehensive Polkadot, ink!, and XCM guides
- And much more!

## Installation Options

### Option A: Install from NPM (Recommended)

```bash
npm install -g @pop-cli/mcp-server
```

Then configure Claude Desktop with just:

```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

### Option B: Build from Source

If installing from source or for development:

#### Step 1: Clone and Build (2 minutes)

```bash
# Clone the repository
git clone https://github.com/r0gue-io/pop-mcp.git
cd pop-mcp

# Install dependencies and build
npm install
npm run build
```

You should see:
```
✅ Build output exists
```

#### Step 2: Configure Claude Desktop (2 minutes)

1. Open Claude Desktop configuration:
   ```bash
   # macOS
   open ~/Library/Application\ Support/Claude/claude_desktop_config.json

   # Windows
   # Open: %APPDATA%\Claude\claude_desktop_config.json

   # Linux
   # Open: ~/.config/Claude/claude_desktop_config.json
   ```

   Or create if it doesn't exist:
   ```bash
   # macOS
   mkdir -p ~/Library/Application\ Support/Claude
   touch ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

2. Add this configuration (replace with your actual absolute path):
   ```json
   {
     "mcpServers": {
       "pop-cli": {
         "type": "stdio",
         "command": "node",
         "args": [
           "/absolute/path/to/pop-mcp/build/index.js"
         ],
         "env": {}
       }
     }
   }
   ```

3. Save and **restart Claude Desktop** (important!)

### Option C: Use with Claude Code in VS Code

You can also use this MCP server directly in VS Code with the Claude Code extension!

1. **Install the Claude Code extension** in VS Code

2. **Configure in Global Claude Settings**:
   Edit `~/.claude.json` (create if it doesn't exist):

For globally installed:
```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "pop-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

For local build:
```json
{
  "mcpServers": {
    "pop-cli": {
      "type": "stdio",
      "command": "node",
      "args": [
        "/absolute/path/to/pop-mcp/build/index.js"
      ],
      "env": {}
    }
  }
}
```

**Or** add a project-specific `.mcp.json` file in your project directory with the same format.

3. **Restart Claude Code** (or reload VS Code window)

4. **Verify** with `/mcp` command in Claude Code

Now you can develop Polkadot projects with Claude's help right in your IDE!

## Step 3: Test It Works (2 minutes)

Open Claude Desktop (or Claude Code in VS Code) and try:

```
Do you have access to Pop CLI tools?
```

Claude should respond that it has access to the Pop MCP server!

Then try:

```
Is Pop CLI installed?
```

Claude should check and report that Pop CLI 0.11.0 is installed.

### Step 4: Try It Out!

Now you can use natural language for all Polkadot development tasks:

#### Smart Contracts
```
Create a new ERC20 token contract called MyToken
```

#### Parachains
```
Create a new parachain with contracts support
```

#### Chain Interaction
```
Check the balance of account 5GrwvaEF... on Polkadot
```

## What Can You Do?

### Smart Contract Development

```
What contract templates are available?
Create an NFT contract called CoolNFT
Build the contract with optimizations
Run e2e tests
Deploy to a local node
```

### Parachain Development

```
Create a new parachain with asset management
Build my parachain for testnet
Launch a local parachain network
Benchmark my pallets
```

### Pallet Development

```
Create a new custom pallet called rewards
Add my pallet to the runtime
```

### Chain Interaction

```
Query storage for System.Account on Polkadot
Call Balances.transfer on my local chain
Read the runtime version constant
Execute a sudo call
```

### Documentation & Learning

The MCP server includes comprehensive documentation that Claude can access automatically:

```
How do I define storage in an ink! contract?
Explain XCM and how to use it
What's the difference between contracts and parachains?
Show me examples of cross-contract calls
Search the docs for "testing patterns"
What are the best practices for gas optimization?
```

## How to Access Documentation

The Pop MCP Server includes comprehensive Polkadot documentation. Claude can access it in **three ways**:

### 1. 🎯 Just Ask (Easiest!)

Claude automatically has access to all docs. Simply ask questions:

```
How do I implement storage in ink!?
What are the testing best practices?
Explain how XCM works
Show me an example of cross-contract calls
```

### 2. 🔍 Search Specific Topics

Use the search tool for targeted queries:

```
Search the documentation for "storage macros"
Find examples of ERC20 implementations
Look up XCM integration patterns
```

### 3. 📚 Direct Resource Access

Claude can read specific documentation files:
- `ink://docs/llm-guide` - Complete ink! reference
- `ink://docs/technical-guide` - Technical implementation details
- `pop://docs/cli-guide` - Pop CLI documentation
- `xcm://docs/comprehensive-guide` - XCM theory and patterns
- `xcm://docs/ink-examples` - Real XCM contract examples

### Available Documentation

Your AI assistant has comprehensive knowledge about:

1. **ink! Smart Contracts** - Complete language reference and patterns
2. **Pop CLI** - All tooling for contracts, parachains, and pallets
3. **XCM** - Cross-chain messaging theory and implementation
4. **Technical Guides** - Deep implementation details
5. **Code Examples** - Real-world contracts and XCM integrations

Just ask about anything related to Polkadot, Substrate, or Web3 development!

## Next Steps

### Learn More

- **README.md** - Complete feature documentation
- **EXAMPLES.md** - Detailed usage examples
- **QUICK_REFERENCE.md** - Command cheat sheet

### Try These Workflows

1. **Smart Contract Project**
   ```
   Create an ERC20 token, build it, test it, and deploy locally
   ```

2. **Parachain Project**
   ```
   Create a parachain with contracts support, build it, and launch locally
   ```

3. **Chain Interaction**
   ```
   Query the total issuance on Polkadot mainnet
   ```

4. **Learning & Exploration**
   ```
   Explain the difference between building a contract vs a parachain
   What are the available parachain templates?
   Show me how to use XCM in a contract
   ```

## Troubleshooting

### Claude Doesn't See the Server

1. Check configuration file path is correct
2. Verify absolute path to `build/index.js`
3. **Restart Claude Desktop completely** (quit and reopen)
4. Check for typos in JSON (trailing commas, quotes, etc.)

### "Pop CLI not found"

The server can still help! Ask:

```
I need to install Pop CLI. Can you help?
```

Claude will provide installation instructions.

### Build Errors

```bash
# Clean and rebuild
rm -rf node_modules build
npm install
npm run build
```

### Need Help?

See **TEST.md** for comprehensive troubleshooting.

## Tips for Best Results

1. **Be Specific**: "Create an ERC20 contract called MyToken in /tmp"
2. **Ask for Explanations**: "Explain storage in ink!"
3. **Complete Workflows**: "Create, build, test, and deploy an NFT contract"
4. **Use Absolute Paths**: `/full/path/to/contract` works better than `./contract`

## Common First Projects

### Simple Token

```
Create a new ERC20 token contract called SimpleToken.
Set it up in /tmp/my-contracts.
```

### NFT Collection

```
I want to create an ERC721 NFT collection.
What's the best approach?
```

### DNS System

```
Create a domain name service contract from the DNS template.
Explain how it works.
```

## Example Conversation

```
You: Is Pop CLI installed?
Claude: [Checks] Yes, Pop CLI 0.11.0 is installed.

You: Great! Create a new ERC20 token called HackToken
Claude: [Creates contract] I've created HackToken in ./HackToken

You: Build it with optimizations
Claude: [Builds with --release] Build successful! The optimized contract is ready.

You: How do I add a minting function?
Claude: [Reads ink! docs] Here's how to add a mint function...
```

## For Hackathons

This tool is perfect for hackathons! You can:

- ⚡ **Rapid Development**: From idea to deployed solution in minutes
- 🎯 **Full Stack**: Build contracts, parachains, or both
- 📚 **Learn as You Go**: Ask questions, get instant answers
- 🔧 **Complete Toolkit**: Create, build, test, deploy - all through chat
- 📖 **Comprehensive Docs**: All Polkadot/Substrate knowledge at your fingertips
- 🚀 **Focus on Innovation**: Let AI handle the boilerplate and tooling

## What Makes This Special?

- **Natural Language**: No need to remember CLI commands
- **Context Aware**: AI understands your project and goals
- **Comprehensive**: Contracts, parachains, pallets, and chain interaction
- **Documentation Access**: Instant answers from official guides
- **Error Recovery**: AI helps troubleshoot and debug issues
- **Complete Workflows**: From idea to deployed solution

## Ready to Build!

You're all set! Start a conversation with Claude and begin building on Polkadot.

Try:
```
I have an idea for a [your idea]. Can you help me build it?
```

Or:
```
I want to learn about Polkadot development. Where should I start?
```

Or:
```
Create a parachain with smart contract support and show me how to deploy a token on it
```

Happy building! 🚀

---

## Need More Help?

- **Configuration**: See SETUP.md
- **Examples**: See EXAMPLES.md
- **Testing**: See TEST.md
- **Publishing**: See PUBLISHING.md
- **Complete Reference**: See README.md

## Resources

- [Pop MCP Server GitHub](https://github.com/r0gue-io/pop-mcp) - This repository
- [Pop CLI](https://learn.onpop.io)
- [Pop CLI GitHub](https://github.com/r0gue-io/pop-cli)
- [ink! Documentation](https://use.ink)
- [Polkadot](https://polkadot.com)

## Support

For issues and questions:
- [Pop MCP Server Issues](https://github.com/r0gue-io/pop-mcp/issues)
- [Pop CLI Issues](https://github.com/r0gue-io/pop-cli/issues)
