/**
 * Songbird WebSocket Client - Real-Time Communication
 * 
 * Official JavaScript/Node.js client for Songbird WebSocket API.
 * Provides real-time bidirectional communication with event subscriptions.
 * 
 * Version: 0.2.1
 * Last Updated: November 11, 2025 - Phase 4
 * 
 * Features:
 * - Real-time bidirectional communication
 * - Event subscription system
 * - Automatic reconnection
 * - Ping/pong keep-alive
 * - Query federation status
 * - Service discovery
 * - Full error handling
 * 
 * Requirements:
 *     npm install ws
 * 
 * Usage:
 *     // Basic connection
 *     const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
 *     await client.connect();
 *     
 *     // Subscribe to events
 *     await client.subscribe(['service_update', 'health_update']);
 *     
 *     // Query status
 *     const status = await client.queryStatus();
 *     
 *     // Listen for events
 *     client.on('service_update', (event) => {
 *         console.log('Service updated:', event);
 *     });
 */

const WebSocket = require('ws');
const EventEmitter = require('events');

/**
 * Federation Status
 */
class FederationStatus {
    constructor(data) {
        this.totalServices = data.total_services;
        this.totalPeers = data.total_peers;
        this.uptimeSeconds = data.uptime_seconds;
    }
}

/**
 * Service Summary
 */
class ServiceSummary {
    constructor(data) {
        this.name = data.name;
        this.address = data.address;
        this.port = data.port;
        this.capabilities = data.capabilities;
    }
}

/**
 * Songbird WebSocket Client
 * 
 * Provides real-time bidirectional communication with Songbird.
 * 
 * @extends EventEmitter
 * 
 * @example
 * const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
 * await client.connect();
 * 
 * // Subscribe to events
 * await client.subscribe(['service_update', 'health_update']);
 * 
 * // Listen for events
 * client.on('service_update', (event) => {
 *     console.log('Service updated:', event.service_name);
 * });
 * 
 * // Query status
 * const status = await client.queryStatus();
 * console.log(`Services: ${status.totalServices}`);
 * 
 * await client.close();
 */
class SongbirdWebSocketClient extends EventEmitter {
    /**
     * Create a WebSocket client
     * 
     * @param {string} url - WebSocket URL (e.g., 'ws://localhost:8080/api/ws/ws')
     * @param {Object} options - Client options
     * @param {number} options.pingInterval - Seconds between ping messages (default: 30)
     * @param {number} options.pingTimeout - Timeout for pong response (default: 10)
     * @param {boolean} options.autoReconnect - Automatically reconnect (default: true)
     * @param {number} options.maxReconnectAttempts - Max reconnection attempts (default: 5)
     */
    constructor(url, options = {}) {
        super();
        
        this.url = url;
        this.pingInterval = (options.pingInterval || 30) * 1000;
        this.pingTimeout = (options.pingTimeout || 10) * 1000;
        this.autoReconnect = options.autoReconnect !== false;
        this.maxReconnectAttempts = options.maxReconnectAttempts || 5;
        
        this.ws = null;
        this.connected = false;
        this.pingTimer = null;
        this.subscriptions = [];
        this.reconnectAttempts = 0;
        this.pendingRequests = new Map();
        this.requestId = 0;
    }
    
