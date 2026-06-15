use anyhow::{Context, Result};
use k8s_openapi::api::batch::v1::Job;
use kube::{Api, Client};

/// Lightweight wrapper around a Kubernetes client for Job management.
pub struct K8sClient {
    client: Client,
}

impl K8sClient {
    /// Create a new K8s client using the in-cluster config
    /// (falls back to kubeconfig when running locally).
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .context("failed to create Kubernetes client")?;
        Ok(K8sClient { client })
    }

    /// Create a Kubernetes Job from a parsed Job object.
    ///
    /// Returns the Job name on success.
    pub async fn create_job(&self, namespace: &str, job: &Job) -> Result<String> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), namespace);
        let job = api
            .create(&Default::default(), job)
            .await
            .context("failed to create Kubernetes Job")?;
        let name = job.metadata.name.clone().unwrap_or_default();
        tracing::info!("Created Job {name} in namespace {namespace}");
        Ok(name)
    }

    /// Get the status of a Job by name.
    pub async fn get_job_status(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<JobStatusResponse> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), namespace);
        let job = api
            .get(name)
            .await
            .context(format!("failed to get Job {name}"))?;

        let status = job.status.unwrap_or_default();
        let phase = if status.succeeded.unwrap_or(0) > 0 {
            "Completed".to_string()
        } else if status.failed.unwrap_or(0) > 0 && status.active.unwrap_or(0) == 0 {
            "Failed".to_string()
        } else {
            "Running".to_string()
        };

        Ok(JobStatusResponse {
            name: name.to_string(),
            active: status.active.unwrap_or(0),
            succeeded: status.succeeded.unwrap_or(0),
            failed: status.failed.unwrap_or(0),
            phase,
        })
    }

    /// Check whether the cluster is reachable and the client works.
    pub async fn check_health(&self) -> Result<()> {
        // Simple API discovery check
        self.client
            .apiserver_version()
            .await
            .context("Kubernetes API server not reachable")?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobStatusResponse {
    pub name: String,
    pub active: i32,
    pub succeeded: i32,
    pub failed: i32,
    pub phase: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_response_serde() {
        let resp = JobStatusResponse {
            name: "install-headscale-1234".into(),
            active: 0,
            succeeded: 1,
            failed: 0,
            phase: "Completed".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("install-headscale-1234"));
        assert!(json.contains("Completed"));

        let deserialized: JobStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "install-headscale-1234");
        assert_eq!(deserialized.phase, "Completed");
    }
}
