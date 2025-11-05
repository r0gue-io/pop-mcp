#!/bin/bash
echo "🧪 Quick Test Suite for Pop MCP Server"
echo "======================================"
echo ""

# Test 1: Build exists
if [ -f "build/index.js" ]; then
    echo "✅ Build output exists"
else
    echo "❌ Build not found"
    exit 1
fi

# Test 2: Documentation files
MISSING=0
for file in ink-llms.txt ink-technical-guide.txt pop-cli-comprehensive-guide.txt xcm-comprehensive-guide.txt xcm-ink-examples-guide.txt; do
    if [ -f ".claude/docs/$file" ]; then
        echo "✅ Doc: $file"
    else
        echo "❌ Missing: $file"
        MISSING=1
    fi
done

if [ $MISSING -eq 1 ]; then
    exit 1
fi

# Test 3: Pop CLI
if command -v pop &> /dev/null; then
    echo "✅ Pop CLI: $(pop --version)"
else
    echo "⚠️  Pop CLI not installed (optional)"
fi

# Test 4: Node version
NODE_VERSION=$(node --version)
echo "✅ Node: $NODE_VERSION"

echo ""
echo "🎉 All critical tests passed!"
echo ""
echo "Next steps:"
echo "  1. Configure Claude Desktop (see SETUP.md)"
echo "  2. npm run inspector (optional - test server)"
echo "  3. Start using in Claude!"
