# Testing the Pop MCP Server

This document describes how to test the MCP server to ensure it's working correctly.

## Prerequisites

- Node.js 18+ installed
- Pop CLI installed (or you can test the installation check feature)
- MCP server built (`npm run build`)

## Manual Testing with MCP Inspector

The MCP Inspector provides a web UI for testing your server without needing to configure it in Claude Desktop.

### Start the Inspector

```bash
npm run inspector
```

This will:
1. Start the MCP server
2. Launch a proxy server
3. Open a web browser with the inspector UI

### Test Each Tool

#### 1. Test check_pop_installation

**Input:** (no parameters needed)

**Expected Output:**
```
Pop CLI is installed:
pop-cli 0.11.0

Run 'pop --help' for more information.
```

#### 2. Test list_templates

**Input:** (no parameters needed)

**Expected Output:**
List of 7 templates with descriptions

#### 3. Test install_pop_instructions

**Input:**
```json
{
  "platform": "macos"
}
```

**Expected Output:**
Installation instructions for macOS

#### 4. Test pop_help

**Input:**
```json
{
  "command": "new contract"
}
```

**Expected Output:**
Help text for the `pop new contract` command

### Test Resources

In the inspector, navigate to the Resources tab and verify all 5 documentation resources are available:

1. `ink://docs/llm-guide`
2. `ink://docs/technical-guide`
3. `pop://docs/cli-guide`
4. `xcm://docs/comprehensive-guide`
5. `xcm://docs/ink-examples`

Click on each to verify the content loads.

## Testing in Claude Desktop

### Step 1: Configure Claude Desktop

Follow the instructions in SETUP.md to configure Claude Desktop.

### Step 2: Restart Claude Desktop

Completely quit and restart Claude Desktop after adding the configuration.

### Step 3: Verify Connection

Open a new conversation and ask:

```
What MCP servers do you have access to?
```

Claude should mention the pop-cli server.

### Step 4: Test Basic Functionality

Try these prompts in order:

#### Test 1: Installation Check
```
Is Pop CLI installed?
```

**Expected:** Claude should report that Pop CLI 0.11.0 (or later) is installed.

#### Test 2: List Templates
```
What contract templates are available?
```

**Expected:** Claude should list all 7 templates with descriptions.

#### Test 3: Documentation Access
```
How do I define storage in an ink! smart contract?
```

**Expected:** Claude should read from the ink! documentation and provide a detailed answer about storage macros, storage layout, etc.

#### Test 4: XCM Documentation
```
Explain how to send XCM messages from an ink! contract
```

**Expected:** Claude should reference the XCM documentation and provide detailed information.

#### Test 5: Pop CLI Help
```
Show me the help for the pop build command
```

**Expected:** Claude should execute `pop build --help` and show the output.

### Step 5: Test Contract Workflow (Optional)

If you want to test the full workflow:

#### Create a Test Contract
```
Create a new standard contract called test-contract in /tmp
```

**Expected:** Contract created successfully

#### Build the Contract
```
Build the contract at /tmp/test-contract
```

**Expected:** Build succeeds, shows output location

#### Test the Contract
```
Run tests for /tmp/test-contract
```

**Expected:** Tests pass

#### Get Contract Info
```
Show me information about the built contract
```

**Expected:** Displays metadata and file locations

## Automated Testing

### Unit Test (Future Enhancement)

You could create a test file `src/index.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { executeCommand } from './index.js';

describe('Pop MCP Server', () => {
  it('should check pop installation', () => {
    const result = executeCommand('pop --version');
    expect(result.success).toBe(true);
    expect(result.stdout).toContain('pop-cli');
  });

  it('should list templates', () => {
    const result = executeCommand('pop new contract --help');
    expect(result.success).toBe(true);
    expect(result.stdout).toContain('template');
  });
});
```

To use this, you'd need to:
1. Add vitest to devDependencies
2. Export the helper functions
3. Add test script to package.json

## Integration Testing

### Test Script

Create a simple test script `test.sh`:

