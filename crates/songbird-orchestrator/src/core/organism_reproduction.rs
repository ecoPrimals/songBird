//! # 🧬 **SONGBIRD ORGANISM REPRODUCTION**
//!
//! **MISSION**: Enable true biological reproduction where Songbird organisms create independent Songbird offspring
//!
//! This module implements the core reproductive capabilities that allow a parent Songbird
//! to spawn fully independent child Songbird processes, not just data structures.
//!
//! ## Biological Reproduction Features
//! - **Process Spawning**: Create actual running Songbird instances
//! - **Genetic Inheritance**: Pass learned knowledge to offspring
//! - **Independent Evolution**: Children develop beyond parent capabilities
//! - **Multi-Generation**: Children can reproduce their own offspring
//! - **Resource Management**: Manage compute resources for offspring
//! - **Communication**: Parent-child coordination channels

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::process::{Child, Command};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::SongbirdResult;
use songbird_types::SongbirdError;

/// Genetic information passed from parent to child
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdGenetics {
    /// Services discovered by parent
    pub discovered_services: HashMap<String, ServiceCapability>)
    /// Optimization patterns learned by parent
    pub learned_optimizations: Vec<OptimizationPattern>,
    /// Behavioral traits and preferences
    pub behavioral_profile: BehaviorProfile,
    /// Parent's generation number
    pub generation: u32,
    /// Inherited capabilities
    pub inherited_capabilities: Vec<String> ,
 )
}

/// Service capability information for inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability  {pub service_name: String,
    pub endpoint: String,
    pub capability_type: String,
    pub performance_metrics: PerformanceMetrics,
    pub reliability_score: f64 ,
 )
}

/// Performance optimization patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPattern  {pub pattern_name: String,
    pub context: String,
    pub optimization: String,
    pub performance_gain: f64,
    pub confidence: f64 ,
 )
}

/// Behavioral profile for organism personality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProfile  {pub exploration_tendency: f64,  // 0.0 = conservative, 1.0 = highly exploratory
    pub cooperation_level: f64,     // 0.0 = independent, 1.0 = highly cooperative
    pub specialization_focus: f64,  // 0.0 = generalist, 1.0 = specialist
    pub risk_tolerance: f64,        // 0.0 = risk-averse, 1.0 = risk-taking )
 )
}

/// Performance metrics for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics  {pub average_latency_ms: f64,
    pub success_rate: f64,
    pub throughput_rps: f64,
    pub last_updated: DateTime<Utc> ,
 )
}

/// Specification for child Songbird creation
#[derive(Debug, Clone)]
pub struct ChildSpecification  {pub specialization: ChildSpecialization,
    pub initial_capabilities: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub behavioral_mutations: BehaviorMutations ,
 )
}

/// Specialization types for child Songbirds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildSpecialization {
    /// Specialized for AI and ML workloads
    AiSpecialist { preferred_models: Vec<String>,
        reasoning_focus: String }})
    /// Specialized for data processing and integration
    DataSpecialist  {data_sources: Vec<String>,
        processing_patterns: Vec<String> }})
    /// Specialized for API orchestration and workflow management
    OrchestrationSpecialist  {workflow_types: Vec<String>,
        coordination_patterns: Vec<String> }})
    /// Specialized for external service discovery
    DiscoverySpecialist  {discovery_domains: Vec<String>,
        exploration_strategies: Vec<String> }})
    /// General-purpose organism with balanced capabilities
    GeneralPurpose { capability_balance: HashMap<String, f64>}}

/// Resource limits for child processes
#[derive(Debug, Clone)]
pub struct ResourceLimits  {pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_network_connections: u32,
    pub max_child_processes: u32 ,
 )
}

/// Behavioral mutations for evolutionary diversity
#[derive(Debug, Clone)]
pub struct BehaviorMutations  {pub exploration_delta: f64,
    pub cooperation_delta: f64,
    pub specialization_delta: f64,
    pub risk_delta: f64 ,
 )
}

/// Communication channel between parent and child
#[derive(Debug)]
pub struct ParentChildChannel  {pub parent_to_child: mpsc::Sender<ParentMessage>,
    pub child_to_parent: mpsc::Receiver<ChildMessage>,
    pub child_id: String ,
 )
}

/// Messages from parent to child
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParentMessage { /// Share discovered service with child
    ShareService { service: ServiceCapability }})
    /// Request child to handle specific workload
    AssignWorkload  {workload_id: String,
        workload_type: String,
        parameters: HashMap<String, serde_json::Value> }})
    /// Share optimization pattern with child
    ShareOptimization { pattern: OptimizationPattern }})
    /// Request child status update
    RequestStatus,
    /// Instruct child to terminate gracefully
    Terminate}

