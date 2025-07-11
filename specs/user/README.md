# Songbird Orchestrator - User Documentation

Welcome to the Songbird Orchestrator user documentation! This section contains everything you need to know to integrate and use the Songbird Orchestrator **Rust library** in your projects.

## 📚 Documentation Overview

### 🚀 Getting Started
- **[Getting Started Guide](GETTING_STARTED.md)** - Quick start guide for library integration
- **[Installation & Setup](INSTALLATION.md)** - Library installation and setup instructions

### 📖 Core Documentation  
- **[Architecture Overview](ARCHITECTURE.md)** - System architecture and design principles
- **[API Reference](API_REFERENCE.md)** - Complete API documentation for the library
- **[Production Guide](PRODUCTION_GUIDE.md)** - Production deployment best practices

## 🎯 Quick Navigation

### New Users Start Here:
1. [Installation Guide](INSTALLATION.md) - Add the library to your project
2. [Getting Started Guide](GETTING_STARTED.md) - Basic integration examples
3. [API Reference](API_REFERENCE.md) - Detailed API documentation

### Integration & Development:
1. [Architecture Overview](ARCHITECTURE.md) - Understanding the system design
2. [Production Guide](PRODUCTION_GUIDE.md) - Production deployment patterns

## 🏗️ System Requirements

- **Rust**: 1.70+ (for building and integration)
- **Operating System**: Linux, macOS, Windows
- **Memory**: 512MB minimum, 2GB recommended for your application
- **Network**: HTTP/HTTPS connectivity for service communication

## 🔧 What is Songbird Orchestrator?

Songbird Orchestrator is a **Rust library** that provides enterprise-grade service orchestration capabilities. It's designed to be integrated into your existing Rust projects to add:

- **Service Management**: Registration, lifecycle, health monitoring
- **Load Balancing**: Multiple algorithms with health-aware routing
- **Communication**: WebSocket and HTTP communication layers
- **Service Discovery**: Multiple backends (static, consul, kubernetes)
- **Robustness**: Circuit breakers, rate limiting, retries
- **Security**: Authentication, authorization, audit logging

## 📋 Key Features

### ✅ **Currently Available**
- **UniversalService Trait**: Standardized service interface
- **REST API Layer**: Complete HTTP endpoints for management
- **WebSocket Communication**: Real-time bidirectional communication
- **Load Balancing**: Round-robin, health-aware, least-connections
- **Health Monitoring**: Comprehensive health checking system
- **Configuration Management**: File-based and programmatic configuration
- **Metrics & Monitoring**: Prometheus-compatible metrics
- **Service Registry**: Central service management
- **Error Handling**: Comprehensive error system

### 🚀 **Working Examples**
- **API Demo**: Complete REST API demonstration
- **WebSocket Demo**: Real-time communication examples
- **Integration Examples**: Various service integration patterns

## 🤝 Community & Support

- **GitHub Issues**: [Report bugs and request features](https://github.com/songbird-project/songbird-orchestrator/issues)
- **Documentation Issues**: Found a problem with these docs? [Let us know](https://github.com/songbird-project/songbird-orchestrator/issues/new?labels=documentation)

## 📄 License

Songbird Orchestrator is released under the MIT License. See the [LICENSE](../../LICENSE) file for details.

---

*This documentation is for Songbird Orchestrator v0.1.0 - a Rust library for service orchestration.* 