    /**
     * Connect to Songbird WebSocket server
     * 
     * @returns {Promise<void>}
     * @throws {Error} If connection fails
     */
    connect() {
        return new Promise((resolve, reject) => {
            console.log(`Connecting to ${this.url}`);
            
            this.ws = new WebSocket(this.url);
            
            this.ws.on('open', () => {
                this.connected = true;
                this.reconnectAttempts = 0;
                console.log('Connected successfully');
                
                // Start ping loop
                this._startPingLoop();
                
                this.emit('connected');
                resolve();
            });
            
            this.ws.on('message', (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    this._handleMessage(message);
                } catch (error) {
                    console.error('Failed to parse message:', error);
                }
            });
            
            this.ws.on('close', () => {
                this.connected = false;
                this._stopPingLoop();
                console.log('Connection closed');
                
                this.emit('disconnected');
                
                if (this.autoReconnect) {
                    this._reconnect();
                }
            });
            
            this.ws.on('error', (error) => {
                console.error('WebSocket error:', error);
                this.emit('error', error);
                
                if (!this.connected) {
                    reject(error);
                }
            });
        });
    }
    
    /**
     * Close WebSocket connection
     * 
     * @returns {Promise<void>}
     */
    async close() {
        this.autoReconnect = false;
        this._stopPingLoop();
        
        if (this.ws) {
            this.ws.close();
        }
        
        console.log('Connection closed');
    }
    
    /**
     * Start ping loop
     * @private
     */
    _startPingLoop() {
        this.pingTimer = setInterval(() => {
            this.ping().catch(err => {
                console.warn('Ping failed:', err);
            });
        }, this.pingInterval);
    }
    
    /**
     * Stop ping loop
     * @private
     */
    _stopPingLoop() {
        if (this.pingTimer) {
            clearInterval(this.pingTimer);
            this.pingTimer = null;
        }
    }
    
    /**
     * Handle reconnection
     * @private
     */
    async _reconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }
        
        this.reconnectAttempts++;
        const delay = Math.min(Math.pow(2, this.reconnectAttempts) * 1000, 30000);
        
        console.log(`Reconnecting in ${delay/1000}s (attempt ${this.reconnectAttempts})`);
        
        await new Promise(resolve => setTimeout(resolve, delay));
        
        try {
            await this.connect();
            
            // Re-subscribe to events
            if (this.subscriptions.length > 0) {
                await this.subscribe(this.subscriptions);
            }
        } catch (error) {
            console.error('Reconnection failed:', error);
            this._reconnect();
        }
    }
    
    /**
     * Send message to server
     * @private
     */
    _sendMessage(message) {
        if (!this.connected || !this.ws) {
            throw new Error('Not connected');
        }
        
        this.ws.send(JSON.stringify(message));
    }
    
    /**
     * Handle incoming message
     * @private
     */
    _handleMessage(message) {
        const { type } = message;
        
        // Emit event for message type
        this.emit(type, message);
        
        // Handle responses to requests
        if (message.requestId !== undefined) {
            const pending = this.pendingRequests.get(message.requestId);
            if (pending) {
                pending.resolve(message);
                this.pendingRequests.delete(message.requestId);
            }
        }
    }
    
    /**
     * Send request and wait for response
     * @private
     */
    _sendRequest(message, timeout = 5000) {
        return new Promise((resolve, reject) => {
            const requestId = this.requestId++;
            message.requestId = requestId;
            
            const timer = setTimeout(() => {
                this.pendingRequests.delete(requestId);
                reject(new Error('Request timeout'));
            }, timeout);
            
            this.pendingRequests.set(requestId, {
                resolve: (response) => {
                    clearTimeout(timer);
                    resolve(response);
                },
                reject
            });
            
            this._sendMessage(message);
        });
    }
    
    /**
     * Subscribe to event types
     * 
     * @param {string[]} events - Event types to subscribe to
     * @returns {Promise<void>}
     * 
     * @example
     * await client.subscribe(['service_update', 'health_update']);
     */
    async subscribe(events) {
        this.subscriptions.push(...events);
        
        this._sendMessage({
            type: 'subscribe',
            events: events
        });
        
        console.log('Subscribed to:', events);
    }
    
    /**
     * Unsubscribe from event types
     * 
     * @param {string[]} events - Event types to unsubscribe from
     * @returns {Promise<void>}
     */
    async unsubscribe(events) {
        this.subscriptions = this.subscriptions.filter(e => !events.includes(e));
        
        this._sendMessage({
            type: 'unsubscribe',
            events: events
        });
        
        console.log('Unsubscribed from:', events);
    }
    
    /**
     * Send ping message
     * 
     * @param {string} [data] - Optional data to include
     * @returns {Promise<void>}
     */
    async ping(data = null) {
        const message = { type: 'ping' };
        if (data) {
            message.data = data;
        }
        this._sendMessage(message);
    }
    
    /**
     * Query federation status
     * 
     * @returns {Promise<FederationStatus>}
     * 
     * @example
     * const status = await client.queryStatus();
     * console.log(`Services: ${status.totalServices}`);
     */
    async queryStatus() {
        this._sendMessage({ type: 'query_status' });
        
        return new Promise((resolve) => {
            const handler = (message) => {
                if (message.type === 'federation_status') {
                    this.removeListener('federation_status', handler);
                    resolve(new FederationStatus(message));
                }
            };
            this.on('federation_status', handler);
        });
    }
    
    /**
     * Query services by capability
     * 
     * @param {string[]} capabilities - Required capabilities
     * @returns {Promise<ServiceSummary[]>}
     * 
     * @example
     * const services = await client.queryServices(['ml', 'training']);
     * services.forEach(s => console.log(`Service: ${s.name}`));
     */
    async queryServices(capabilities) {
        this._sendMessage({
            type: 'query_services',
            capabilities: capabilities
        });
        
        return new Promise((resolve) => {
            const handler = (message) => {
                if (message.type === 'service_list') {
                    this.removeListener('service_list', handler);
                    const services = message.services.map(s => new ServiceSummary(s));
                    resolve(services);
                }
            };
            this.on('service_list', handler);
        });
    }
    
    /**
     * Get current subscriptions
     * 
     * @returns {string[]}
     */
    getSubscriptions() {
        return [...this.subscriptions];
    }
}

