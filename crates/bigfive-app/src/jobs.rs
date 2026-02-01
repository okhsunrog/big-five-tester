//! Background job management for async AI analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

/// Unique job identifier
pub type JobId = String;

/// Status of a background analysis job
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued, waiting to start
    Pending,
    /// Job is currently processing
    Processing,
    /// Job completed successfully with result
    Complete(String),
    /// Job failed with error message
    Error(String),
}

/// Internal job data with metadata
struct JobEntry {
    status: JobStatus,
    created_at: Instant,
}

/// In-memory job store
struct JobStore {
    jobs: HashMap<JobId, JobEntry>,
}

impl JobStore {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    /// Clean up old jobs (older than 1 hour)
    fn cleanup_old_jobs(&mut self) {
        let max_age = Duration::from_secs(3600); // 1 hour
        self.jobs
            .retain(|_, entry| entry.created_at.elapsed() < max_age);
    }
}

/// Global job store instance
static JOB_STORE: LazyLock<Mutex<JobStore>> = LazyLock::new(|| Mutex::new(JobStore::new()));

/// Generate a new unique job ID
pub fn generate_job_id() -> JobId {
    uuid::Uuid::new_v4().to_string()
}

/// Create a new job with Pending status
pub fn create_job(job_id: &JobId) {
    let mut store = JOB_STORE.lock().unwrap();
    store.cleanup_old_jobs();
    store.jobs.insert(
        job_id.clone(),
        JobEntry {
            status: JobStatus::Pending,
            created_at: Instant::now(),
        },
    );
}

/// Update job status
pub fn update_job_status(job_id: &JobId, status: JobStatus) {
    let mut store = JOB_STORE.lock().unwrap();
    if let Some(entry) = store.jobs.get_mut(job_id) {
        entry.status = status;
    }
}

/// Get job status
pub fn get_job_status(job_id: &JobId) -> Option<JobStatus> {
    let store = JOB_STORE.lock().unwrap();
    store.jobs.get(job_id).map(|entry| entry.status.clone())
}

/// Remove a completed job (optional cleanup)
pub fn remove_job(job_id: &JobId) {
    let mut store = JOB_STORE.lock().unwrap();
    store.jobs.remove(job_id);
}
