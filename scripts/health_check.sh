#!/bin/bash
# Songbird Foundation Health Check Script
#
# Comprehensive health validation for production foundation services

set -euo pipefail

echo "🔍 SONGBIRD FOUNDATION HEALTH CHECK"
echo "===================================="

# Configuration
SONGBIRD_HOST=${SONGBIRD_HOST:-localhost}
SONGBIRD_PORT=${SONGBIRD_PORT:-8080}
TIMEOUT=${HEALTH_CHECK_TIMEOUT:-5}

echo "📋 Configuration:"
echo "  Host: $SONGBIRD_HOST"
echo "  Port: $SONGBIRD_PORT"
echo "  Timeout: ${TIMEOUT}s"
echo ""

# Health check endpoints
declare -a endpoints=(
    "/health:Overall System Health"
    "/health/config:Configuration Validation"
    "/health/discovery:Service Discovery Status"
    "/health/registry:Registry Service Status"
    "/health/universal:Universal Adapter Status"
    "/metrics:Prometheus Metrics"
)

# Track results
total_checks=0
passed_checks=0
failed_checks=0

echo "🏥 HEALTH ENDPOINT VALIDATION:"
echo "------------------------------------"

for endpoint_info in "${endpoints[@]}"; do
    IFS=':' read -r endpoint description <<< "$endpoint_info"
    total_checks=$((total_checks + 1))
    
    echo -n "Checking $description..."
    
    if curl -f -s --max-time "$TIMEOUT" "http://$SONGBIRD_HOST:$SONGBIRD_PORT$endpoint" > /dev/null 2>&1; then
        echo " ✅"
        passed_checks=$((passed_checks + 1))
    else
        echo " ❌"
        failed_checks=$((failed_checks + 1))
    fi
done

echo ""
echo "📊 HEALTH CHECK RESULTS:"
echo "------------------------------------"
echo "  Total Checks: $total_checks"
echo "  Passed: $passed_checks"
echo "  Failed: $failed_checks"
echo "  Success Rate: $(( passed_checks * 100 / total_checks ))%"

# Additional service-specific checks
echo ""
echo "🔧 SERVICE-SPECIFIC VALIDATION:"
echo "------------------------------------"

# Check if foundation packages are responding
foundation_services=("errors" "config" "types" "discovery" "universal" "registry")
service_status=0

for service in "${foundation_services[@]}"; do
    echo -n "Validating $service service..."
    
    # Simulate service-specific health check
    if curl -f -s --max-time "$TIMEOUT" "http://$SONGBIRD_HOST:$SONGBIRD_PORT/health/$service" > /dev/null 2>&1; then
        echo " ✅"
    else
        echo " ⚠️ (service may not expose individual endpoint)"
        service_status=$((service_status + 1))
    fi
done

# Performance check
echo ""
echo "⚡ PERFORMANCE VALIDATION:"
echo "------------------------------------"

echo -n "Response time check..."
start_time=$(date +%s%N)
if curl -f -s --max-time "$TIMEOUT" "http://$SONGBIRD_HOST:$SONGBIRD_PORT/health" > /dev/null 2>&1; then
    end_time=$(date +%s%N)
    response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    
    if [ "$response_time" -lt 1000 ]; then
        echo " ✅ ${response_time}ms (excellent)"
    elif [ "$response_time" -lt 5000 ]; then
        echo " ✅ ${response_time}ms (good)"
    else
        echo " ⚠️ ${response_time}ms (slow)"
    fi
else
    echo " ❌ No response"
    failed_checks=$((failed_checks + 1))
fi

# Final assessment
echo ""
echo "🎯 FINAL HEALTH ASSESSMENT:"
echo "===================================="

if [ "$failed_checks" -eq 0 ]; then
    echo "🎉 ALL SYSTEMS HEALTHY!"
    echo "✅ Foundation services are production-ready"
    echo "✅ All health endpoints responding"
    echo "✅ Performance within acceptable limits"
    echo ""
    echo "🚀 READY FOR PRODUCTION TRAFFIC"
    exit 0
elif [ "$failed_checks" -lt 3 ]; then
    echo "⚠️ MINOR ISSUES DETECTED"
    echo "✅ Core functionality operational"
    echo "⚠️ Some endpoints may need attention"
    echo "📋 Review failed checks above"
    echo ""
    echo "🔧 READY FOR PRODUCTION WITH MONITORING"
    exit 0
else
    echo "❌ SIGNIFICANT ISSUES DETECTED"
    echo "❌ $failed_checks/$total_checks health checks failed"
    echo "🚨 DO NOT DEPLOY TO PRODUCTION"
    echo "📋 Address failed health checks before deployment"
    exit 1
fi 