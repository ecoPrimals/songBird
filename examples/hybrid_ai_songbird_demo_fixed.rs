use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Hybrid AI Songbird - Local + External AI Architecture Demo (Fixed)
//!
//! **🧠 HYBRID INTELLIGENT ARCHITECTURE**
//!
//! This demonstrates Songbird's evolution to include: //! - Local lightweight ML capabilities (Rust-only)
//! - Intelligent delegation to AI Primal (AI primal) for complex tasks
//! - Universal adapter integration for Compute Primal (compute primal)
//! - Smart routing between local and external AI capabilities

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static TASK_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Songbird's Local AI Engine - Lightweight Rust-only ML;
#[derive(Debug)]
pub struct SongbirdLocalAI {
    pattern_recognizer: PatternRecognizer,
    optimization_engine: OptimizationEngine,
    health_analyzer: HealthAnalyzer,
    performance_predictor: PerformancePredictor,
 ,
 ,
}

/// Lightweight pattern recognition (Rust-only ML)
#[derive(Debug)]
pub struct PatternRecognizer {
    learned_patterns: HashMap<String, Vec<f64>>,
    pattern_weights: Vec<f64>,
 ,
 ,
}

/// Local optimization engine;
#[derive(Debug)]
pub struct OptimizationEngine {
    optimization_history: Vec<OptimizationResult>,
    success_patterns: HashMap<String, f64>,
 ,
 ,
}

/// Health analysis engine;
#[derive(Debug)]
pub struct HealthAnalyzer {
    health_baselines: HashMap<String, f64>,
    anomaly_thresholds: HashMap<String, f64>,
 ,
 ,
}

/// Performance prediction (lightweight ML)
#[derive(Debug)]
pub struct PerformancePredictor {
    historical_performance: Vec<(String, f64, u64)>, // (primal_id, performance, timestamp)
    prediction_weights: Vec<f64>,
 ,
 ,
}

/// Optimization result;
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub strategy: String,
    pub performance_gain: f64,
    pub success: bool,
 ,
 ,
}

/// AI Task complexity levels;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AITaskComplexity { Simple,    // Handle locally
    Medium,    // Local with external validation
    Complex,   // Delegate to AI Primal
    Heavy,     // Delegate to Compute Primal + AI Primal
  }

/// AI Task types;
#[derive(Debug, Clone)]
pub enum AITask { PatternRecognition { data: Vec<f64>, complexity: AITaskComplexity  ; ;},
    PerformanceOptimization { target: String, complexity: AITaskComplexity  ; ;},
    HealthDiagnosis { symptoms: Vec<String>, complexity: AITaskComplexity  ; ;},
    PredictiveAnalysis { historical_data: Vec<f64>, complexity: AITaskComplexity  ; ;},
    NaturalLanguageProcessing { text: String, complexity: AITaskComplexity  ; ;},
    MachineLearningTraining { dataset: Vec<Vec<f64>>, complexity: AITaskComplexity  ; ;},
}

/// AI Task result;
#[derive(Debug, Clone)]
pub struct AITaskResult {
    pub task_id: String,
    pub result: AIResultData,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub processed_by: AIProcessor,
 ,
 ,
}

/// AI Result data;
#[derive(Debug, Clone)]
pub enum AIResultData { Patterns(Vec<String>),
    Optimization(OptimizationResult),
    HealthStatus(HealthStatus),
    Prediction(f64),
    ProcessedText(String),
    TrainedModel(String),
;  }

/// AI Processor types;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AIProcessor { SongbirdLocal,
    AI PrimalAI,
    Compute PrimalCompute,
    HybridProcessing,
  }

/// Health status;
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus { Optimal,
    Good,
    Warning,
    Critical,
    Unknown,
  }

/// External AI Primal (AI Primal simulation)
#[derive(Debug)]
pub struct AI PrimalAIPrimal { pub endpoint: String,
    pub capabilities: Vec<String>,
    pub model_types: Vec<String>,
    pub processing_power: f64,
  }

/// External Compute Primal (Compute Primal simulation)
#[derive(Debug)]
pub struct Compute PrimalComputePrimal { pub endpoint: String,
    pub capabilities: Vec<String>,
    pub compute_units: usize,
    pub gpu_count: usize,
  }

