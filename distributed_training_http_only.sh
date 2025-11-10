#!/bin/bash
# Pure HTTP-based Distributed GPU Training - NO SSH!
# Uses Songbird Execution Agent API on all towers

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed GPU Training - Pure HTTP (No SSH!)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
TOWER_A_IP="192.168.1.144"  # Eastgate (Hub, RTX 2070 SUPER)
TOWER_B_IP="192.168.1.134"  # Strandgate
TOWER_C_IP="192.168.1.207"  # Southgate (RTX 3090)

AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29502"

WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

# Paths on remote towers
SCRIPT_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

OUTPUT_DIR="/tmp/imagenet_distributed"

echo "🔧 Configuration:"
echo "   Master: $MASTER_ADDR:$MASTER_PORT"
echo "   World Size: $WORLD_SIZE"
echo "   Towers: A (RTX 2070 SUPER), B (Worker), C (RTX 3090 💪)"
echo "   API: HTTP Execution Agent on port $AGENT_PORT"
echo ""

# Function to launch worker via HTTP
launch_worker_http() {
    local tower_name=$1
    local tower_ip=$2
    local rank=$3
    local script_path=$4
    local agent_url="http://$tower_ip:$AGENT_PORT"
    local output_dir="$OUTPUT_DIR/rank_$rank"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 $tower_name (Rank $rank) - $tower_ip"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Health check via HTTP
    if ! curl -s -f "$agent_url/health" > /dev/null; then
        echo "❌ Execution Agent not responding"
        return 1
    fi
    echo "✅ Execution Agent responding"
    
    # Prepare command
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    # Launch via HTTP API
    echo "🚀 Launching training..."
    RESPONSE=$(curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d "{
        \"command\": \"bash\",
        \"args\": [\"-c\", \"$command\"],
        \"env\": {
          \"MASTER_ADDR\": \"$MASTER_ADDR\",
          \"MASTER_PORT\": \"$MASTER_PORT\",
          \"CUDA_VISIBLE_DEVICES\": \"0\"
        },
        \"background\": true,
        \"capture_output\": true
      }")
    
    if echo "$RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
        JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id')
        PID=$(echo "$RESPONSE" | jq -r '.pid')
        echo "✅ Worker launched (Job: $JOB_ID, PID: $PID)"
        
        # Store job ID for monitoring
        echo "$JOB_ID" > "/tmp/training_job_${tower_name}_rank_${rank}.id"
    else
        echo "❌ Failed to launch"
        echo "   Response: $RESPONSE"
        return 1
    fi
    echo ""
}

# Launch on all towers via HTTP
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Workers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

launch_worker_http "Tower_A_Eastgate" "$TOWER_A_IP" 0 "$SCRIPT_PATH_A"
sleep 2
launch_worker_http "Tower_B_Strandgate" "$TOWER_B_IP" 1 "$SCRIPT_PATH_B"
sleep 2
launch_worker_http "Tower_C_Southgate" "$TOWER_C_IP" 2 "$SCRIPT_PATH_C"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All Workers Launched via HTTP!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor via HTTP (no SSH needed!):"
echo "  ./monitor_distributed_training_http.sh"
echo ""
echo "Check status:"
echo "  curl http://$TOWER_A_IP:$AGENT_PORT/api/v1/execution/job/\$(cat /tmp/training_job_Tower_A_Eastgate_rank_0.id)"
echo "  curl http://$TOWER_B_IP:$AGENT_PORT/api/v1/execution/job/\$(cat /tmp/training_job_Tower_B_Strandgate_rank_1.id)"
echo "  curl http://$TOWER_C_IP:$AGENT_PORT/api/v1/execution/job/\$(cat /tmp/training_job_Tower_C_Southgate_rank_2.id)"
echo ""
echo "This is proper HTTP-based ecoPrimals distributed ML! 🐦🍄🔐"
echo ""

