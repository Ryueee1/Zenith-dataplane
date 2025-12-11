use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub node_count: usize,
    pub plugin_count: usize,
    pub deployment_count: usize,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataNode {
    pub id: String,
    pub address: String,
    pub capacity: u64,
    pub status: NodeStatus,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Active,
    Inactive,
    Maintenance,
}

#[derive(Debug, Deserialize)]
pub struct RegisterNodeRequest {
    pub address: String,
    pub capacity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub wasm_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPluginRequest {
    pub name: String,
    pub version: String,
    pub wasm_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub plugin_id: String,
    pub node_ids: Vec<String>,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Active,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub plugin_id: String,
    pub node_ids: Vec<String>,
}