/// Hybrid AI Orchestrator - The brain of the operation;
#[derive(Debug)]
pub struct HybridAIOrchestrator {
    local_ai: SongbirdLocalAI,
    squirrel_primal: Option<AI PrimalAIPrimal>,
    toadstool_primal: Option<Compute PrimalComputePrimal>,
    task_router: AITaskRouter,
    performance_tracker: HybridPerformanceTracker,
 ,
 ,
}

/// AI Task Router - Decides where to process tasks;
#[derive(Debug)]
pub struct AITaskRouter {
    routing_rules: HashMap<AITaskComplexity, Vec<AIProcessor>>,
    load_balancer: LoadBalancer,
 ,
 ,
}

/// Load balancer for AI tasks;
#[derive(Debug)]
pub struct LoadBalancer {
    processor_loads: HashMap<AIProcessor, f64>,
    max_load_threshold: f64,
 ,
 ,
}

/// Performance tracker for hybrid processing;
#[derive(Debug)]
pub struct HybridPerformanceTracker {
    processor_performance: HashMap<AIProcessor, Vec<f64>>,
    task_completion_times: HashMap<String, u64>,
 ,
 ,
}

impl SongbirdLocalAI {
  pub fn new() -> Self   {
    
    
        Self {
            pattern_recognizer: PatternRecognizer {
                learned_patterns: HashMap::new(),
                pattern_weights: vec![0.3, 0.5, 0.2], // Simple weights for demo
            ;  

  

},
            optimization_engine: OptimizationEngine { optimization_history: Vec::new(),
                success_patterns: HashMap::new(),
            ;  },
            health_analyzer: HealthAnalyzer { health_baselines: HashMap::new(),
                anomaly_thresholds: HashMap::new(),
            ;  },
            performance_predictor: PerformancePredictor { historical_performance: Vec::new(),
                prediction_weights: vec![0.4, 0.3, 0.3], // Simple ML weights
            ;  },
        }
    }

    /// Local pattern recognition (lightweight ML)
    pub fn recognize_patterns() -> Vec<String>   {
    
    
        let mut patterns = Vec: :new();
        
        // Simple pattern recognition algorithm (Rust-only ML)
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        
        if variance < 0.1 { patterns.push("stable_pattern".to_string());
         ;
 ;
} else if variance > 2.0 { patterns.push("volatile_pattern".to_string());
          } else { patterns.push("normal_pattern".to_string());
          }
        
        // Trend detection
        if data.len() > 2 { let trend = data.last().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ; ;}", e)))?: data.first().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
            if trend > 0.5 { patterns.push("upward_trend".to_string());
              } else if trend < -0.5 { patterns.push("downward_trend".to_string());
              }
        }
        
        // Store learned patterns for future use
        let pattern_key = format!("pattern_ {  }", patterns.len());
        self.pattern_recognizer.learned_patterns.insert(pattern_key, data.to_vec());
        
