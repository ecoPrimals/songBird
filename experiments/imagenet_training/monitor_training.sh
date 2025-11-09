#!/bin/bash
# Real-time training monitor

LOG_FILE="../results/baseline_20epochs.log"

echo "================================================================================"
echo "  📊 LIVE TRAINING MONITOR - 20 Epoch Run"
echo "================================================================================"
echo ""
echo "Training for academic-level accuracy..."
echo "Target: 50-60% top-1 accuracy after 20 epochs"
echo ""

while true; do
    clear
    echo "================================================================================"
    echo "  📊 LIVE TRAINING MONITOR"
    echo "================================================================================"
    echo ""
    
    # Check if training is still running
    if pgrep -f "train_single.py.*baseline_20epochs" > /dev/null; then
        echo "✅ Training Status: RUNNING"
        PID=$(pgrep -f "train_single.py.*baseline_20epochs")
        echo "   PID: $PID"
        
        # Get process stats
        RUNTIME=$(ps -p $PID -o etime= | tr -d ' ')
        CPU=$(ps -p $PID -o %cpu= | tr -d ' ')
        MEM=$(ps -p $PID -o %mem= | tr -d ' ')
        
        echo "   Runtime: $RUNTIME"
        echo "   CPU: ${CPU}%"
        echo "   Memory: ${MEM}%"
    else
        echo "❌ Training Status: STOPPED"
        echo "   Training may have completed or crashed"
        echo ""
        echo "Last 20 lines of log:"
        tail -20 "$LOG_FILE"
        break
    fi
    
    echo ""
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo "  📈 PROGRESS"
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo ""
    
    # Extract latest epoch results
    tail -100 "$LOG_FILE" 2>/dev/null | grep -E "Epoch \[|Train:|Val:" | tail -10
    
    echo ""
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo "  📊 LATEST RESULTS"
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo ""
    
    # Get most recent validation results
    LAST_VAL=$(tail -100 "$LOG_FILE" 2>/dev/null | grep "Val:" | tail -1)
    if [ -n "$LAST_VAL" ]; then
        echo "$LAST_VAL"
    else
        echo "Waiting for first validation..."
    fi
    
    echo ""
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo "  🎯 TARGETS"
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo ""
    echo "Academic Benchmarks (Tiny ImageNet 200 classes):"
    echo "  Epoch 5:  10-15% top-1  ← Fast check"
    echo "  Epoch 10: 25-35% top-1  ← Good for demo"
    echo "  Epoch 15: 40-50% top-1  ← Strong results"
    echo "  Epoch 20: 50-60% top-1  ← Publication quality"
    echo ""
    echo "Published SOTA: ~62-68% top-1"
    echo ""
    echo "───────────────────────────────────────────────────────────────────────────────"
    echo ""
    echo "Refreshing every 10 seconds... (Ctrl+C to exit)"
    echo "Full log: $LOG_FILE"
    echo ""
    
    sleep 10
done

