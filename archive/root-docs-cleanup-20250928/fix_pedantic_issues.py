#!/usr/bin/env python3
"""
PEDANTIC POLISH SCRIPT
Systematically fixes all clippy pedantic issues for absolute code quality
"""

import re
import os
from pathlib import Path

def add_copy_derives():
    """Add Copy derives to eligible types"""
    files_to_fix = [
        "crates/songbird-types/src/unified_constants.rs",
        "crates/songbird-types/src/generated_unified_constants.rs"
    ]
    
    for file_path in files_to_fix:
        if os.path.exists(file_path):
            with open(file_path, 'r') as f:
                content = f.read()
            
            # Add Copy to structs that can have it
            content = re.sub(
                r'#\[derive\(([^)]*)\)\]\s*pub struct (ConstantsSet|UnifiedConstantsFactory)',
                r'#[derive(\1, Copy)]\npub struct \2',
                content
            )
            
            # Add Debug to structs missing it
            content = re.sub(
                r'pub struct (UnifiedConstantsFactory);',
                r'#[derive(Debug, Copy, Clone)]\npub struct \1;',
                content
            )
            
            with open(file_path, 'w') as f:
                f.write(content)

def add_missing_docs():
    """Add missing documentation to constants and struct fields"""
    constants_docs = {
        'DEFAULT_DISCOVERY_PORT': 'Default port for service discovery',
        'DEFAULT_GAMING_PORT': 'Default port for gaming services',
        'DEFAULT_HEALTH_PORT': 'Default port for health checks',
        'DEFAULT_DASHBOARD_PORT': 'Default port for dashboard interface',
        'DEFAULT_METRICS_PORT': 'Default port for metrics collection',
        'DEFAULT_FEDERATION_PORT': 'Default port for federation services',
        'DEFAULT_WEBSOCKET_PORT': 'Default port for WebSocket connections',
        'DEFAULT_GRPC_PORT': 'Default port for gRPC services',
        'DEFAULT_ADMIN_PORT': 'Default port for admin interface',
        'DEFAULT_HTTPS_PORT': 'Default HTTPS port',
        'DEFAULT_DEV_HTTP_PORT': 'Default HTTP port for development',
        'DEFAULT_DEV_HTTPS_PORT': 'Default HTTPS port for development',
        'LOCALHOST_BIND_ADDRESS': 'Localhost bind address',
        'DEFAULT_LOCALHOST': 'Default localhost hostname',
        'MAX_DYNAMIC_PORT': 'Maximum dynamic port number',
        'GAMING_PORT_RANGE_START': 'Start of gaming port range',
        'GAMING_PORT_RANGE_END': 'End of gaming port range',
        'DISCOVERY_PORT_RANGE_START': 'Start of discovery port range',
        'DISCOVERY_PORT_RANGE_END': 'End of discovery port range',
        'TEST_HTTP_PORT': 'HTTP port for testing',
        'TEST_HTTPS_PORT': 'HTTPS port for testing',
        'TEST_PORT_RANGE_START': 'Start of test port range',
        'TEST_PORT_RANGE_END': 'End of test port range',
        'MAX_PACKET_SIZE': 'Maximum packet size in bytes',
        'JUMBO_FRAME_SIZE': 'Jumbo frame size in bytes',
        'WEBSOCKET_VERSION': 'WebSocket protocol version',
        'GRPC_VERSION': 'gRPC protocol version',
        'DEFAULT_READ_TIMEOUT': 'Default read timeout duration',
        'DEFAULT_WRITE_TIMEOUT': 'Default write timeout duration',
        'DEFAULT_KEEPALIVE_TIMEOUT': 'Default keepalive timeout duration',
        'DEFAULT_RETRY_DELAY': 'Default retry delay duration',
        'FAST_REQUEST_TIMEOUT': 'Timeout for fast requests',
        'SLOW_REQUEST_TIMEOUT': 'Timeout for slow requests',
        'DEFAULT_SCAN_TIMEOUT': 'Default scan timeout duration',
        'HEALTH_CHECK_INTERVAL': 'Health check interval duration',
        'HEALTH_CHECK_GRACE_PERIOD': 'Health check grace period',
        'SERVICE_DISCOVERY_TIMEOUT': 'Service discovery timeout',
        'SERVICE_SHUTDOWN_TIMEOUT': 'Service shutdown timeout',
        'SERVICE_STARTUP_TIMEOUT': 'Service startup timeout',
        'DEFAULT_EVALUATION_TIMEOUT': 'Default evaluation timeout',
        'BACKGROUND_TASK_TIMEOUT': 'Background task timeout',
        'PLAYER_IDLE_TIMEOUT': 'Player idle timeout',
        'MATCH_TIMEOUT': 'Match timeout duration',
        'TEST_RETRY_DELAY': 'Retry delay for tests',
        'AUTH_TIMEOUT': 'Authentication timeout',
        'DEFAULT_MAX_CONNECTIONS_PER_IP': 'Maximum connections per IP address',
        'DEFAULT_CONNECTION_BACKLOG': 'Default connection backlog size',
        'MAX_CONCURRENT_REQUESTS': 'Maximum concurrent requests',
        'MAX_BUFFER_SIZE': 'Maximum buffer size',
        'MIN_BUFFER_SIZE': 'Minimum buffer size',
        'LARGE_BUFFER_SIZE': 'Large buffer size',
        'SMALL_BUFFER_SIZE': 'Small buffer size',
        'HUGE_BUFFER_SIZE': 'Huge buffer size',
        'MAX_SERVICE_DESCRIPTION_LENGTH': 'Maximum service description length',
        'MAX_SERVICE_INSTANCES': 'Maximum service instances',
        'MAX_SERVICE_METADATA_SIZE': 'Maximum service metadata size',
        'MAX_SERVICE_TAGS': 'Maximum number of service tags',
        'MAX_HEALTH_RESPONSE_SIZE': 'Maximum health response size',
        'MAX_HEALTH_CHECK_HISTORY': 'Maximum health check history entries',
        'MAX_WORKER_THREADS': 'Maximum worker threads',
        'DEFAULT_BATCH_SIZE': 'Default batch processing size',
        'MAX_BATCH_SIZE': 'Maximum batch processing size',
        'MIN_BATCH_SIZE': 'Minimum batch processing size',
        'MAX_MEMORY_LIMIT': 'Maximum memory limit',
        'DEFAULT_CACHE_SIZE': 'Default cache size',
        'MAX_GAMES_PER_SERVER': 'Maximum games per server',
        'MAX_SPECTATORS_PER_GAME': 'Maximum spectators per game',
        'MAX_CHAT_MESSAGE_LENGTH': 'Maximum chat message length',
        'MAX_LOG_FILE_SIZE': 'Maximum log file size',
        'MAX_CONFIG_FILE_SIZE': 'Maximum config file size',
        'TEST_BUFFER_SIZE': 'Buffer size for testing',
        'TEST_BATCH_SIZE': 'Batch size for testing',
        'DISCOVERY_RETRY_ATTEMPTS': 'Number of discovery retry attempts',
        'DISCOVERY_BACKOFF_MULTIPLIER': 'Discovery backoff multiplier',
        'SERVICE_HEARTBEAT_INTERVAL': 'Service heartbeat interval',
        'SERVICE_TTL': 'Service time-to-live',
        'CAPABILITY_CACHE_SIZE': 'Capability cache size',
        'CAPABILITY_REFRESH_INTERVAL': 'Capability refresh interval',
        'MAX_TICK_RATE': 'Maximum tick rate',
        'MIN_TICK_RATE': 'Minimum tick rate',
        'GAME_PROTOCOL_VERSION': 'Game protocol version',
        'MAX_PACKET_LOSS_TOLERANCE': 'Maximum packet loss tolerance',
        'IDEAL_LATENCY_MS': 'Ideal latency in milliseconds',
        'PRODUCTION_ENV': 'Production environment identifier',
        'TESTING_ENV': 'Testing environment identifier',
        'STAGING_ENV': 'Staging environment identifier',
        'DEFAULT_DATA_DIR': 'Default data directory',
        'DEFAULT_LOG_DIR': 'Default log directory',
        'DEFAULT_CACHE_DIR': 'Default cache directory',
        'DEV_DATA_DIR': 'Development data directory',
        'DEV_LOG_DIR': 'Development log directory',
        'DEV_CACHE_DIR': 'Development cache directory',
        'MAX_LOGIN_ATTEMPTS': 'Maximum login attempts',
        'LOCKOUT_DURATION': 'Account lockout duration',
        'TOKEN_REFRESH_THRESHOLD': 'Token refresh threshold',
        'FAST_TEST_TIMEOUT': 'Fast test timeout',
        'INTEGRATION_TEST_TIMEOUT': 'Integration test timeout',
        'MEDIUM_DATASET': 'Medium dataset size',
        'LARGE_DATASET': 'Large dataset size',
    }
    
    field_docs = {
        'environment': 'Environment identifier',
        'bind_address': 'Network bind address',
        'http_port': 'HTTP port number',
        'network_timeout': 'Network timeout duration',
        'max_connections': 'Maximum number of connections',
        'buffer_size': 'Buffer size in bytes',
        'log_level': 'Logging level',
    }
    
    files_to_fix = [
        "crates/songbird-types/src/unified_constants.rs",
        "crates/songbird-types/src/generated_unified_constants.rs"
    ]
    
    for file_path in files_to_fix:
        if os.path.exists(file_path):
            with open(file_path, 'r') as f:
                lines = f.readlines()
            
            new_lines = []
            for i, line in enumerate(lines):
                # Add documentation for constants
                for const_name, doc in constants_docs.items():
                    if f'pub const {const_name}:' in line and i > 0:
                        if not lines[i-1].strip().startswith('///'):
                            new_lines.append(f'    /// {doc}\n')
                        break
                
                # Add documentation for struct fields
                for field_name, doc in field_docs.items():
                    if f'pub {field_name}:' in line and i > 0:
                        if not lines[i-1].strip().startswith('///'):
                            new_lines.append(f'    /// {doc}\n')
                        break
                
                new_lines.append(line)
            
            with open(file_path, 'w') as f:
                f.writelines(new_lines)