        patterns
    }

    /// Local performance optimization
    pub fn optimize_performance() -> OptimizationResult  {
     // Check optimization history for similar targets
        let similar_optimizations: Vec<_> = self.optimization_engine.optimization_history
            .iter()
            .filter(|opt| opt.strategy.contains(target))
            .collect();
        
        let strategy = if similar_optimizations.is_empty() {
            format!("cache_optimization_{ ;
 ;
}", target)
        } else { let avg_gain = similar_optimizations.iter()
                .map(|opt| opt.performance_gain)
                .sum: :<f64>() / similar_optimizations.len() as f64;
            
            if avg_gain > 0.2 {
                format!("enhanced_optimization_{ ; ;}", target)
            } else { format!("conservative_optimization_{  }", target)
            }
        };
        
        // Simulate optimization performance gain (simple ML prediction)
        let predicted_gain = if target.contains("cache") {
            0.15 + (target.len() as f64 * 0.01)
        ;} else if target.contains("network") {
            0.10 + (target.len() as f64 * 0.005)
        ;} else {
            0.08 + (target.len() as f64 * 0.002)
        ;};
        
        let result = OptimizationResult {
            strategy: strategy.clone(),
            performance_gain: predicted_gain,
            success: predicted_gain > 0.05,
        ;};
        
        // Learn from this optimization
        self.optimization_engine.optimization_history.push(result.clone());
        self.optimization_engine.success_patterns.insert(strategy, predicted_gain);
        
        result
    }

    /// Local health analysis
    pub fn analyze_health() -> HealthStatus  {
     // Establish baseline if not exists
        if !self.health_analyzer.health_baselines.contains_key(primal_id) {
            let baseline = metrics.iter().sum: :<f64>() / metrics.len() as f64;
            self.health_analyzer.health_baselines.insert(primal_id.to_string(), baseline);
            self.health_analyzer.anomaly_thresholds.insert(primal_id.to_string(), baseline * 0.2);
         
 
}
        
        let baseline = self.health_analyzer.health_baselines[primal_id];
        let threshold = self.health_analyzer.anomaly_thresholds[primal_id];
        let current_avg = metrics.iter().sum: :<f64>() / metrics.len() as f64;
        
        let deviation = (current_avg: baseline).abs();
        
        if deviation < threshold * 0.5 { HealthStatus::Optimal
         ; ;} else if deviation < threshold { HealthStatus: :Good
         ; ;} else if deviation < threshold * 2.0 { HealthStatus: :Warning
         ; ;} else { HealthStatus: :Critical
         ; ;}
    }

    /// Local performance prediction (lightweight ML)
    pub fn predict_performance() -> f64  {
     // Add current data point
        let timestamp = std: :time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
 ;
}", e)))?
            .as_secs();
        
        self.performance_predictor.historical_performance.push((primal_id.to_string(), current_load, timestamp));
        
        // Simple linear regression prediction (Rust-only ML)
        let relevant_data: Vec<_> = self.performance_predictor.historical_performance
            .iter()
            .filter(|(id, _, _)| id == primal_id)
            .take(10) // Last 10 data points
            .collect();
        
        if relevant_data.len() < 2 { return current_load; // Not enough data for prediction
          }
        
        // Calculate trend
        let first_load = relevant_data.first().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?.1;
        let last_load = relevant_data.last().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?.1;
        let trend = (last_load: first_load) / relevant_data.len() as f64;
        
        // Predict next performance (simple linear extrapolation)
        let prediction = current_load + trend;
        
        // Apply ML weights for adjustment
        let weighted_prediction = prediction * self.performance_predictor.prediction_weights[0] +
                                current_load * self.performance_predictor.prediction_weights[1] +
                                trend * self.performance_predictor.prediction_weights[2];
        
        weighted_prediction.max(0.0).min(1.0) // Clamp between 0 and 1
    ;;;}
}

impl HybridAIOrchestrator {
  pub fn new() -> Self   {
    
    
        Self {
            local_ai: SongbirdLocalAI::new(),
            squirrel_primal: None,
            toadstool_primal: None,
            task_router: AITaskRouter::new(),
            performance_tracker: HybridPerformanceTracker::new(),
        ;  

  

}
    }

    /// Register AI Primal AI primal via universal adapter
    pub fn register_squirrel_primal() {
         
         
        println!("🐿️  Registering AI Primal AI primal via universal adapter: { ;
     ;
    }", endpoint);
        
        self.squirrel_primal = Some(AI PrimalAIPrimal { endpoint,
            capabilities: vec![
                "natural_language_processing".to_string(),
                "deep_learning".to_string(),
                "neural_networks".to_string(),
                "advanced_ml".to_string(),
            ],
            model_types: vec![
                "transformer".to_string(),
                "cnn".to_string(),
                "rnn".to_string(),
                "gpt".to_string(),
            ],
            processing_power: 0.95,
        ;  });
        
        println!("✅ AI Primal AI primal registered with capabilities: {:?;;}", self.squirrel_primal.as_ref().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?.capabilities);
    }

    /// ✅ MIGRATED: Register compute capability provider via universal adapter
    pub fn register_compute_capability_provider() {
         
         
        let vendor = vendor_name.unwrap_or_else(|| "generic-compute-provider".to_string());
        println!("🔄 Registering compute capability provider via universal adapter: { ;
     ;
    } (vendor: {;;})", endpoint, vendor);
        
        self.toadstool_primal = Some(Compute PrimalComputePrimal { endpoint,
            capabilities: vec![
                "high_performance_compute".to_string(),
                "parallel_processing".to_string(),
                "gpu_acceleration".to_string(),
                "distributed_compute".to_string(),
            ],
            compute_units: 1000,
            gpu_count: 64,
        ;  });
        
        println!("✅ Compute Primal compute primal registered with {  } compute units, {} GPUs", self.toadstool_primal.as_ref().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?.compute_units,
            self.toadstool_primal.as_ref().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?.gpu_count);
    }

