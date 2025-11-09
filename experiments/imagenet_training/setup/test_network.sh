#!/bin/bash
# Network Performance Test for Distributed Training
# Tests bandwidth and latency between all towers

echo "========================================================================"
echo "  🌐 NETWORK PERFORMANCE TEST"
echo "  Testing connectivity for distributed ImageNet training"
echo "========================================================================"
echo ""

TOWERS=(
    "192.168.1.144:Eastgate:A"
    "192.168.1.134:Strandgate:B"
    "192.168.1.207:Southgate:C"
)

echo "📍 Tower Configuration:"
for tower in "${TOWERS[@]}"; do
    IFS=':' read -r ip name label <<< "$tower"
    echo "  Tower $label ($name): $ip"
done
echo ""

echo "========================================================================"
echo "  🔍 CONNECTIVITY TEST"
echo "========================================================================"
echo ""

for tower in "${TOWERS[@]}"; do
    IFS=':' read -r ip name label <<< "$tower"
    echo "Testing Tower $label ($name) at $ip:"
    
    if ping -c 3 -W 2 "$ip" > /dev/null 2>&1; then
        latency=$(ping -c 5 "$ip" 2>/dev/null | tail -1 | awk '{print $4}' | cut -d '/' -f 2)
        echo "  ✅ ONLINE - Latency: ${latency}ms"
    else
        echo "  ❌ OFFLINE or UNREACHABLE"
    fi
    echo ""
done

echo "========================================================================"
echo "  ⚡ BANDWIDTH TEST (requires iperf3)"
echo "========================================================================"
echo ""

if ! command -v iperf3 &> /dev/null; then
    echo "⚠️  iperf3 not installed. Install with:"
    echo "   sudo apt install iperf3"
    echo ""
    echo "To test bandwidth manually:"
    echo "  1. On remote tower: iperf3 -s"
    echo "  2. On this tower: iperf3 -c <remote_ip> -t 10"
    echo ""
else
    echo "ℹ️  Bandwidth test requires iperf3 server running on remote towers."
    echo ""
    echo "To run full bandwidth test:"
    echo "  1. On Tower B: iperf3 -s -p 5201"
    echo "  2. On Tower C: iperf3 -s -p 5201"
    echo "  3. Run: ./test_network.sh --bandwidth"
    echo ""
    
    if [[ "$1" == "--bandwidth" ]]; then
        echo "Running bandwidth tests..."
        echo ""
        
        for tower in "${TOWERS[@]}"; do
            IFS=':' read -r ip name label <<< "$tower"
            
            # Skip testing to ourselves
            if [[ "$ip" == "192.168.1.144" ]]; then
                continue
            fi
            
            echo "Testing to Tower $label ($name):"
            if timeout 15 iperf3 -c "$ip" -t 10 -f M 2>/dev/null | grep "sender\|receiver"; then
                echo "  ✅ Bandwidth test complete"
            else
                echo "  ⚠️  Server not running or unreachable"
            fi
            echo ""
        done
    fi
fi

echo "========================================================================"
echo "  📊 REQUIREMENTS FOR DISTRIBUTED TRAINING"
echo "========================================================================"
echo ""
echo "Minimum Requirements:"
echo "  • Latency: <10ms (lower is better)"
echo "  • Bandwidth: >500 Mbps (1+ Gbps ideal)"
echo "  • Packet Loss: <0.1%"
echo ""
echo "Expected gradient sync time with 100MB gradients:"
echo "  • At 100 Mbps: ~8 seconds (TOO SLOW)"
echo "  • At 500 Mbps: ~1.6 seconds (ACCEPTABLE)"
echo "  • At 1 Gbps: ~0.8 seconds (GOOD)"
echo "  • At 10 Gbps: ~0.08 seconds (EXCELLENT)"
echo ""
echo "========================================================================"
echo "  ✅ NEXT STEPS"
echo "========================================================================"
echo ""
echo "If connectivity looks good:"
echo "  1. Install iperf3 on all towers"
echo "  2. Run bandwidth tests"
echo "  3. Proceed with data download and sharding"
echo ""
echo "If issues detected:"
echo "  • Check firewall rules"
echo "  • Verify network switch configuration"
echo "  • Test with direct ethernet connection"
echo ""

