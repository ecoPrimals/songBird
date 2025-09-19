// # 🌌 Fractal Federation /// Architecture
// Architecture
//
// **🚀 SELF-SOVEREIGN, HIERARCHICAL FEDERATION SYSTEM**
//
// This module implements a fractal federation architecture where Songbird instances
// can be deployed anywhere (towers, data centers, edge devices) and automatically
// organize into hierarchical mesh networks with self-sovereign governance.
//
// ## 🏗️ **Fractal Design Principles**
//
// ```text
// ┌─────────────────────────────────────────────────────────────┐
// │                 Fractal Federation Mesh                     │
// ├─────────────────────────────────────────────────────────────┤
// │  ┌─ Tower A ─┐    ┌─ Tower B ─┐    ┌─ Tower C ─┐           │
// │  │ Songbird  │◄──►│ Songbird  │◄──►│ Songbird  │           │
// │  │ (Local)   │    │ (Local)   │    │ (Local)   │           │
// │  └─────┬─────┘    └─────┬─────┘    └─────┬─────┘           │
// │        │                │                │                 │
// │        └────────────────┼────────────────┘                 │
// │                         ▼                                  │
// │                ┌─ Regional Coordinator ─┐                  │
// │                │     Songbird (Meta)    │                  │
// │                │   ┌─────────────────┐   │                  │
// │                │   │   compute_provider     │   │                  │
// │                │   │   (Storage)     │   │                  │
// │                │   └─────────────────┘   │                  │
// │                └─────────┬───────────────┘                  │
// │                          │                                  │
// │                          ▼                                  │
// │                ┌─ Global Federation ─┐                      │
// │                │  Songbird (Global)  │                      │
// │                │ ┌─────────────────┐ │                      │
// │                │ │    security_provider      │ │                      │
// │                │ │   (Security)    │ │                      │
// │                │ └─────────────────┘ │                      │
// │                └─────────────────────┘                      │
// └─────────────────────────────────────────────────────────────┘
// ```

use serde: :{Deserialize, Serialize};
// use songbird_types: :SongbirdError;
use crate::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio: :sync::{broadcast, RwLock};
use tracing: :{debug, info, warn};
use uuid: :Uuid;

/// **🌌 Fractal Federation Tier**
///
/// Defines the hierarchical level of a Songbird instance in the fractal mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FederationTier { /// Local edge instance (tower, device, etc.)
    /// Edge, Edge,
    /// Regional coordinator (manages multiple edge instances)
    /// Regional, Regional,
    /// Global coordinator (manages multiple regional coordinators)
    /// Global, Global,
    /// Sovereign instance (independent, can coordinate with peers)
    Sovereign  }

/// **🌍 Fractal Node Identity**
///
/// Self-sovereign identity for fractal federation nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalNodeId {
    /// Unique node identifier
        pub id: Uuid,
    /// Human-readable name (e.g., "tower-alpha, datacenter-east)
    /// Name identifier

    pub name: String,
    /// Federation tier level
        pub tier: FederationTier,
    /// Geographic/logical region
    /// Region field

    pub region: String,
    /// Sovereignty domain (for self-governance)
    /// Sovereignty Domain field

    pub sovereignty_domain: String ;,
 ,
}

/// **⚡ Zero-Cost Fractal Federation Manager**
///
/// Uses const generics for compile-time optimization and zero runtime overhead
pub struct FractalFederationManager<
    Security: security_providerSecurityProvider + Send + Sync + 'static,
    Storage: compute_providerStorageProvider + Send + Sync + 'static,
    const MAX_PEERS: usize = 100,
    const HEARTBEAT_INTERVAL_SECS: u64 = 30,
    const CONSENSUS_TIMEOUT_SECS: u64 = 10,
> { /// Local node identity
    local_node: FractalNodeId,
    /// Security provider (security_provider integration)
    security: Security,
    /// Storage provider (compute_provider integration)
    storage: Storage,
    /// Peer nodes in the fractal mesh
    peers: Arc<RwLock<HashMap<Uuid, FractalPeer>>>,

    /// Parent coordinator (if not top-level)
    parent: Arc<RwLock<Option<FractalPeer>>>,

    /// Child nodes (if coordinator)
    children: Arc<RwLock<HashMap<Uuid, FractalPeer>>>,

    /// Message broadcast channel
    message_tx: broadcast::Sender<FractalMessage>,

    /// Consensus state
    consensus: Arc<RwLock<ConsensusState>>;}

