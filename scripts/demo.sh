#!/bin/bash
# Demo script for screen recording

set -e
clear

echo "🧠 CORTEX DEMO"
echo "=============="
echo ""
echo "1. Ingesting a real project..."
cortex ingest --path .
echo ""

echo "2. Asking about the codebase..."
cortex ask "what does the temporal module do"
echo ""

echo "3. Semantic search..."
cortex search "working memory stack"
echo ""

echo "4. Checking cognitive debt..."
cortex debt
echo ""

echo "5. Recalling yesterday..."
cortex recall yesterday
echo ""

echo "6. Current focus..."
cortex focus
echo ""

echo "Done! 🎉"
