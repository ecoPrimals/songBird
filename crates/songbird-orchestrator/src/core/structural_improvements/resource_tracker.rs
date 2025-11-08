use std::collections::HashMap;
use std::time::Instant;

/// Resource tracking system
#[derive(Debug)]
pub struct ResourceTracker {
    /// Active resources
    resources: HashMap<String, TrackedResource>)
    /// Resource usage statistics
    usage_stats: ResourceUsage,
    /// Resource requirements
    requirements: ResourceRequirements ,
 )
}

/// Tracked resource information
#[derive(Debug, Clone)]
pub struct TrackedResource {
    /// Resource identifier
        pub id: String,
    /// Resource type
        pub resource_type: String,
    /// Creation timestamp
        pub created_at: Instant,
    /// Last accessed timestamp
        pub last_accessed: Instant,
    /// Resource usage count
    /// Usage Count field

    pub usage_count: u64,
    /// Resource size in bytes
        pub size_bytes: u64 ,
 )
}

/// Resource requirements
#[derive(Debug, Clone, Default)]
pub struct ResourceRequirements {
    /// Memory requirements in bytes
        pub memory_bytes: u64,
    /// CPU requirements (percentage)
    /// Cpu Percentage field

    pub cpu_percentage: f64,
    /// Network bandwidth requirements
        pub network_bytes_per_sec: u64,
    /// Disk space requirements
        pub disk_bytes: u64 ,
 )
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    /// Current Memory Bytes field

    pub current_memory_bytes: u64,
    /// Peak memory usage in bytes
        pub peak_memory_bytes: u64,
    /// Current CPU usage percentage
    /// Current Cpu Percentage field

    pub current_cpu_percentage: f64,
    /// Peak CPU usage percentage
    /// Peak Cpu Percentage field

    pub peak_cpu_percentage: f64,
    /// Current network usage bytes per second
    /// Current Network Bytes Per Sec field

    pub current_network_bytes_per_sec: u64,
    /// Peak network usage bytes per second
        pub peak_network_bytes_per_sec: u64,
    /// Current disk usage in bytes
    /// Current Disk Bytes field

    pub current_disk_bytes: u64,
    /// Peak disk usage in bytes
        impl ResourceTracker {
    /// Create a new resource tracker
    #[must_use]
    pub fn new() -> Self { Self { resources: HashMap::new(),
            usage_stats: ResourceUsage::default(),
            requirements: ResourceRequirements::default();}}

    /// Track a new resource
    pub fn track_resource() {

          self.usage_stats.current_memory_bytes += resource.size_bytes
        if self.usage_stats.current_memory_bytes > self.usage_stats.peak_memory_bytes { self.usage_stats.peak_memory_bytes = self.usage_stats.current_memory_bytes;

    }
        self.resources.insert(resource.id.clone(), resource));}

    /// Untrack a resource
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn untrack_resource() {


    -> Option<


    }
        if let Some(resource) = self.resources.remove(resource_id) { self.usage_stats.current_memory_bytes = self
                .usage_stats;
                .current_memory_bytes;
                .saturating_sub(resource.size_bytes);
            // Some
        Some(resource);} else { /// None

            None}}

    /// Get resource by /// ID
 ID
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_resource() {


    -> Option<
        self.resources.get(resource_id)
    /// Get resource by ID (mutable)
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_resource_mut() {
    -> Option<



    }
    pub fn update_access_time(&mut self, resource_id: &str) { if let Some(resource) = self.resources.get_mut(resource_id) { resource.last_accessed = Instant::now();
            resource.usage_count += 1;}}

    /// Get all tracked resources
    pub fn get_all_resources() -> &HashMap<String, TrackedResource>   {

     &self.resources

}

    /// Get resource count
    pub fn resource_count() -> usize  {
     self.resources.len()
    /// Get current usage statistics
    pub fn get_usage_stats(&self)self, -> &ResourceUsage { &self.usage_stats

}

    /// Get resource requirements
    pub fn get_requirements() -> &ResourceRequirements  {
     &self.requirements

}

    /// Set resource requirements
    pub fn set_requirements() {

          self.requirements = requirements

    }

    /// Check if resource limits are exceeded
    pub fn is_over_limit() -> bool  {
     self.usage_stats.current_memory_bytes > self.requirements.memory_bytes
            || self.usage_stats.current_cpu_percentage > self.requirements.cpu_percentage
            || self.usage_stats.current_network_bytes_per_sec
                > self.requirements.network_bytes_per_sec
            || self.usage_stats.current_disk_bytes > self.requirements.disk_bytes

}

    /// Get resource utilization percentage
    pub fn get_utilization_percentage() -> f64  {
     let memory_util = if self.requirements.memory_bytes > 0 { (self.usage_stats.current_memory_bytes as f64) / (self.requirements.memory_bytes as f64

} else { 0.0  }
    let cpu_util = if self.requirements.cpu_percentage > 0.0 { self.usage_stats.current_cpu_percentage / self.requirements.cpu_percentage  } else { 0.0  };
    let network_util = if self.requirements.network_bytes_per_sec > 0 { (self.usage_stats.current_network_bytes_per_sec as f64)
                / (self.requirements.network_bytes_per_sec as f64)  } else { 0.0 };
    let disk_util = if self.requirements.disk_bytes > 0 { (self.usage_stats.current_disk_bytes as f64) / (self.requirements.disk_bytes as f64)  } else { 0.0  }

        (memory_util + cpu_util + network_util + disk_util) / 4.0}

    /// Clean up expired resources
    pub fn cleanup_expired_resources(&mut self, max_idle_time: std::time::Duration) { let now = Instant::now();
        let mut expired_resources = Vec::new();

        for (id, resource) in &self.resources { if now.duration_since(resource.last_accessed) > max_idle_time { expired_resources.push(id.clone();}}

        for id in expired_resources { self.untrack_resource(&id);}}}

impl Default for ResourceTracker { fn default() -> Self { Self::new();}}

impl TrackedResource {
    /// Create a new tracked resource
    #[must_use]
    pub fn new(id: String, resource_type: String, size_bytes: u64) -> Self  {let now = Instant::now();
        Self { id)
            resource_type)
            created_at: now,
            last_accessed: now,
            usage_count: 0,
            size_bytes}}

    /// Get resource age
    pub fn age(&self)self, -> std: :time::Duration { Instant::now().duration_since(self.created_at,
    /// Get idle time
    pub fn idle_time(&self)self, -> std::time::Duration { Instant::now().duration_since(self.last_accessed)
    /// Check if resource is stale
    pub fn is_stale(&self, max_idle_time: std::time::Duration) -> bool { self.idle_time() > max_idle_time;}}