// ============================================================================
// Example Usage
// ============================================================================

/**
 * Basic usage example
 */
async function exampleBasic() {
    console.log('=== Basic WebSocket Example ===\n');
    
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    
    try {
        // Connect
        await client.connect();
        console.log('✅ Connected to Songbird\n');
        
        // Query status
        const status = await client.queryStatus();
        console.log('📊 Federation Status:');
        console.log(`   Services: ${status.totalServices}`);
        console.log(`   Peers: ${status.totalPeers}`);
        console.log(`   Uptime: ${status.uptimeSeconds}s\n`);
        
        // Close
        await client.close();
        console.log('✅ Connection closed');
        
    } catch (error) {
        console.error('❌ Error:', error.message);
    }
}

/**
 * Event subscription example
 */
async function exampleSubscriptions() {
    console.log('=== Event Subscription Example ===\n');
    
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    
    try {
        await client.connect();
        console.log('✅ Connected to Songbird\n');
        
        // Subscribe to events
        await client.subscribe(['service_update', 'health_update']);
        console.log('✅ Subscribed to events\n');
        
        // Listen for events
        console.log('🔊 Listening for events (Ctrl+C to stop)...\n');
        
        client.on('service_update', (event) => {
            console.log('📨 Service Update:');
            console.log(`   Service: ${event.service_name}`);
            console.log(`   Status: ${event.status}`);
            console.log(`   Address: ${event.address}\n`);
        });
        
        client.on('health_update', (event) => {
            console.log('📨 Health Update:');
            console.log(`   Service: ${event.service_name}`);
            console.log(`   Healthy: ${event.healthy}`);
            if (event.message) {
                console.log(`   Message: ${event.message}`);
            }
            console.log();
        });
        
        // Keep running for 30 seconds
        await new Promise(resolve => setTimeout(resolve, 30000));
        
        await client.close();
        
    } catch (error) {
        console.error('❌ Error:', error.message);
    }
}

/**
 * Service discovery example
 */
async function exampleServiceDiscovery() {
    console.log('=== Service Discovery Example ===\n');
    
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    
    try {
        await client.connect();
        console.log('✅ Connected to Songbird\n');
        
        // Query services with ML capability
        console.log('🔍 Querying services with \'ml\' capability...');
        const services = await client.queryServices(['ml']);
        
        console.log(`📋 Found ${services.length} service(s):\n`);
        services.forEach(service => {
            console.log(`   • ${service.name}`);
            console.log(`     Address: ${service.address}:${service.port}`);
            console.log(`     Capabilities: ${service.capabilities.join(', ')}\n`);
        });
        
        await client.close();
        
    } catch (error) {
        console.error('❌ Error:', error.message);
    }
}

/**
 * Run examples
 */
async function main() {
    console.log('\n' + '='.repeat(80));
    console.log('Songbird WebSocket Client - JavaScript/Node.js Examples');
    console.log('='.repeat(80) + '\n');
    
    // Run examples
    await exampleBasic();
    console.log('\n' + '-'.repeat(80) + '\n');
    
    await exampleServiceDiscovery();
    console.log('\n' + '-'.repeat(80) + '\n');
    
    // Uncomment to test event subscriptions (requires active events)
    // await exampleSubscriptions();
}

// Export for use as a module
module.exports = {
    SongbirdWebSocketClient,
    FederationStatus,
    ServiceSummary
};

// Run examples if executed directly
if (require.main === module) {
    main().catch(console.error);
}

