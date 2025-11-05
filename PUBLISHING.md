# Publishing to NPM

This guide helps you publish the Pop MCP Server to NPM for distribution.

**Source Code**: [https://github.com/r0gue-io/pop-mcp](https://github.com/r0gue-io/pop-mcp)

## Prerequisites

- [x] NPM account (create at https://www.npmjs.com/signup)
- [x] Verified email address
- [ ] Decided on package name (check availability)
- [ ] Updated package.json with your details

## Pre-Publishing Checklist

### 1. Update package.json

```bash
# Edit package.json and update:
```

```json
{
  "name": "@your-scope/pop-mcp-server",  // Or "pop-mcp-server" without scope
  "version": "1.0.0",
  "description": "MCP Server for Polkadot ink! smart contract development with Pop CLI integration",
  "author": "Your Name <your.email@example.com>",
  "repository": {
    "type": "git",
    "url": "https://github.com/yourusername/pop-mcp-server"
  },
  "bugs": {
    "url": "https://github.com/yourusername/pop-mcp-server/issues"
  },
  "homepage": "https://github.com/yourusername/pop-mcp-server#readme"
}
```

### 2. Check Package Name Availability

```bash
npm search @your-scope/pop-mcp-server
# or
npm search pop-mcp-server
```

### 3. Test Build

```bash
npm run build
```

Verify:
- `build/` directory created
- `build/index.js` is executable
- No TypeScript errors

### 4. Test Installation Locally

```bash
# Pack the package
npm pack

# This creates a .tgz file
# Test install in another directory
cd /tmp
npm install /path/to/pop-mcp-server/pop-cli-mcp-server-1.0.0.tgz

# Test it works
pop-mcp-server --version 2>&1 | head -1
```

### 5. Clean Up Test Files

```bash
rm *.tgz
```

## Publishing Steps

### Option A: Publishing to NPM (Public)

#### 1. Login to NPM

```bash
npm login
```

Enter your username, password, and email.

#### 2. Publish Package

For scoped packages:
```bash
npm publish --access public
```

For unscoped packages:
```bash
npm publish
```

#### 3. Verify Publication

```bash
npm view @your-scope/pop-mcp-server
```

### Option B: GitHub Packages (Alternative)

If you prefer GitHub Packages:

#### 1. Create .npmrc

```bash
echo "@yourusername:registry=https://npm.pkg.github.com" > .npmrc
```

#### 2. Update package.json

```json
{
  "name": "@yourusername/pop-mcp-server",
  "publishConfig": {
    "registry": "https://npm.pkg.github.com"
  }
}
```

#### 3. Authenticate

```bash
npm login --registry=https://npm.pkg.github.com
```

Use your GitHub username and a personal access token (with `write:packages` scope).

#### 4. Publish

```bash
npm publish
```

## Post-Publishing

### 1. Test Global Install

```bash
npm install -g @your-scope/pop-mcp-server
# or
npm install -g pop-mcp-server

# Test
pop-mcp-server
```

### 2. Update Documentation

Update README.md with installation command:

```markdown
## Installation

```bash
npm install -g @your-scope/pop-mcp-server
```

### 3. Tag Release on GitHub

```bash
git tag v1.0.0
git push origin v1.0.0
```

### 4. Create GitHub Release

1. Go to your repository on GitHub
2. Click "Releases" → "Create a new release"
3. Choose the tag you just created
4. Add release notes
5. Publish release

### 5. Update MCP Servers List

Submit your server to the MCP servers directory:
- https://github.com/modelcontextprotocol/servers

## Version Updates

For future updates:

### 1. Update Version

```bash
# Patch: 1.0.0 → 1.0.1 (bug fixes)
npm version patch

# Minor: 1.0.0 → 1.1.0 (new features)
npm version minor

# Major: 1.0.0 → 2.0.0 (breaking changes)
npm version major
```

### 2. Push Changes

```bash
git push && git push --tags
```

### 3. Publish Update

```bash
npm publish
```

## Package.json Best Practices

Complete example:

```json
{
  "name": "@your-scope/pop-mcp-server",
  "version": "1.0.0",
  "description": "MCP Server for Polkadot ink! smart contract development with Pop CLI integration",
  "type": "module",
  "bin": {
    "pop-mcp-server": "./build/index.js"
  },
  "main": "./build/index.js",
  "types": "./build/index.d.ts",
  "files": [
    "build/",
    ".claude/docs/",
    "README.md",
    "LICENSE"
  ],
  "scripts": {
    "build": "tsc && chmod +x build/index.js",
    "watch": "tsc --watch",
    "prepare": "npm run build",
    "inspector": "node --inspect node_modules/@modelcontextprotocol/inspector/dist/index.js build/index.js"
  },
  "keywords": [
    "mcp",
    "model-context-protocol",
    "polkadot",
    "ink",
    "substrate",
    "smart-contracts",
    "web3",
    "blockchain",
    "pop-cli",
    "claude"
  ],
  "author": "Your Name <your.email@example.com>",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/yourusername/pop-mcp-server"
  },
  "bugs": {
    "url": "https://github.com/yourusername/pop-mcp-server/issues"
  },
  "homepage": "https://github.com/yourusername/pop-mcp-server#readme",
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.4"
  },
  "devDependencies": {
    "@modelcontextprotocol/inspector": "^0.1.3",
    "@types/node": "^22.10.2",
    "typescript": "^5.7.2"
  },
  "engines": {
    "node": ">=18"
  }
}
```

## Important Notes

### Files to Include

The `files` array in package.json controls what gets published:

```json
"files": [
  "build/",
  ".claude/docs/",
  "README.md",
  "LICENSE",
  "SETUP.md",
  "EXAMPLES.md",
  "QUICK_REFERENCE.md"
]
```

### Files to Exclude

`.npmignore` or use `.gitignore` patterns:

```
src/
node_modules/
*.log
*.tgz
test-quick.sh
PROJECT_SUMMARY.md
PUBLISHING.md
TEST.md
```

### License

Ensure LICENSE file is included and matches the license in package.json.

### README

Ensure README.md:
- Has clear installation instructions
- Shows configuration examples
- Lists all features
- Includes troubleshooting section
- Has links to documentation

## Troubleshooting

### "Package name already exists"

Choose a different name or use a scoped package:
- `@yourname/pop-mcp-server`
- `pop-mcp-server-enhanced`
- `polkadot-mcp-server`

### "Permission denied"

Ensure you're logged in:
```bash
npm whoami
npm login
```

### "Files not included in package"

Check the `files` array in package.json includes all necessary directories.

### "Binary not executable"

Ensure:
1. Shebang at top of index.ts: `#!/usr/bin/env node`
2. Build script includes: `chmod +x build/index.js`
3. bin field in package.json is correct

## Marketing Your Package

### 1. NPM Keywords

Use relevant keywords in package.json:
- mcp
- model-context-protocol
- polkadot
- ink
- smart-contracts
- claude

### 2. README Badges

Add badges to README.md:

```markdown
[![npm version](https://badge.fury.io/js/%40your-scope%2Fpop-mcp-server.svg)](https://badge.fury.io/js/%40your-scope%2Fpop-mcp-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Node.js Version](https://img.shields.io/node/v/@your-scope/pop-mcp-server)](https://nodejs.org)
```

### 3. Social Media

Share on:
- Twitter/X with #Polkadot #ink #MCP
- Reddit: r/Polkadot, r/programming
- Dev.to article
- Polkadot forum

### 4. Submit to Lists

- MCP servers list
- Awesome Polkadot
- Awesome ink!

## Cost

Publishing to NPM is **free** for public packages!

## Support

After publishing, monitor:
- NPM download stats
- GitHub issues
- User feedback

## Success!

Once published, users can install with:

```bash
npm install -g @your-scope/pop-mcp-server
```

And configure Claude Desktop:

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

That's it! 🎉