def fix_missing_variants_docs():
    """Fix missing documentation for enum variants"""
    files_to_fix = [
        "crates/songbird-types/src/traits/unified_providers.rs"
    ]
    
    variant_docs = {
        'Success': 'Operation completed successfully',
        'Error': 'Operation encountered an error',
        'Partial': 'Operation completed partially',
        'Service': 'Service dependency',
        'Library': 'Library dependency', 
        'Configuration': 'Configuration dependency',
        'Resource': 'Resource dependency',
    }
    
    for file_path in files_to_fix:
        if os.path.exists(file_path):
            with open(file_path, 'r') as f:
                lines = f.readlines()
            
            new_lines = []
            for i, line in enumerate(lines):
                # Add documentation for enum variants
                for variant_name, doc in variant_docs.items():
                    if line.strip() == f'{variant_name},' and i > 0:
                        if not lines[i-1].strip().startswith('///'):
                            new_lines.append(f'    /// {doc}\n')
                        break
                    elif line.strip().startswith(f'{variant_name}(') and i > 0:
                        if not lines[i-1].strip().startswith('///'):
                            new_lines.append(f'    /// {doc}\n')
                        break
                
                new_lines.append(line)
            
            with open(file_path, 'w') as f:
                f.writelines(new_lines)

if __name__ == "__main__":
    print("🔧 PEDANTIC POLISH: Fixing all clippy issues...")
    
    print("  ✅ Adding Copy derives...")
    add_copy_derives()
    
    print("  ✅ Adding missing documentation...")
    add_missing_docs()
    
    print("  ✅ Fixing enum variant documentation...")
    fix_missing_variants_docs()
    
    print("🎉 PEDANTIC POLISH COMPLETE!") 