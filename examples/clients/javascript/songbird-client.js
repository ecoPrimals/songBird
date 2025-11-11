/**
 * Songbird JSON-RPC JavaScript/Node.js Client
 * Universal Gateway Client for Songbird Orchestrator
 * 
 * Usage (Node.js):
 *   const { SongbirdClient } = require('./songbird-client');
 *   const client = new SongbirdClient('http://localhost:8080');
 *   const health = await client.health();
 *   console.log(health);
 * 
 * Usage (Browser):
 *   <script src="songbird-client.js"></script>
 *   <script>
 *     const client = new SongbirdClient('http://localhost:8080');
 *     client.health().then(console.log);
 *   </script>
 * 
 * Version: 0.2.1
 * Last Updated: November 11, 2025
 */

/**
 * Custom error class for Songbird client errors
 */
class SongbirdError extends Error {
    constructor(code, message, data = null) {
        super(`Error ${code}: ${message}`);
        this.name = 'SongbirdError';
        this.code = code;
        this.data = data;
    }
}

/**
 * Songbird JSON-RPC Client
 * 
 * Simple, universal client for Songbird's JSON-RPC API
 * Works in both Node.js and modern browsers
 * 
 * @example
 * const client = new SongbirdClient('http://localhost:8080');
 * const health = await client.health();
 * console.log(health.status); // 'healthy'
 */
class SongbirdClient {
    /**
     * Create a Songbird client
     * 
     * @param {string} baseUrl - Songbird orchestrator URL (default: http://localhost:8080)
     * @param {number} timeout - Request timeout in milliseconds (default: 30000)
     */
    constructor(baseUrl = 'http://localhost:8080', timeout = 30000) {
        this.baseUrl = baseUrl.replace(/\/$/, '');
        this.jsonrpcUrl = `${this.baseUrl}/jsonrpc`;
        this.timeout = timeout;
        this.requestId = 0;
    }

    /**
     * Get next request ID
     * @private
     */
    _getNextId() {
        return ++this.requestId;
    }

    /**
     * Make a JSON-RPC call
     * 
     * @private
     * @param {string} method - JSON-RPC method name
     * @param {object} [params] - Method parameters
     * @returns {Promise<any>} Method result
     * @throws {SongbirdError} If the call fails
     */
    async _call(method, params = null) {
        const payload = {
            jsonrpc: '2.0',
            method: method,
            id: this._getNextId()
        };

        if (params !== null) {
            payload.params = params;
        }

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);