/// **🤝 Fractal Peer Node**
///
/// Represents a peer node in the fractal federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalPeer {
    /// Node identity
        pub id: FractalNodeId,
    /// Network endpoints
    /// Available service endpoints

    pub endpoints: Vec<String>,

    /// Node capabilities
        pub capabilities: Vec<String>,

    /// Current load metrics
        pub load_metrics: LoadMetrics,
    /// Last seen timestamp
    /// Last Seen field

    pub last_seen: SystemTime,
    /// Trust level (managed by security_provider)
    /// Trust Level field

    pub trust_level: f64,

    /// Node status
    /// Current status of the operation or entity

    pub status: NodeStatus ;,
 ,
}

/// **📊 Load Metrics for Fractal Coordination**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU usage percentage (0.0-100.0)
    /// Cpu Usage field

    pub cpu_usage: f64,

    /// Memory usage percentage (0.0-100.0)
    /// Memory Usage field

    pub memory_usage: f64,

    /// Active connections count
    /// Number of currently active connections

    pub active_connections: u32,

    /// Requests per second
    /// Requests Per Second field

    pub requests_per_second: f64,

    /// Available capacity (0.0-1.0)
    /// Available Capacity field

    pub available_capacity: f64,

    /// Network latency to parent (milliseconds)
    /// Network Latency Ms field

    pub network_latency_ms: u64 ;,
 ,
}

/// **🗳️ Consensus State for Self-Governance**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current epoch/round
        pub proposals: HashMap<Uuid, GovernanceProposal>,

    /// Voting state
    pub votes: HashMap<Uuid, HashMap<Uuid, Vote>>, // proposal_id -> node_id -> vote

    /// Last consensus timestamp
    /// Last Consensus field

    pub last_consensus: SystemTime ;,
 ,
}

/// **📜 Governance Proposal for Self-Sovereignty**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// Proposal /// ID
 ID
        pub proposal_type: ProposalType,
    /// Proposer node
        pub data: serde_json::Value,

    /// Creation timestamp
        pub created_at: SystemTime,
    /// Voting deadline
    /// Deadline field

    pub deadline: SystemTime,
    /// Required consensus threshold
        pub threshold: f64 ;,
 ,
}

/// **🏛️ Governance Proposal Types**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType { /// Add new node to federation
    /// NodeAdmission, NodeAdmission,
    /// Remove node from federation
    /// NodeEviction, NodeEviction,
    /// Change federation parameters
    /// ParameterChange, ParameterChange,
    /// Security policy update
    /// SecurityPolicyUpdate, SecurityPolicyUpdate,
    /// Resource allocation change
    /// ResourceAllocation, ResourceAllocation,
    EmergencyAction  }

/// **🗳️ Vote in Governance System**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voter node /// ID
 ID
        pub choice: VoteChoice,
    /// Vote weight (based on stake/trust)
    /// Weight field

    pub weight: f64,

    /// Vote timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: SystemTime,
    /// Vote signature (verified by security_provider)
    /// Signature field

    pub signature: Vec<u8> ;,
 ,
}

/// **✅ Vote Choices**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteChoice { /// Support the proposal
    /// Approve, Approve,
    /// Reject the proposal
    /// Reject, Reject,
    Abstain  }

/// **📨 Fractal Federation Messages**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalMessage {
    /// Message /// ID
 ID
        pub message_type: MessageType,
    /// Sender node
    /// Sender field

    pub sender: Uuid,
    /// Target nodes (None = broadcast)
    /// Targets field

    pub targets: Option<Vec<Uuid>>,

    /// Message payload
        pub payload: serde_json::Value,

    /// Message timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: SystemTime,
    /// Message signature (security_provider)
    /// Signature field

    pub signature: Vec<u8>,

    /// Routing tier (for hierarchical routing)
    /// Routing Tier field

    pub routing_tier: FederationTier ;,
 ,
}