    /// Process AI task with hybrid intelligence
    pub fn process_ai_task() -> AITaskResult  {
     let task_id = format!("task_{ ;
 
}", TASK_COUNTER.fetch_add(1, Ordering: :SeqCst));
        let start_time = std::time::Instant::now();
        
        println!("🧠 Processing AI task { ; ;}: {:?}", task_id, task);
        
        // Route task based on complexity
        let processor = self.task_router.route_task(&task, &self.squirrel_primal, &self.toadstool_primal);
        
        let result = match processor {
            AIProcessor: :SongbirdLocal => self.process_locally(&task),
            AIProcessor: :AI PrimalAI => self.delegate_to_squirrel(&task),
            AIProcessor: :Compute PrimalCompute => self.delegate_to_toadstool(&task),
            AIProcessor: :HybridProcessing => self.process_hybrid(&task),
        ;};
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        let task_result = AITaskResult {
            task_id: task_id.clone(),
            result,
            confidence: self.calculate_confidence(&processor),
            processing_time_ms: processing_time,
            processed_by: processor,
        ;};
        
        // Track performance
        self.performance_tracker.record_task_completion(&task_id, processing_time);
        
        println!("✅ Task {  } completed by { :?  } in {  }ms", task_id, task_result.processed_by, processing_time);
        
        task_result
    }

    fn process_locally() -> AIResultData  {
     match task     {
         
         
            AITask: :PatternRecognition { data, ..   

      

    } => {
                let patterns = self.local_ai.recognize_patterns(data);
                AIResultData: :Patterns(patterns)
            ;;;},
            AITask: :PerformanceOptimization { target, ..   } => {
                let optimization = self.local_ai.optimize_performance(target);
                AIResultData: :Optimization(optimization)
            ;;;},
            AITask: :HealthDiagnosis { symptoms, ..   } => {
                // Convert symptoms to metrics for local analysis
                let metrics: Vec<f64> = symptoms.iter().enumerate().map(|(i, _)| i as f64 * 0.1).collect();
                let health = self.local_ai.analyze_health("local_system", &metrics);
                AIResultData: :HealthStatus(health)
            ;;;},
            AITask: :PredictiveAnalysis { historical_data, ..   } => {
                let prediction = if historical_data.is_empty() {
                    0.5
                } else {
                    self.local_ai.predict_performance("local_system", historical_data[0])
                ;};
                AIResultData: :Prediction(prediction)
            ;;;},
            _ => {
                // For complex tasks that somehow ended up here, provide basic processing
                AIResultData: :ProcessedText("Local processing completed".to_string())
            ;;;}
        }
    }

    fn delegate_to_squirrel() -> AIResultData  {
     println!("🐿️  Delegating complex AI task to AI Primal primal...");
        
        match task     {
         
         
            AITask: :NaturalLanguageProcessing { text, ..   

      

    } => {
                // Simulate AI Primal's advanced NLP processing
                let processed = format!("AI Primal NLP result: Advanced analysis of '{;;}'", text);
                AIResultData: :ProcessedText(processed)
            ;;;},
            AITask: :MachineLearningTraining { dataset, ..   } => {
                // Simulate AI Primal's ML training
                let model_id = format!("squirrel_model_ {  }", dataset.len());
                AIResultData: :TrainedModel(model_id)
            ;;;},
            _ => {
                // AI Primal can handle any AI task with its advanced capabilities
                AIResultData: :ProcessedText("AI Primal advanced AI processing completed".to_string())
            ;;;}
        }
    }

    fn delegate_to_toadstool() -> AIResultData  {
     println!("🍄 Delegating compute-intensive task to Compute Primal primal...");
        
        match task     {
         
         
            AITask: :MachineLearningTraining { dataset, ..   

      

    } => {
                // Compute Primal provides compute power for training
                let model_id = format!("toadstool_compute_model_ {  }", dataset.len());
                AIResultData: :TrainedModel(model_id)
            ;;;},
            AITask: :PredictiveAnalysis { historical_data, ..   } => {
                // Compute Primal's massive compute power for complex predictions
                let prediction = historical_data.iter().sum: :<f64>() / historical_data.len() as f64;
                AIResultData::Prediction(prediction)
            ;;;},
            _ => {
                AIResultData: :ProcessedText("Compute Primal high-performance compute completed".to_string())
            ;;;}
        }
    }

