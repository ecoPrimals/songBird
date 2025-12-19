#!/bin/bash
# Watch for Westgate Federation Connection
# Monitors eastgate logs for westgate discovery and federation join

echo "🔍 Watching for Westgate Connection..."
echo "========================================"
echo ""
echo "Monitoring eastgate logs for:"
echo "  - Westgate discovery"
echo "  - Trust establishment"
echo "  - Federation join"
echo ""
echo "Press Ctrl+C to stop"
echo ""
echo "Watching logs..."
echo ""

# Find the most recent log file
LOG_FILE=$(ls -t logs/eastgate-*.log 2>/dev/null | head -1)

if [ -z "$LOG_FILE" ]; then
    echo "❌ No eastgate log file found!"
    echo "Is eastgate running?"
    exit 1
fi

echo "📄 Monitoring: $LOG_FILE"
echo ""

# Watch for specific patterns
tail -f "$LOG_FILE" | grep --line-buffered -E "westgate|Discovered peer|192\.168\.1\.123|Federation.*joined|Trust.*established|Active nodes.*2" | while read -r line; do
    timestamp=$(date '+%H:%M:%S')
    
    # Colorize different event types
    if echo "$line" | grep -q "Discovered peer.*westgate"; then
        echo "🎯 [$timestamp] DISCOVERY: $line"
    elif echo "$line" | grep -q "192\.168\.1\.123"; then
        echo "📡 [$timestamp] WESTGATE: $line"
    elif echo "$line" | grep -q "Trust.*established"; then
        echo "🤝 [$timestamp] TRUST: $line"
    elif echo "$line" | grep -q "Federation.*joined"; then
        echo "🎊 [$timestamp] FEDERATION: $line"
        echo ""
        echo "✅ Westgate has joined the federation!"
        echo ""
        echo "Verify with: ./check-tower.sh"
        echo ""
    elif echo "$line" | grep -q "Active nodes.*2"; then
        echo "🌐 [$timestamp] SUCCESS: $line"
        echo ""
        echo "🎉 Federation established with 2 nodes!"
        echo ""
        ./check-tower.sh
        echo ""
        exit 0
    else
        echo "📝 [$timestamp] $line"
    fi
done