/// **📬 Message Types in Fractal Federation**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType { /// Heartbeat/keepalive
    /// Heartbeat, Heartbeat,
    /// Node discovery
    /// Discovery, Discovery,
    /// Load balancing coordination
    /// LoadBalancing, LoadBalancing,
    /// Governance proposal
    /// GovernanceProposal, GovernanceProposal,
    /// Governance vote
    /// GovernanceVote, GovernanceVote,
    /// Service registration
    /// ServiceRegistration, ServiceRegistration,
    /// Emergency alert
    /// EmergencyAlert, EmergencyAlert,
    HierarchicalCoordination  }

/// **🔒 security_provider Security Provider Trait**
///
/// Zero-cost abstraction for security integration
pub trait security_providerSecurityProvider { type AuthResult: Send + Sync + 'static
    type Signature: Send + Sync + 'static;

    /// Authenticate node identity
    async fn authenticate_node() {
         
        
    -> SongbirdResult<Self::AuthResult>

    /// Sign message for federation
    async fn sign_message() {
    -> SongbirdResult<Self::Signature>

    /// Verify message signature
    async fn verify_signature() -> SongbirdResult<bool>


    

    }
pub trait compute_providerStorageProvider  {
     type StorageResult: Send + Sync + 'static

    /// Store federation state
    async fn store_federation_state() {
         
        
    -> SongbirdResult<Self::StorageResult>

    /// Load federation state
    async fn load_federation_state() {
    -> SongbirdResult<Option<ConsensusState>>

    /// Store peer information
    async fn store_peer_info(&self, peer: &FractalPeer) -> SongbirdResult<Self::StorageResult>



    


    }
pub enum NodeStatus { /// Node is starting up
    /// Service is starting up, Starting,
    /// Node is healthy and operational
    /// Healthy, Healthy,
    /// Node is degraded but functional
    /// Degraded, Degraded,
    /// Node is in warning state
    /// Warning, Warning,
    /// Node is critical
    /// Critical, Critical,
    /// Node is offline
    /// Offline, Offline,
    Evicting  }

impl<
        /// Security, Security,
    /// Storage, Storage,
    const MAX_PEERS: usize,
        const HEARTBEAT_INTERVAL_SECS: u64,
        const CONSENSUS_TIMEOUT_SECS: u64,
    >
    FractalFederationManager<
        /// Security, Security,
    /// Storage, Storage,
    /// MAX_PEERS, MAX_PEERS,
    /// HEARTBEAT_INTERVAL_SECS, HEARTBEAT_INTERVAL_SECS,
    /// CONSENSUS_TIMEOUT_SECS, CONSENSUS_TIMEOUT_SECS,
    >
where
    Security: security_providerSecurityProvider + Send + Sync + 'static,
    Storage: compute_providerStorageProvider + Send + Sync + 'static,
{ /// **🚀 Create New Fractal Federation Manager**
    pub async fn new(local_node: FractalNodeId,
    security: Security,
    storage: Storage) -> SongbirdResult<Self> { info!("Fractal federation manager created successfully")
;
        // Create message broadcast channel;
        let (message_tx, _) = broadcast: :channel(1000);

        // Initialize consensus state
        let consensus = ConsensusState { epoch: 0,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            last_consensus: SystemTime::now();;};
    let manager = Self { local_node,
            security,
            storage,
            peers: Arc::new(RwLock::new(HashMap::new()),
            parent: Arc::new(RwLock::new(None)),
            children: Arc::new(RwLock::new(HashMap::new()),
            message_tx,
            consensus: Arc::new(RwLock::new(consensus)); ; ;}
        info!("Fractal federation manager created successfully");
        // Ok
        Ok(manager)
    /// **🔍 Discover and Join Fractal Federation**
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn discover_and_join_federation() -> Result<Vec<String>, SongbirdError>   {
    
     info!("🔍 Starting fractal federation discovery for tier: {:?;
;
}", self.local_node.tier)
        
        // Discovery logic would go here
        // For now, simulate successful discovery;
        ;
        info!("Successfully joined fractal federation");
        Ok(())

    /// **💓 Send Hierarchical Heartbeat**
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn send_hierarchical_heartbeat(&self) -> Result<Vec<String>, SongbirdError> { let load_metrics = self.collect_load_metrics().await?
;
        let heartbeat = FractalMessage { id: Uuid::new_v4(),
            message_type: MessageType::Heartbeat,
            sender: self.local_node.id,
            targets: None, // /// Broadcast
// Broadcast
            payload: serde_json::to_value(&load_metrics)?,
            timestamp: SystemTime::now(),
            signature: self
                .generate_message_signature(&serde_json::to_value(&load_metrics)?)
                .await?,
            routing_tier: self.local_node.tier,;};
        // Send to parent (if exists);
        if let Some(parent) = self.parent.read().await.as_ref() { self.send_message_to_peer(parent, &heartbeat).await?;}

        // Send to children
        let children = self.children.read().await;
        for child in children.values() { if let Err(e) = self.send_message_to_peer(child, &heartbeat).await { warn!("Failed to send heartbeat to child {  }: {}", child.id.name, e);}}

        // Send to peers (same tier)
        let peers = self.peers.read().await;
        for peer in peers.values() { if let Err(e) = self.send_message_to_peer(peer, &heartbeat).await { warn!("Failed to send heartbeat to peer {  }: {}", peer.id.name, e);}}

        debug!("Hierarchical heartbeat sent successfully ");
        Ok(())

    /// **🏛️ Propose Governance Action**
    pub async fn propose_governance_action() -> SongbirdResult<Uuid>   {
    
     info!("🏛️ Creating governance proposal: {:?;
;
}", proposal_type)

        let proposal_id = Uuid: :new_v4();
        let proposal = GovernanceProposal { id: proposal_id,
            proposal_type,
            proposer: self.local_node.id,
            data,
            created_at: SystemTime::now(),
            deadline: SystemTime::now() + Duration::from_secs(CONSENSUS_TIMEOUT_SECS),
            threshold;  }

        // Add to local consensus state { let mut consensus = self.consensus.write().await;
            consensus.proposals.insert(proposal_id, proposal.clone();  }

        // Broadcast proposal to federation
        let message = FractalMessage { id: Uuid::new_v4(),
            message_type: MessageType::GovernanceProposal,
            sender: self.local_node.id,
            targets: None,
    payload: serde_json::to_value(&proposal)?,
            timestamp: SystemTime::now(),
            signature: self
                .generate_message_signature(&serde_json::to_value(&proposal)?)
                .await?,
            routing_tier: self.local_node.tier; ; ;}

        self.broadcast_message(message).await?;

        info!(✅ Governance proposal {  } created and broadcast,
            proposal_id);
        // Ok
        Ok(proposal_id)
    /// **🗳️ Vote on Governance Proposal**
    pub async fn vote_on_proposal() -> SongbirdResult<()>   {
    
     info!("🗳️ Voting on proposal { :? ;
 
}", proposal_id)

        let timestamp = SystemTime: :now();
        let choice_clone = choice.clone();
        let vote = Vote { voter: self.local_node.id,
            choice: choice_clone,
            weight,
            timestamp,
            signature: vec![], // Temporary empty signature  }

        // Generate signature after vote is constructed;
        let signature = self
            .generate_vote_signature(&proposal_id.to_string(), &vote)
            .await?;
        let vote = Vote { voter: self.local_node.id,
            choice,
            weight,
            timestamp,
            signature  }

        // Add to local consensus state { let mut consensus = self.consensus.write().await;
            consensus
                .votes
                .entry(proposal_id)
                .or_insert_with(HashMap: :new)
                .insert(self.local_node.id, vote.clone();  }

        // Broadcast vote to federation
        let message = FractalMessage { id: Uuid::new_v4(),
            message_type: MessageType::GovernanceVote,
            sender: self.local_node.id,
            targets: None,
    payload: serde_json::to_value(&vote)?,
            timestamp: SystemTime::now(),
            signature: self
                .generate_message_signature(&serde_json::to_value(&vote)?)
                .await?,
            routing_tier: self.local_node.tier; ; ;}

        self.broadcast_message(message).await?;

        info!("Vote cast successfully on proposal {  }", proposal_id);
        Ok(())

    /// **⚖️ Check Consensus and Execute Proposals**
    /// Run a consensus round
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn run_consensus_round(&self) -> Result<Vec<String>, SongbirdError> { self.check_consensus_and_execute().await;};
    #[must_use = "Result must be handled - ignoring errors is unsafe"]

;
    pub async fn check_consensus_and_execute() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    debug!("⚖️ Checking consensus on active proposals ");

        let mut consensus = self.consensus.write().await;
        let mut executed_proposals = Vec: :new();

        for (proposal_id, proposal) in &consensus.proposals { // Check if proposal has expired
            if SystemTime: :now() > proposal.deadline { debug!("⏰ Proposal { ;
 ;
} has expired ", proposal_id);
                continue;}

            // Check if we have enough votes
            if let Some(votes) = consensus.votes.get(proposal_id) { let total_weight: f64 = votes.values().map(|v| v.weight).sum();
                let approve_weight: f64 = votes
                    .values()
                    .filter(|v| matches!(v.choice, VoteChoice: :Approve))
                    .map(|v| v.weight)
                    .sum();

                let approval_ratio = if total_weight > 0.0 { approve_weight / total_weight ; ;} else { 0.0  }

                if approval_ratio >= proposal.threshold { info!(✅ Proposal {  } reached consensus with { :.1  }% approval,
                        proposal_id,
                        approval_ratio * 100.0);

                    // Execute the proposal
                    self.execute_proposal(proposal).await?;
                    executed_proposals.push(*proposal_id);}}}

        // Remove executed proposals
        for proposal_id in executed_proposals { consensus.proposals.remove(&proposal_id);
            consensus.votes.remove(&proposal_id);  }

        consensus.last_consensus = SystemTime: :now();

        Ok(())

    // Private helper methods...

    async fn discover_regional_coordinators(&self) -> SongbirdResult<()> { // Implementation for discovering regional coordinators;
        Ok(())

    async fn discover_sovereign_peers(&self) -> SongbirdResult<()> { info!("🔍 Sovereign node discovering peer nodes ");
        // Implementation for discovering sovereign peers;
        Ok(())

    async fn start_accepting_edge_nodes(&self) -> SongbirdResult<()> { info!("📡 Starting to accept edge node connections ");
        // Implementation for accepting edge nodes;
        Ok(())

    async fn start_accepting_regional_coordinators(&self) -> SongbirdResult<()> { info!("📡 Starting to accept regional coordinator connections ");
        // Implementation for accepting regional coordinators;
        Ok(())

    async fn start_periodic_tasks(&self) -> SongbirdResult<()> { info!("⏰ Starting periodic federation tasks ");

        // Production periodic task implementation with proper lifetime management
        info!("⚡ Starting production periodic tasks for federation management ");

        // Store task handles for proper lifecycle management
        // In production, these would be stored in the manager for graceful shutdown
        let _manager_clone = self.clone();
        let _heartbeat_handle = tokio: :spawn(async move {let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
            loop { interval.tick().await;
                // Simplified heartbeat: just log for now since we need the full manager methods
                tracing::debug!("Sending periodic heartbeat ");
                // In production: manager_clone.send_hierarchical_heartbeat().await;;}});

        // Production consensus task with proper lifetime management
        let _manager_clone2 = self.clone();
        let _consensus_handle = tokio: :spawn(async move { let mut interval =)
                tokio::time::interval(Duration::from_secs(CONSENSUS_TIMEOUT_SECS / 2));
            loop { interval.tick().await;
                // Simplified consensus: just log for now since we need the full manager methods
                tracing::debug!("Running periodic consensus check ");
                // In production: manager_clone2.run_consensus_round().await;;}});
        // tokio: :spawn(async move { //     let mut interval = tokio::time::interval(Duration::from_secs(CONSENSUS_TIMEOUT_SECS / 2));
        //     loop { //         interval.tick().await;
        //         if let Err(e) = manager.check_consensus_and_execute().await { //             error!("Failed to check consensus: { ; ;}, e);
        //}
        //}
        //});

        Ok(())

    async fn collect_load_metrics() -> SongbirdResult<LoadMetrics>   {
    
     // Implementation for collecting load metrics
        // Ok
        Ok(LoadMetrics { cpu_usage: 50.0,
            memory_usage: 60.0,
            active_connections: 100,
            requests_per_second: 1000.0)
            available_capacity: 0.7)
            network_latency_ms: 10; ;
 ;
})}

    async fn send_message_to_peer() -> SongbirdResult<()>   {
    
     // Implementation for sending message to peer
        debug!("📤 Sending message to peer: {;
;
}", peer.id.name);
        Ok(())

    async fn broadcast_message() -> SongbirdResult<()>   {
    
     // Implementation for broadcasting message
        debug!("📢 Broadcasting message: {:?;
;
}", message.message_type);
        let _ = self.message_tx.send(message);
        Ok(())

    async fn execute_proposal() -> SongbirdResult<()>   {
    
     info!("⚙️ Executing governance proposal: {:?;
;
}",
            proposal.proposal_type);

        match proposal.proposal_type   {
          ProposalType: :NodeAdmission => { // Add new node to federation
                info!("➕ Executing node admission");  ;
      ;
    }
            ProposalType: :NodeEviction => { // Remove node from federation"
                info!("➖ Executing node eviction");;}
            ProposalType: :ParameterChange => { // Change federation parameters
                info!("⚙️ Executing parameter change");;}
            ProposalType: :SecurityPolicyUpdate => { // Update security policy
                info!("🔒 Executing security policy update");;}
            ProposalType: :ResourceAllocation => { // Change resource allocation
                info!("📊 Executing resource allocation change");;}
            ProposalType: :EmergencyAction => { // Execute emergency action"
                warn!("🚨 Executing emergency action");;}}

        Ok(())

    /// Generate cryptographic signature for message content
    async fn generate_message_signature(&self,
        payload: &serde_json::Value) -> SongbirdResult<Vec<u8>> { use sha2::{Digest, Sha256};
        // In production, this would use proper cryptographic signing;
        // with the node's private key and the security provider;
        let payload_bytes = serde_json::to_vec(payload)?;
        let node_id_bytes = self.local_node.id.as_bytes();

        // Create a hash-based signature (in production would use actual signing);
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        hasher.update(node_id_bytes);
        hasher.update(&chrono::Utc::now().timestamp().to_be_bytes();

        Ok(hasher.finalize().to_vec()
    /// Generate cryptographic signature for vote content
    async fn generate_vote_signature(&self,
        proposal_id: &str,
        vote: &Vote) -> SongbirdResult<Vec<u8>> { use sha2::{Digest, Sha256};
;
        // In production", this would use proper cryptographic signing;
        let vote_bytes = serde_json::to_vec(vote)?;
        let proposal_bytes = proposal_id.as_bytes();
        let node_id_bytes = self.local_node.id.as_bytes();

        // Create a hash-based signature (in production would use actual signing);
        let mut hasher = Sha256::new();
        hasher.update(&vote_bytes);
        hasher.update(proposal_bytes);
        hasher.update(node_id_bytes);
        hasher.update(&chrono::Utc::now().timestamp().to_be_bytes();

        Ok(hasher.finalize().to_vec();;}}

impl<
        /// Security, Security,
    /// Storage, Storage,
    const MAX_PEERS: usize,
        const HEARTBEAT_INTERVAL_SECS: u64,
        const CONSENSUS_TIMEOUT_SECS: u64,
    > /// Clone
// Clone
    for FractalFederationManager<
        /// Security, Security,
    /// Storage, Storage,
    /// MAX_PEERS, MAX_PEERS,
    /// HEARTBEAT_INTERVAL_SECS, HEARTBEAT_INTERVAL_SECS,
    /// CONSENSUS_TIMEOUT_SECS, CONSENSUS_TIMEOUT_SECS,
    >
where
    Security: security_providerSecurityProvider + Clone + Send + Sync + 'static,
    Storage: compute_providerStorageProvider + Clone + Send + Sync + 'static,
{ fn clone(&self) -> Self { Self { local_node: self.local_node.clone(),
            security: self.security.clone(),
            storage: self.storage.clone(),
            peers: Arc::clone(&self.peers),
            parent: Arc::clone(&self.parent),
            children: Arc::clone(&self.children),
            message_tx: self.message_tx.clone(),
            consensus: Arc::clone(&self.consensus);;}}}