```bash
#!/bin/bash

echo "Testing Pop MCP Server..."

# Test 1: Check if build exists
if [ ! -f "build/index.js" ]; then
    echo "❌ Build not found. Run 'npm run build' first."
    exit 1
fi
echo "✅ Build exists"

# Test 2: Check if server starts
timeout 2 node build/index.js > /dev/null 2>&1
if [ $? -eq 124 ]; then
    echo "✅ Server starts (timed out as expected)"
else
    echo "⚠️  Server exited unexpectedly"
fi

# Test 3: Check Pop CLI
if command -v pop &> /dev/null; then
    POP_VERSION=$(pop --version)
    echo "✅ Pop CLI installed: $POP_VERSION"
else
    echo "⚠️  Pop CLI not installed (some features will not work)"
fi

# Test 4: Check documentation files
DOCS_DIR=".claude/docs"
DOC_FILES=(
    "ink-llms.txt"
    "ink-technical-guide.txt"
    "pop-cli-comprehensive-guide.txt"
    "xcm-comprehensive-guide.txt"
    "xcm-ink-examples-guide.txt"
)

for file in "${DOC_FILES[@]}"; do
    if [ -f "$DOCS_DIR/$file" ]; then
        echo "✅ Documentation found: $file"
    else
        echo "❌ Missing documentation: $file"
        exit 1
    fi
done

echo ""
echo "All tests passed! 🎉"
echo ""
echo "Next steps:"
echo "1. Configure Claude Desktop (see SETUP.md)"
echo "2. Test in Claude Desktop (see TEST.md)"
```

Make it executable and run:

```bash
chmod +x test.sh
./test.sh
```

## Troubleshooting Tests

### Server Won't Start

**Issue:** `node build/index.js` fails

**Solutions:**
1. Rebuild: `npm run build`
2. Check Node version: `node --version` (should be 18+)
3. Check for TypeScript errors in src/index.ts

### Pop CLI Not Found

**Issue:** Commands fail with "pop: command not found"

**Solutions:**
1. Install Pop CLI: `brew install r0gue-io/pop-cli/pop`
2. Check PATH: `echo $PATH | grep homebrew`
3. Restart terminal after installation

### Documentation Not Loading

**Issue:** Resources return errors

**Solutions:**
1. Verify files exist: `ls -la .claude/docs/`
2. Check file permissions: `chmod 644 .claude/docs/*.txt`
3. Verify paths in src/index.ts match actual file locations

### Claude Desktop Not Seeing Server

**Issue:** Claude doesn't show MCP tools

**Solutions:**
1. Verify config file location (see SETUP.md)
2. Check JSON syntax is valid
3. Use absolute path to build/index.js
4. Restart Claude Desktop completely
5. Check Claude Desktop logs (Help > Debug Info)

## Success Criteria

Your MCP server is working correctly if:

- ✅ Server starts without errors
- ✅ All 5 documentation resources are accessible
- ✅ All 12 tools are available
- ✅ Pop CLI commands execute successfully
- ✅ Claude Desktop shows the server in available tools
- ✅ You can create, build, test, and deploy a contract through Claude

## Performance Testing

Test with a real contract workflow:

```bash
# Time how long it takes to create and build a contract
time (
    pop new contract perf-test --template standard &&
    cd perf-test &&
    pop build
)
```

Typical times:
- Contract creation: 1-2 seconds
- Build (first time): 30-60 seconds
- Build (incremental): 5-10 seconds

## Reporting Issues

If you encounter issues:

1. Check the console output for errors
2. Verify all prerequisites are met
3. Try the test commands in isolation
4. Check Claude Desktop logs
5. Open an issue with:
   - Error messages
   - Your environment (OS, Node version, Pop CLI version)
   - Steps to reproduce

## Next Steps

Once testing is complete:

1. Review EXAMPLES.md for usage patterns
2. Start using the server in Claude Desktop
3. Provide feedback on what works well
4. Suggest additional tools or improvements

Happy testing! 🧪