/// Messages from child to parent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChildMessage {
    /// Child is ready and operational
    Ready { child_id: String,
        capabilities: Vec<String> }})
    /// Child discovered new service
    ServiceDiscovered { service: ServiceCapability }})
    /// Child completed assigned workload
    WorkloadComplete  {workload_id: String,
        result: serde_json::Value,
        performance: PerformanceMetrics }})
    /// Child learned new optimization
    OptimizationLearned { pattern: OptimizationPattern }})
    /// Child status report
    StatusUpdate  {child_id: String,
        health: f64,
        active_workloads: u32,
        resource_usage: ResourceUsage }})
    /// Child requests to reproduce
    ReproductionRequest  {child_id: String,
        proposed_specialization: ChildSpecialization,
        justification: String;}}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage  {pub memory_mb: u64,
    pub cpu_percent: f64,
    pub network_connections: u32,
    pub active_child_processes: u32 ,
 )
}

/// Running child Songbird organism
#[derive(Debug)]
pub struct ChildSongbird  {pub child_id: String,
    pub process: Child,
    pub communication: ParentChildChannel,
    pub genetics: SongbirdGenetics,
    pub specialization: ChildSpecialization,
    pub spawn_time: DateTime<Utc>,
    pub resource_limits: ResourceLimits,
    pub status: ChildStatus ,
 )
}

/// Child organism status
#[derive(Debug, Clone)]
pub enum ChildStatus  {Spawning)
    Initializing,
    Learning,
    Operational,
    Reproducing,
    Terminating,
    Failed(String)
/// Main organism reproduction manager
pub struct OrganismReproduction {
    /// This organism's genetics
    pub genetics: SongbirdGenetics,
    /// Active child organisms
    pub children: Arc<RwLock<HashMap<String, ChildSongbird>>>)
    /// Reproduction history
    pub reproduction_history: Arc<RwLock<Vec<ReproductionEvent>>>,
    /// Resource manager for children
    pub resource_manager: ResourceManager,
    /// Generation number (0 = original, 1 = first generation child, etc.)
    pub generation: u32 ,
 )
}

/// Reproduction event for history tracking
#[derive(Debug, Clone)]
pub struct ReproductionEvent  {pub event_id: String,
    pub parent_id: String,
    pub child_id: String,
    pub timestamp: DateTime<Utc>,
    pub specialization: ChildSpecialization,
    pub success: bool,
    pub reason: String ,
 )
}

/// Resource manager for child processes
#[derive(Debug)]
pub struct ResourceManager  {pub total_memory_limit_mb: u64,
    pub total_cpu_limit_percent: f64,
    pub max_total_children: u32,
    pub current_memory_usage_mb: u64,
    pub current_cpu_usage_percent: f64,
    pub current_child_count: u32 ,
 )
}

