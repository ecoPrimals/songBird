import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTimeTrend = new Trend('response_time');
const requestCounter = new Counter('requests_total');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 10 }, // Ramp up to 10 users
    { duration: '5m', target: 10 }, // Stay at 10 users
    { duration: '2m', target: 20 }, // Ramp up to 20 users
    { duration: '5m', target: 20 }, // Stay at 20 users
    { duration: '2m', target: 50 }, // Ramp up to 50 users
    { duration: '5m', target: 50 }, // Stay at 50 users
    { duration: '2m', target: 0 },  // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests should be below 500ms
    http_req_failed: ['rate<0.1'],    // Error rate should be below 10%
    errors: ['rate<0.1'],             // Custom error rate should be below 10%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Test health endpoint
  let healthResponse = http.get(`${BASE_URL}/health`);
  let healthCheck = check(healthResponse, {
    'health endpoint status is 200': (r) => r.status === 200,
    'health response time < 100ms': (r) => r.timings.duration < 100,
  });
  
  if (!healthCheck) {
    errorRate.add(1);
  }
  requestCounter.add(1);
  responseTimeTrend.add(healthResponse.timings.duration);

  // Test metrics endpoint
  let metricsResponse = http.get(`${BASE_URL}/metrics`);
  let metricsCheck = check(metricsResponse, {
    'metrics endpoint status is 200': (r) => r.status === 200,
    'metrics response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  if (!metricsCheck) {
    errorRate.add(1);
  }
  requestCounter.add(1);
  responseTimeTrend.add(metricsResponse.timings.duration);

  // Test discovery endpoint
  let discoveryResponse = http.get(`${BASE_URL}/discovery/services`);
  let discoveryCheck = check(discoveryResponse, {
    'discovery endpoint status is 200': (r) => r.status === 200,
    'discovery response time < 300ms': (r) => r.timings.duration < 300,
    'discovery response has content': (r) => r.body.length > 0,
  });
  
  if (!discoveryCheck) {
    errorRate.add(1);
  }
  requestCounter.add(1);
  responseTimeTrend.add(discoveryResponse.timings.duration);

  // Test configuration endpoint
  let configResponse = http.get(`${BASE_URL}/config`);
  let configCheck = check(configResponse, {
    'config endpoint status is 200': (r) => r.status === 200,
    'config response time < 150ms': (r) => r.timings.duration < 150,
  });
  
  if (!configCheck) {
    errorRate.add(1);
  }
  requestCounter.add(1);
  responseTimeTrend.add(configResponse.timings.duration);

  // Simulate realistic user behavior
  sleep(1 + Math.random() * 2); // Sleep between 1-3 seconds
}

export function handleSummary(data) {
  return {
    'load-test-results.json': JSON.stringify(data, null, 2),
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  };
}

function textSummary(data, options) {
  const indent = options.indent || '';
  const enableColors = options.enableColors || false;
  
  let summary = `
${indent}📊 Songbird Load Test Results
${indent}============================
${indent}
${indent}🎯 Test Summary:
${indent}  - Total Requests: ${data.metrics.requests_total.values.count}
${indent}  - Failed Requests: ${data.metrics.http_req_failed.values.rate * 100}%
${indent}  - Average Response Time: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms
${indent}  - 95th Percentile: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms
${indent}  - Max Response Time: ${data.metrics.http_req_duration.values.max.toFixed(2)}ms
${indent}
${indent}🚀 Performance Metrics:
${indent}  - Requests/sec: ${data.metrics.http_reqs.values.rate.toFixed(2)}
${indent}  - Data Received: ${(data.metrics.data_received.values.count / 1024 / 1024).toFixed(2)} MB
${indent}  - Data Sent: ${(data.metrics.data_sent.values.count / 1024).toFixed(2)} KB
${indent}
${indent}⚡ Thresholds:
${indent}  - Response Time P95 < 500ms: ${data.metrics.http_req_duration.values['p(95)'] < 500 ? '✅ PASS' : '❌ FAIL'}
${indent}  - Error Rate < 10%: ${data.metrics.http_req_failed.values.rate < 0.1 ? '✅ PASS' : '❌ FAIL'}
${indent}
`;
  
  return summary;
} 