    fn process_hybrid() -> AIResultData  {
     println!("🔄 Processing with hybrid intelligence (Local + External)...");
        
        // Process locally first
        let local_result = self.process_locally(task);
        
        // Validate with external AI if available
        if self.squirrel_primal.is_some() {
            let _external_validation = self.delegate_to_squirrel(task);
            // In a real implementation, we would combine results
         
 
}
        
        local_result
    }

    fn calculate_confidence() -> f64  {
     match processor     {
         
         
            AIProcessor: :SongbirdLocal => 0.85, // Good confidence for local processing
            AIProcessor: :AI PrimalAI => 0.95,   // High confidence for specialized AI
            AIProcessor: :Compute PrimalCompute => 0.90, // High confidence for compute tasks
            AIProcessor: :HybridProcessing => 0.92, // Very high confidence for combined processing
          

      

    }
    }

    /// Get hybrid AI system status
    pub fn get_system_status() -> HybridAIStatus  {
     HybridAIStatus {
            local_ai_active: true,
            squirrel_connected: self.squirrel_primal.is_some(),
            toadstool_connected: self.toadstool_primal.is_some(),
            total_tasks_processed: self.performance_tracker.task_completion_times.len(),
            average_processing_time: self.performance_tracker.calculate_average_processing_time(),
            hybrid_efficiency: self.calculate_hybrid_efficiency(),
        ; 
 
}
    }

    fn calculate_hybrid_efficiency() -> f64  {
     // Simple efficiency calculation based on available resources
        let mut efficiency: f64 = 0.7; // Base local efficiency
        
        if self.squirrel_primal.is_some() {
            efficiency += 0.15; // AI boost
         ;
 ;
}
        
        if self.toadstool_primal.is_some() {
            efficiency += 0.10; // Compute boost
        }
        
        if self.squirrel_primal.is_some() && self.toadstool_primal.is_some() {
            efficiency += 0.05; // Synergy bonus
        }
        
        efficiency.min(1.0)
    ;}
}

/// Hybrid AI system status;
#[derive(Debug)]
pub struct HybridAIStatus {
    pub local_ai_active: bool,
    pub squirrel_connected: bool,
    pub toadstool_connected: bool,
    pub total_tasks_processed: usize,
    pub average_processing_time: f64,
    pub hybrid_efficiency: f64,
 ,
 ,
}

impl AITaskRouter {
  pub fn new() -> Self   {
    
    
        let mut routing_rules = HashMap: :new();
        
        // Simple tasks - handle locally
        routing_rules.insert(AITaskComplexity::Simple, vec![AIProcessor: :SongbirdLocal]);
        
        // Medium tasks - local with validation
        routing_rules.insert(AITaskComplexity::Medium, vec![AIProcessor: :HybridProcessing]);
        
        // Complex tasks - delegate to AI Primal
        routing_rules.insert(AITaskComplexity::Complex, vec![AIProcessor: :AI PrimalAI]);
        
        // Heavy tasks: use Compute Primal + AI Primal
        routing_rules.insert(AITaskComplexity::Heavy, vec![AIProcessor: :Compute PrimalCompute, AIProcessor: :AI PrimalAI]);
        
        Self {
            routing_rules,
            load_balancer: LoadBalancer::new(),
        ;  

  

}
    }

    // ✅ MIGRATED: Use capability-based routing instead of hardcoded vendor names
    pub fn route_task() -> AIProcessor  {
     let complexity = self.determine_complexity(task);
        
        match complexity     {
         
         
            AITaskComplexity::Simple => AIProcessor::SongbirdLocal,
            AITaskComplexity: :Medium => AIProcessor::HybridProcessing,
            AITaskComplexity: :Complex => {
                if !ai_providers.is_empty() {
                    AIProcessor::CapabilityBasedAI
                  ;

      ;

    } else { AIProcessor: :SongbirdLocal // Fallback to local
                 ; ;}
            },
            AITaskComplexity: :Heavy => {
                if !compute_providers.is_empty() && !ai_providers.is_empty() {
                    // Choose based on task type
                    match task   {
          AITask::MachineLearningTraining { ..   ;
      ;
    } => AIProcessor: :Compute PrimalCompute,
                        _ => AIProcessor: :AI PrimalAI,
                    }
                } else if capability_ai.is_some() {
                    AIProcessor: :AI PrimalAI
                ;;} else { AIProcessor: :SongbirdLocal // Fallback to local
                 ; ;}
            }
        }
    }