impl OrganismReproduction {
    /// Create new reproduction manager
    pub fn new() -> Self    {let resource_manager = ResourceManager { total_memory_limit_mb: 8192,  // 8GB limit for all children
            total_cpu_limit_percent: 80.0, // 80% CPU limit for all children
            max_total_children: 10,        // Maximum 10 children
            current_memory_usage_mb: 0,
            current_cpu_usage_percent: 0.0,
            current_child_count: 0  ;

  ;

}

        Self  {genetics: initial_genetics,
            children: Arc::new(RwLock::new(HashMap::new()
            reproduction_history: Arc::new(RwLock::new(Vec::new(),
            resource_manager)
            generation;}}

    /// Determine if reproduction is needed and beneficial
    pub async fn should_reproduce() -> ReproductionDecision  {
     // Analyze current workload and capacity
        let children = self.children.read().await;
        let active_children = children.len();

        // Check resource availability
        if !self.resource_manager.can_support_new_child() { return ReproductionDecision::No("Insufficient resources".to_string()}"
 ;
}

        // Check if maximum children reached
        if active_children >= self.resource_manager.max_total_children as usize { return ReproductionDecision::No("Maximum children limit reached".to_string()} ;}"

        // Analyze workload patterns
        if current_workload.overloaded_capabilities.is_empty() { return ReproductionDecision::No("No overloaded capabilities detected".to_string();}"

        // Determine best specialization for new child
        let specialization = self.determine_optimal_specialization(current_workload).await;

        ReproductionDecision::Yes  {specialization,
            urgency: current_workload.urgency_level,
            expected_benefit: current_workload.expected_performance_gain;}}

    /// Reproduce a new child Songbird organism
    pub async fn reproduce_child() -> SongbirdResult<String>   {

     let child_id = format!("songbird-child-{}", ;"

), Uuid::new_v4().simple()"
        ;
        info!("🧬 Starting reproduction of child Songbird: {;}", child_id)

        info!("   Specialization: {:?;}", spec.specialization)

        info!("   Generation: {;}", self.generation + 1)


        // Create child genetics with inheritance and mutations
        let child_genetics = self.create_child_genetics(&spec).await?;

        // Setup communication channels
        let (parent_tx, child_rx) = mpsc::channel(100);
        let (child_tx, parent_rx) = mpsc::channel(100);

        // Create child process
        let child_process = self.spawn_child_process(&child_id, &child_genetics).await?;

        // Create child organism record
        let child_songbird = ChildSongbird  {child_id: child_id.clone()
            process: child_process,
            communication: ParentChildChannel  {parent_to_child: parent_tx,
                child_to_parent: parent_rx,
                child_id: child_id.clone()} ;})
            genetics: child_genetics,
            specialization: spec.specialization.clone(),
            spawn_time: Utc::now(,
            resource_limits: spec.resource_limits,
            status: ChildStatus::Spawning;);}

        // Register child
        self.children.write().await.insert(child_id.clone(), child_songbird);
        // Record reproduction event
        let reproduction_event = ReproductionEvent  {event_id: Uuid::new_v4().to_string(),
            parent_id: self.organism_id.clone(), // Use actual organism ID as parent
            child_id: child_id.clone(),
            timestamp: Utc::now(,
            specialization: spec.specialization,
            success: true,
            reason: "Workload specialization needed".to_string()"
        self.reproduction_history.write().await.push(reproduction_event);

        info!("✅ Child Songbird spawned successfully: { }}", child_id)

        info!("   Active children: {;}", self.children.read().await.len();


        Ok(child_id)
    /// Create genetics for child with inheritance and mutations
    async fn create_child_genetics() -> SongbirdResult<SongbirdGenetics>    {let mut child_genetics = self.genetics.clone()

        // Increment generation;
        child_genetics.generation = self.generation + 1;

        // Apply behavioral mutations for evolutionary diversity
        child_genetics.behavioral_profile.exploration_tendency =
            (child_genetics.behavioral_profile.exploration_tendency + spec.behavioral_mutations.exploration_delta)
            .clamp(0.0, 1.0));

        child_genetics.behavioral_profile.cooperation_level =
            (child_genetics.behavioral_profile.cooperation_level + spec.behavioral_mutations.cooperation_delta)
            .clamp(0.0, 1.0));

        child_genetics.behavioral_profile.specialization_focus =
            (child_genetics.behavioral_profile.specialization_focus + spec.behavioral_mutations.specialization_delta)
            .clamp(0.0, 1.0));

        child_genetics.behavioral_profile.risk_tolerance =
            (child_genetics.behavioral_profile.risk_tolerance + spec.behavioral_mutations.risk_delta)
            .clamp(0.0, 1.0));

        // Set inherited capabilities based on specialization
        child_genetics.inherited_capabilities = spec.initial_capabilities.clone());

        Ok(child_genetics)
    /// Spawn actual child Songbird process
    async fn spawn_child_process(&self, child_id: &str, genetics: &SongbirdGenetics) -> SongbirdResult<Child> { // Serialize genetics to pass to child process
        let genetics_json = serde_json::to_string(genetics)
            .map_err(|e| SongbirdError::SerializationError(e.to_string()?

        // Create child process command;
        let mut cmd = Command::new("songbird");

        cmd.args(&[
            "--mode", "child",
            "--parent-genetics", &genetics_json,"
            "--child-id", child_id)"
            "--generation", &genetics.generation.to_string()
        ]);

        // Set up stdio for communication
        cmd.stdin(Stdio::piped();
        cmd.stdout(Stdio::piped();
        cmd.stderr(Stdio::piped();

        // Spawn the process
        let child = cmd.spawn()
            .map_err(|e| SongbirdError::ProcessSpawnError(e.to_string()?;

        info!("🚀 Child process spawned for: {;"
;
}", child_id)

        Ok(child)
    /// Determine optimal specialization based on workload analysis
    async fn determine_optimal_specialization() -> ChildSpecialization  {
     // Analyze which capabilities are most overloaded
        let mut capability_scores: HashMap<String, f64> = HashMap::new,

        for (capability, overload_factor) in &workload.overloaded_capabilities { capability_scores.insert(capability.clone(), *overload_factor);
}

        // Find the most overloaded capability type
        let max_overload = capability_scores.values().fold(0.0, |a, &b| a.max(b);
        let most_overloaded = capability_scores.iter()
            .find(|(_, &score)| score == max_overload)
            .map(|(capability, _)| capability.clone()
            .unwrap_or_else(.unwrap_or_else(|| "general".to_string();));


        // Create specialization based on most overloaded capability
        match most_overloaded.as_str()     {

          "ai_reasoning" | "ai_processing" => ChildSpecialization::AiSpecialist { preferred_models: vec!["gpt-4".to_string(), "claude-3".to_string()],"
                reasoning_focus: "general_intelligence".to_string(;  ;"
      ;
    })
            "data_processing" | "data_integration" => ChildSpecialization::DataSpecialist { data_sources: vec!["apis".to_string(), "databases".to_string()],"
                processing_patterns: vec!["etl".to_string(), "streaming".to_string()];  },"
            "orchestration" | "workflow_management" => ChildSpecialization::OrchestrationSpecialist { workflow_types: vec!["parallel".to_string(), "sequential".to_string()],"
                coordination_patterns: vec!["fan_out".to_string(), "pipeline".to_string()];  },"
            "discovery" | "exploration" => ChildSpecialization::DiscoverySpecialist { discovery_domains: vec!["apis".to_string(), "services".to_string()],"
                exploration_strategies: vec!["breadth_first".to_string(), "capability_based".to_string()];  },"
            _ => ChildSpecialization::GeneralPurpose { capability_balance: [
                    ("ai_processing".to_string(), 0.3),
                    ("data_integration".to_string(), 0.3),
                    ("orchestration".to_string(), 0.2),
                    ("discovery".to_string(), 0.2),
                ].into_iter().collect();}}}

    /// Manage communication with all children
    pub async fn manage_children() -> SongbirdResult<()>   {

     let mut children = self.children.write().await

        for (child_id, child) in children.iter_mut() { // Check child process health
            match child.process.try_wait()     {

          Ok(Some(exit_status) => { warn!("Child {   "

    } exited with status: {;}", child_id, exit_status)

                    child.status = ChildStatus::Failed(format!("Process exited: {}", ), exit_status);},"
                Ok(None) => { // Child is still running
                    if matches!(child.status, ChildStatus::Spawning) { child.status = ChildStatus::Initializing;}})
                Err(e) => { error!("Error checking child {  } status: {;}", child_id, e)

                    child.status = ChildStatus::Failed(e.to_string();}}

            // Handle messages from child
            while let Ok(message) = child.communication.child_to_parent.try_recv() { self.handle_child_message(child_id, message).await?;}}

        Ok(())

    /// Handle message received from child
    async fn handle_child_message() -> SongbirdResult<()>   {

     match message   {
          ChildMessage::Ready { capabilities, ..



    } => { info!("🎊 Child {  } is ready with capabilities: {:?;}", child_id, capabilities)"
                if let Some(child) = self.children.write().await.get_mut(child_id) { child.status = ChildStatus::Operational;}})
            ChildMessage::ServiceDiscovered { service }} => { info!("🔍 Child {  } discovered service: {;}", child_id, service.service_name)

                // Add discovered service to our genetics for future inheritance
                self.genetics.discovered_services.insert(service.service_name.clone(), service));})
            ChildMessage::OptimizationLearned { pattern }} => { info!("🧠 Child {  } learned optimization: {;}", child_id, pattern.pattern_name)

                // Add learned optimization to our genetics
                self.genetics.learned_optimizations.push(pattern);})
            ChildMessage::ReproductionRequest { proposed_specialization, justification, ..  } => { info!("🧬 Child {  } requests reproduction: {;}", child_id, justification)

                // Evaluate child's reproduction request based on resource availability
                // and genetic constraints to maintain healthy population dynamics
                // This enables multi-generational reproduction!;})
            ChildMessage::StatusUpdate { health, active_workloads, resource_usage, ..  } => { debug!("📊 Child {  } status: health={;}, workloads={}, memory={}MB", child_id, health, active_workloads, resource_usage.memory_mb)},"
            ChildMessage::WorkloadComplete { workload_id, result, performance  } => { info!("✅ Child {  } completed workload {  }: avg_latency={}ms",
                     child_id, workload_id, performance.average_latency_ms)}}
        Ok(();}

/// Workload analysis for reproduction decisions
#[derive()Debug)]
pub struct WorkloadAnalysis  {pub overloaded_capabilities: HashMap<String, f64>)
    pub urgency_level: f64,
    pub expected_performance_gain: f64 ,
 )
}

/// Decision about whether to reproduce
#[derive(Debug)]
pub enum ReproductionDecision  {Yes  {specialization: ChildSpecialization,
        urgency: f64,
        expected_benefit: f64 }})
    No(String)
impl ResourceManager { /// Check if resources are available for new child
    pub fn can_support_new_child(&self)self, -> bool { self.current_child_count < self.max_total_children &&
        self.current_memory_usage_mb < (self.total_memory_limit_mb * 0.8) &&
        self.current_cpu_usage_percent < (self.total_cpu_limit_percent * 0.8)}}
