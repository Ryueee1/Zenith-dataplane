//! gRPC Service Implementation for Scheduler

use tonic::Status;
use std::sync::Arc;
use crate::scheduler::Scheduler;
use crate::node::NodeRegistry;
use crate::job::{Job, JobDescriptor, ResourceRequirements, LocalityPreferences, SchedulingPolicy};
use std::collections::HashMap;

/// Job submission request
#[derive(Debug, Clone)]
pub struct SubmitJobRequest {
    pub name: String,
    pub user_id: String,
    pub project_id: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environment: HashMap<String, String>,
    pub working_directory: String,
    pub gpu_count: u32,
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub priority: i32,
    pub gang_schedule: bool,
}

/// Job submission response
#[derive(Debug, Clone)]
pub struct SubmitJobResponse {
    pub job_id: String,
    pub status: String,
}

/// Job status request
#[derive(Debug, Clone)]
pub struct GetJobStatusRequest {
    pub job_id: String,
}

/// Job status response
#[derive(Debug, Clone)]
pub struct GetJobStatusResponse {
    pub job_id: String,
    pub state: String,
    pub message: String,
    pub allocated_nodes: Vec<String>,
}

/// Cancel job request
#[derive(Debug, Clone)]
pub struct CancelJobRequest {
    pub job_id: String,
    pub reason: String,
}

/// Cancel job response
#[derive(Debug, Clone)]
pub struct CancelJobResponse {
    pub success: bool,
    pub message: String,
}

/// Cluster status response
#[derive(Debug, Clone)]
pub struct ClusterStatusResponse {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub total_gpus: usize,
    pub available_gpus: usize,
    pub running_jobs: usize,
    pub queued_jobs: usize,
}

/// Scheduler gRPC service
pub struct SchedulerService {
    scheduler: Arc<Scheduler>,
    node_registry: Arc<NodeRegistry>,
}

impl SchedulerService {
    /// Create a new scheduler service
    pub fn new(scheduler: Arc<Scheduler>, node_registry: Arc<NodeRegistry>) -> Self {
        Self {
            scheduler,
            node_registry,
        }
    }
    
    /// Submit a job
    pub fn submit_job(&self, request: SubmitJobRequest) -> Result<SubmitJobResponse, Status> {
        let descriptor = JobDescriptor {
            name: request.name,
            user_id: request.user_id,
            project_id: request.project_id,
            command: request.command,
            arguments: request.arguments,
            environment: request.environment,
            working_directory: request.working_directory,
            resources: ResourceRequirements {
                gpu_count: request.gpu_count,
                cpu_cores: request.cpu_cores,
                cpu_memory: request.memory_mb * 1024 * 1024, // Convert MB to bytes
                ..Default::default()
            },
            locality: LocalityPreferences::default(),
            policy: SchedulingPolicy {
                priority: request.priority,
                gang_schedule: request.gang_schedule,
                ..Default::default()
            },
            labels: HashMap::new(),
            annotations: HashMap::new(),
        };
        
        let job = Job::new(descriptor);
        
        match self.scheduler.submit(job) {
            Ok(job_id) => Ok(SubmitJobResponse {
                job_id,
                status: "QUEUED".to_string(),
            }),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    
    /// Get job status
    pub fn get_job_status(&self, request: GetJobStatusRequest) -> Result<GetJobStatusResponse, Status> {
        match self.scheduler.get_job(&request.job_id) {
            Some(job) => Ok(GetJobStatusResponse {
                job_id: job.id.to_string(),
                state: format!("{:?}", job.state),
                message: job.message.clone(),
                allocated_nodes: job.allocated_nodes,
            }),
            None => Err(Status::not_found(format!("Job not found: {}", request.job_id))),
        }
    }
    
    /// Cancel a job
    pub fn cancel_job(&self, request: CancelJobRequest) -> Result<CancelJobResponse, Status> {
        match self.scheduler.cancel(&request.job_id, &request.reason) {
            Ok(()) => Ok(CancelJobResponse {
                success: true,
                message: "Job cancelled".to_string(),
            }),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    
    /// Get cluster status
    pub fn get_cluster_status(&self) -> ClusterStatusResponse {
        let summary = self.node_registry.summary();
        
        ClusterStatusResponse {
            total_nodes: summary.total_nodes,
            healthy_nodes: summary.healthy_nodes,
            total_gpus: summary.total_gpus,
            available_gpus: summary.available_gpus,
            running_jobs: summary.running_jobs,
            queued_jobs: self.scheduler.queue_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_submit_request() {
        let request = SubmitJobRequest {
            name: "test-job".to_string(),
            user_id: "user1".to_string(),
            project_id: "project1".to_string(),
            command: "python".to_string(),
            arguments: vec!["train.py".to_string()],
            environment: HashMap::new(),
            working_directory: "/app".to_string(),
            gpu_count: 4,
            cpu_cores: 8,
            memory_mb: 16384,
            priority: 50,
            gang_schedule: true,
        };
        
        assert_eq!(request.gpu_count, 4);
    }
}
