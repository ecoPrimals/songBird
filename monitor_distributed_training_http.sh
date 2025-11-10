#!/bin/bash
# Monitor Distributed Training via HTTP (No SSH!)

TOWER_A_IP="192.168.1.144"
TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
AGENT_PORT="9020"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Distributed Training Status - HTTP API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Function to check job status via HTTP
check_job() {
    local tower_name=$1
    local tower_ip=$2
    local job_file=$3
    
    if [ ! -f "$job_file" ]; then
        echo "❌ $tower_name: No job ID found"
        return
    fi
    
    JOB_ID=$(cat "$job_file")
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🖥️  $tower_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    RESPONSE=$(curl -s "http://$tower_ip:$AGENT_PORT/api/v1/execution/job/$JOB_ID")
    
    if echo "$RESPONSE" | jq -e '.status' > /dev/null 2>&1; then
        STATUS=$(echo "$RESPONSE" | jq -r '.status')
        PID=$(echo "$RESPONSE" | jq -r '.pid')
        DURATION=$(echo "$RESPONSE" | jq -r '.duration_ms')
        
        echo "  Job ID: $JOB_ID"
        echo "  Status: $STATUS"
        echo "  PID: $PID"
        
        if [ "$DURATION" != "null" ]; then
            DURATION_SEC=$((DURATION / 1000))
            echo "  Duration: ${DURATION_SEC}s"
        fi
        
        # Check if we can get GPU status via HTTP command execution
        GPU_CMD="nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits"
        GPU_RESPONSE=$(curl -s -X POST "http://$tower_ip:$AGENT_PORT/api/v1/execution/command" \
          -H "Content-Type: application/json" \
          -d "{
            \"command\": \"bash\",
            \"args\": [\"-c\", \"$GPU_CMD\"],
            \"env\": {},
            \"background\": false,
            \"capture_output\": true
          }")
        
        if echo "$GPU_RESPONSE" | jq -e '.stdout' > /dev/null 2>&1; then
            GPU_INFO=$(echo "$GPU_RESPONSE" | jq -r '.stdout' | tr -d '\n')
            if [ -n "$GPU_INFO" ] && [ "$GPU_INFO" != "null" ]; then
                IFS=',' read -r GPU_UTIL GPU_MEM_USED GPU_MEM_TOTAL <<< "$GPU_INFO"
                echo "  GPU: ${GPU_UTIL}% utilization, ${GPU_MEM_USED}MB / ${GPU_MEM_TOTAL}MB"
            fi
        fi
    else
        echo "  ❌ Could not fetch status"
    fi
    echo ""
}

# Check all towers
check_job "Tower A (Eastgate - RTX 2070 SUPER)" "$TOWER_A_IP" "/tmp/training_job_Tower_A_Eastgate_rank_0.id"
check_job "Tower B (Strandgate)" "$TOWER_B_IP" "/tmp/training_job_Tower_B_Strandgate_rank_1.id"
check_job "Tower C (Southgate - RTX 3090 💪)" "$TOWER_C_IP" "/tmp/training_job_Tower_C_Southgate_rank_2.id"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Status check complete (via HTTP API)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Refresh status: ./monitor_distributed_training_http.sh"
echo ""
echo "This is proper HTTP-based ecoPrimals monitoring! 🐦🍄🔐"
echo ""