        try {
            const response = await fetch(this.jsonrpcUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload),
                signal: controller.signal
            });

            clearTimeout(timeoutId);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const data = await response.json();

            if (data.error) {
                throw new SongbirdError(
                    data.error.code || -1,
                    data.error.message || 'Unknown error',
                    data.error.data
                );
            }

            return data.result;
        } catch (error) {
            clearTimeout(timeoutId);
            if (error.name === 'AbortError') {
                throw new Error(`Request timeout after ${this.timeout}ms`);
            }
            throw error;
        }
    }

    /**
     * Make multiple JSON-RPC calls in one HTTP request
     * 
     * @private
     * @param {Array<{method: string, params?: object}>} requests - List of requests
     * @returns {Promise<Array<any>>} List of results (may include SongbirdError objects)
     */
    async _batchCall(requests) {
        const payload = requests.map(req => ({
            jsonrpc: '2.0',
            method: req.method,
            params: req.params || null,
            id: this._getNextId()
        }));

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);

        try {
            const response = await fetch(this.jsonrpcUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload),
                signal: controller.signal
            });

            clearTimeout(timeoutId);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${response.statusText}`);
            }

            const results = await response.json();

            // Return results (converting errors to SongbirdError objects)
            return results.map(result => {
                if (result.error) {
                    return new SongbirdError(
                        result.error.code || -1,
                        result.error.message || 'Unknown error',
                        result.error.data
                    );
                }
                return result.result;
            });
        } catch (error) {
            clearTimeout(timeoutId);
            if (error.name === 'AbortError') {
                throw new Error(`Request timeout after ${this.timeout}ms`);
            }
            throw error;
        }
    }

    // ============================================
    // Health & Info Methods
    // ============================================

    /**
     * Check Songbird health
     * 
     * @returns {Promise<{status: string, version: string, uptime_seconds: number}>}
     * 
     * @example
     * const health = await client.health();
     * console.log(health.status); // 'healthy'
     */
    async health() {
        return this._call('songbird.health');
    }

    /**
     * Get Songbird version information
     * 
     * @returns {Promise<{version: string, name: string, commit?: string, build_date?: string}>}
     * 
     * @example
     * const version = await client.version();
     * console.log(version.version); // '0.2.1'
     */
    async version() {
        return this._call('songbird.version');
    }

    // ============================================
    // Protocol Methods
    // ============================================

    /**
     * Get available protocol capabilities
     * 
     * @returns {Promise<{protocols: object}>}
     * 
     * @example
     * const caps = await client.protocolCapabilities();
     * console.log(Object.keys(caps.protocols)); // ['http', 'jsonrpc', ...]
     */
    async protocolCapabilities() {
        return this._call('songbird.protocol.capabilities');
    }

    // ============================================
    // Service Management Methods
    // ============================================

    /**
     * List all registered services
     * 
     * @returns {Promise<Array<object>>}
     * 
     * @example
     * const services = await client.listServices();
     * services.forEach(s => console.log(`${s.name}: ${s.endpoint}`));
     */
    async listServices() {
        const result = await this._call('songbird.services.list');
        return result.services || [];
    }

    /**
     * Get details for a specific service
     * 
     * @param {string} serviceId - Service identifier
     * @returns {Promise<object>}
     * 
     * @example
     * const service = await client.getService('my-service-123');
     * console.log(service.endpoint);
     */
    async getService(serviceId) {
        return this._call('songbird.services.get', { service_id: serviceId });
    }

    /**
     * Register a new service with Songbird
     * 
     * @param {object} serviceConfig - Service configuration
     * @returns {Promise<object>}
     * 
     * @example
     * const result = await client.registerService({
     *     name: 'my-service',
     *     endpoint: 'http://localhost:3000',
     *     capabilities: ['compute', 'storage']
     * });
     */
    async registerService(serviceConfig) {
        return this._call('songbird.services.register', serviceConfig);
    }

    // ============================================
    // Compute Methods
    // ============================================

    /**
     * Schedule a compute task (integrates with Toadstool)
     * 
     * @param {object} taskConfig - Task configuration
     * @returns {Promise<{task_id: string}>}
     * 
     * @example
     * const result = await client.scheduleCompute({
     *     task: 'train_model',
     *     language: 'python',
     *     code: 'import numpy as np\nprint("Hello!")'
     * });
     * console.log(result.task_id);
     */
    async scheduleCompute(taskConfig) {
        return this._call('songbird.compute.schedule', taskConfig);
    }

    /**
     * Get status of a compute task
     * 
     * @param {string} taskId - Task identifier
     * @returns {Promise<{status: string, progress?: number, result?: any}>}
     * 
     * @example
     * const status = await client.getComputeStatus('task-123');
     * console.log(status.status); // 'completed', 'running', 'failed'
     */
    async getComputeStatus(taskId) {
        return this._call('songbird.compute.status', { task_id: taskId });
    }

    /**
     * Wait for a compute task to complete
     * 
     * @param {string} taskId - Task identifier
     * @param {number} pollInterval - Seconds between status checks (default: 1.0)
     * @param {number|null} timeout - Maximum wait time in seconds (default: null = forever)
     * @returns {Promise<object>} Final task status
     * @throws {Error} If timeout is exceeded
     * 
     * @example
     * const task = await client.scheduleCompute({...});
     * const result = await client.waitForCompute(task.task_id, 1, 300);
     * console.log(result.result);
     */
    async waitForCompute(taskId, pollInterval = 1.0, timeout = null) {
        const startTime = Date.now();

        while (true) {
            const status = await this.getComputeStatus(taskId);

            if (['completed', 'failed', 'cancelled'].includes(status.status)) {
                return status;
            }

            if (timeout && (Date.now() - startTime) / 1000 > timeout) {
                throw new Error(`Task ${taskId} did not complete within ${timeout} seconds`);
            }

            await new Promise(resolve => setTimeout(resolve, pollInterval * 1000));
        }
    }

    // ============================================
    // Federation Methods
    // ============================================

    /**
     * List all federation peers
     * 
     * @returns {Promise<Array<object>>}
     * 
     * @example
     * const peers = await client.listFederationPeers();
     * peers.forEach(p => console.log(`${p.id}: ${p.endpoint}`));
     */
    async listFederationPeers() {
        const result = await this._call('songbird.federation.peers');
        return result.peers || [];
    }

    /**
     * Join a federation by connecting to a peer
     * 
     * @param {object} peerConfig - Peer configuration
     * @returns {Promise<object>}
     * 
     * @example
     * const result = await client.joinFederation({
     *     peer_id: 'tower-a',
     *     endpoint: 'http://tower-a.example.com:8080'
     * });
     */
    async joinFederation(peerConfig) {
        return this._call('songbird.federation.join', peerConfig);
    }

    // ============================================
    // Batch Operations
    // ============================================

    /**
     * Execute multiple operations in one HTTP request
     * 
     * @param {Array<{method: string, params?: object}>} operations - List of operations
     * @returns {Promise<Array<any>>} List of results
     * 
     * @example
     * const [health, version, services] = await client.batch([
     *     { method: 'songbird.health' },
     *     { method: 'songbird.version' },
     *     { method: 'songbird.services.list' }
     * ]);
     */
    async batch(operations) {
        return this._batchCall(operations);
    }

    // ============================================
    // Utility Methods
    // ============================================

    /**
     * Quick health check
     * 
     * @returns {Promise<boolean>} True if healthy, false otherwise
     * 
     * @example
     * if (await client.isHealthy()) {
     *     console.log('Songbird is up!');
     * }
     */
    async isHealthy() {
        try {
            const health = await this.health();
            return health.status === 'healthy';
        } catch (error) {
            return false;
        }
    }
}

// ============================================
// Exports
// ============================================

// Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { SongbirdClient, SongbirdError };
}

// Browser
if (typeof window !== 'undefined') {
    window.SongbirdClient = SongbirdClient;
    window.SongbirdError = SongbirdError;
}

// ============================================
// Example Usage (Node.js)
// ============================================

if (typeof require !== 'undefined' && require.main === module) {
    (async () => {
        const client = new SongbirdClient('http://localhost:8080');

        // Example 1: Basic health check
        console.log('='.repeat(60));
        console.log('Example 1: Basic Health Check');
        console.log('='.repeat(60));

        try {
            const health = await client.health();
            console.log(`✅ Songbird is ${health.status}`);
            console.log(`   Version: ${health.version}`);
            console.log(`   Uptime: ${health.uptime_seconds} seconds`);
        } catch (error) {
            console.log(`❌ Failed to connect: ${error.message}`);
        }

        console.log();

        // Example 2: Get version info
        console.log('='.repeat(60));
        console.log('Example 2: Version Information');
        console.log('='.repeat(60));

        try {
            const version = await client.version();
            console.log(`Name: ${version.name}`);
            console.log(`Version: ${version.version}`);
            console.log(`Commit: ${version.commit || 'N/A'}`);
            console.log(`Build Date: ${version.build_date || 'N/A'}`);
        } catch (error) {
            console.log(`❌ Failed: ${error.message}`);
        }

        console.log();

        // Example 3: Protocol capabilities
        console.log('='.repeat(60));
        console.log('Example 3: Protocol Capabilities');
        console.log('='.repeat(60));

        try {
            const caps = await client.protocolCapabilities();
            const protocols = caps.protocols || {};
            console.log('Available Protocols:');
            for (const [name, info] of Object.entries(protocols)) {
                console.log(`  • ${name}: ${info.version || 'N/A'}`);
            }
        } catch (error) {
            console.log(`❌ Failed: ${error.message}`);
        }

        console.log();

        // Example 4: Batch operations
        console.log('='.repeat(60));
        console.log('Example 4: Batch Operations');
        console.log('='.repeat(60));

        try {
            const [health, version, caps] = await client.batch([
                { method: 'songbird.health' },
                { method: 'songbird.version' },
                { method: 'songbird.protocol.capabilities' }
            ]);

            console.log('✅ Executed 3 operations in 1 HTTP request');
            console.log(`   Health: ${health.status}`);
            console.log(`   Version: ${version.version}`);
            console.log(`   Protocols: ${Object.keys(caps.protocols).length} available`);
        } catch (error) {
            console.log(`❌ Failed: ${error.message}`);
        }

        console.log();

        // Example 5: Quick health check
        console.log('='.repeat(60));
        console.log('Example 5: Quick Health Check');
        console.log('='.repeat(60));

        if (await client.isHealthy()) {
            console.log('✅ Songbird is healthy and ready!');
        } else {
            console.log('❌ Songbird is not responding');
        }

        console.log();
        console.log('='.repeat(60));
        console.log('🎉 All examples complete!');
        console.log('='.repeat(60));
    })();
}