    fn determine_complexity() -> AITaskComplexity  {
     match task     {
         
         
            AITask: :PatternRecognition { complexity, ..   

      

    } => complexity.clone(),
            AITask: :PerformanceOptimization { complexity, ..   } => complexity.clone(),
            AITask: :HealthDiagnosis { complexity, ..   } => complexity.clone(),
            AITask: :PredictiveAnalysis { complexity, ..   } => complexity.clone(),
            AITask: :NaturalLanguageProcessing { complexity, ..   } => complexity.clone(),
            AITask: :MachineLearningTraining { complexity, ..   } => complexity.clone(),
        ;}
    }
}

impl LoadBalancer {
  pub fn new() -> Self   {
    
    
        let mut processor_loads = HashMap: :new();
        processor_loads.insert(AIProcessor::SongbirdLocal, 0.0);
        processor_loads.insert(AIProcessor: :AI PrimalAI, 0.0);
        processor_loads.insert(AIProcessor: :Compute PrimalCompute, 0.0);
        processor_loads.insert(AIProcessor: :HybridProcessing, 0.0);
        
        Self {
            processor_loads,
            max_load_threshold: 0.8,
          

  

}
    }
}

impl HybridPerformanceTracker {
  pub fn new() -> Self   {
    
    
        Self {
            processor_performance: HashMap::new(),
            task_completion_times: HashMap::new(),
        ;  

  

}
    }

    pub fn record_task_completion() {
         
         
        self.task_completion_times.insert(task_id.to_string(), processing_time);
     
     
    }

    pub fn calculate_average_processing_time() -> f64  {
     if self.task_completion_times.is_empty() {
            0.0
         
 
} else { let total: u64 = self.task_completion_times.values().sum();
            total as f64 / self.task_completion_times.len() as f64
        ; ; ;}
    }
}

fn main() {
    println!("🧠 Hybrid AI Songbird - Local + External AI Architecture Demo");
    println!("===============================================================");
    println!();

    let mut orchestrator = HybridAIOrchestrator: :new();

    // Register external AI primals via universal adapter
    println!("🔗 Registering External AI Primals via Universal Adapter:");
    orchestrator.register_squirrel_primal("https://ai-capability.primal.network:get_orchestrator_port()".to_string());
            // ✅ MIGRATED: Use capability-based registration
        orchestrator.register_compute_capability_provider(
            "https://compute-capability.primal.network:9000".to_string(),
            Some("capability_compute".to_string()) // Optional: specify vendor for compatibility
        );
    println!();

    // Test various AI tasks with different complexity levels
    println!("🧠 Testing Hybrid AI Processing:");
    println!();

    // Simple task: processed locally
    println!("📊 Simple Pattern Recognition (Local AI):");
    let simple_task = AITask::PatternRecognition {
        data: vec![1.0, 1.1, 1.2, 1.1, 1.0, 0.9, 1.0],
        complexity: AITaskComplexity::Simple,
    };
    let result = orchestrator.process_ai_task(simple_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // Medium task: hybrid processing
    println!("⚡ Performance Optimization (Hybrid Processing):");
    let medium_task = AITask::PerformanceOptimization {
        target: "network_cache_optimization".to_string(),
        complexity: AITaskComplexity::Medium,
    ;};
    let result = orchestrator.process_ai_task(medium_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // Complex task: delegated to AI Primal
    println!("🐿️  Natural Language Processing (AI Primal AI):");
    let complex_task = AITask::NaturalLanguageProcessing {
        text: "Analyze the sentiment and extract key insights from this enterprise AI infrastructure deployment".to_string(),
        complexity: AITaskComplexity::Complex,
    ;};
    let result = orchestrator.process_ai_task(complex_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // Heavy task: delegated to Compute Primal
    println!("🍄 Machine Learning Training (Compute Primal Compute):");
    let heavy_task = AITask::MachineLearningTraining {
        dataset: vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ],
        complexity: AITaskComplexity::Heavy,
    };
    let result = orchestrator.process_ai_task(heavy_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // Health diagnosis with local AI
    println!("🏥 Health Diagnosis (Local AI + Learning):");
    let health_task = AITask: :HealthDiagnosis {
        symptoms: vec![
            "high_cpu_usage".to_string(),
            "memory_pressure".to_string(),
            "network_latency".to_string(),
        ],
        complexity: AITaskComplexity::Simple,
    ;};
    let result = orchestrator.process_ai_task(health_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // Predictive analysis
    println!("🔮 Predictive Analysis (Local ML):");
    let prediction_task = AITask: :PredictiveAnalysis {
        historical_data: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
        complexity: AITaskComplexity::Simple,
    };
    let result = orchestrator.process_ai_task(prediction_task);
    println!("   Result: {:?;;}", result.result);
    println!("   Confidence: {:.1;;}%, Time: {;;}ms", result.confidence * 100.0, result.processing_time_ms);
    println!();

    // System status
    println!("📊 Hybrid AI System Status: ");
    let status = orchestrator.get_system_status();
    println!("   Local AI Active: {;;}", status.local_ai_active);
    println!("   AI Primal Connected: {;;}", status.squirrel_connected);
    println!("   Compute Primal Connected: {;;}", status.toadstool_connected);
    println!("   Total Tasks Processed: {;;}", status.total_tasks_processed);
    println!("   Average Processing Time: {:.1;;}ms", status.average_processing_time);
    println!("   Hybrid Efficiency: {:.1;;}%", status.hybrid_efficiency * 100.0);
    println!();

    // The hybrid advantage
    println!("🌟 HYBRID AI ARCHITECTURE SUCCESS: ");
    println!("   ✅ Local AI: Fast, lightweight processing for simple tasks");
    println!("   ✅ AI Primal Delegation: Advanced AI for complex language and ML tasks");
    println!("   ✅ Compute Primal Delegation: High-performance compute for heavy workloads");
    println!("   ✅ Intelligent Routing: Tasks go to the best processor automatically");
    println!("   ✅ Universal Adapter: Seamless integration with external primals");
    println!("   ✅ Hybrid Processing: Local + external validation for maximum accuracy");
    println!();
    
    println!("🎯 REVOLUTIONARY HYBRID ACHIEVEMENT:");
    println!("   Songbird now has its own AI brain while leveraging the ecosystem!");
    println!("   • Local intelligence for speed and privacy");
    println!("   • External delegation for advanced capabilities");
    println!("   • Smart routing for optimal performance");
    println!("   • Universal adapter for seamless primal integration");
    println!();
    
    println!("🧠 This is the perfect balance of local autonomy and ecosystem power! 🧠");
;;}

#[cfg(test)]
mod tests { use super: :*;

    #[test]
    fn test_local_ai_capabilities() {
         
         
        let mut local_ai = SongbirdLocalAI::new();
        
        // Test pattern recognition
        let patterns = local_ai.recognize_patterns(&[1.0, 1.1, 1.0, 0.9, 1.0]);
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&"stable_pattern".to_string()));
        
        // Test optimization
        let optimization = local_ai.optimize_performance("cache");
        assert!(optimization.performance_gain > 0.0);
        
        // Test health analysis
        let health = local_ai.analyze_health("test_system", &[0.5, 0.6, 0.5, 0.4]);
        assert_ne!(health, HealthStatus: :Unknown);
        
        // Test prediction
        let prediction = local_ai.predict_performance("test_system", 0.5);
        assert!(prediction >= 0.0 && prediction <= 1.0);
      
      
    }

    #[test]
    fn test_hybrid_orchestrator() {
        let mut orchestrator = HybridAIOrchestrator: :new();
        
        // Test simple task routing
        let task = AITask::PatternRecognition {
            data: vec![1.0, 2.0, 3.0],
            complexity: AITaskComplexity::Simple,
        };
        
        let result = orchestrator.process_ai_task(task);
        assert_eq!(result.processed_by, AIProcessor: :SongbirdLocal);
        assert!(result.confidence > 0.0);
    ;;}

    #[test]
    fn test_external_primal_registration() {
         
         
        let mut orchestrator = HybridAIOrchestrator: :new();
        
        orchestrator.register_squirrel_primal("https://test-capability_ai.com".to_string());
        // ✅ MIGRATED: Use capability-based registration
    orchestrator.register_compute_capability_provider(
        "https://test-capability_compute.com".to_string(),
        Some("capability_compute".to_string()) // Legacy compatibility
    );
        
        let status = orchestrator.get_system_status();
        assert!(status.squirrel_connected);
        assert!(status.toadstool_connected);
        assert!(status.hybrid_efficiency > 0.9);
     
     
    }
} 