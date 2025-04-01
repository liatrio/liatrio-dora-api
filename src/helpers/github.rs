use super::{event_vendor::EventVendorFunctions, loki::ValueItem};

pub struct GitHub {}

impl EventVendorFunctions for GitHub {
    /// Extracts a commit URL from a deployment entry by transforming the deployment URL.
    ///
    /// This function takes a `ValueItem` that contains a deployment object with a URL,
    /// deployment ID, and SHA. It modifies the deployment URL by:
    ///
    /// 1. Removing the "api." portion of the URL.
    /// 2. Replacing "repos/" with an empty string.
    /// 3. Replacing "deployments/" with "commit/".
    /// 4. Replacing the deployment ID with the corresponding SHA value.
    ///
    /// # Arguments
    ///
    /// * `entry` - A reference to a `ValueItem` containing the deployment information.
    ///
    /// # Returns
    ///
    /// A `String` representing the transformed commit URL.
    ///
    /// # Panics
    ///
    /// This function will panic if the `deployment` field inside the `json_data` of the `ValueItem`
    /// is `None` or if any of the fields inside the deployment (such as `url`, `id`, or `sha`) are missing.
    ///
    /// # Example
    ///
    /// ```
    /// let entry = ValueItem::new(Some(Deployment {
    ///     url: "https://api.github.com/repos/owner/repo/deployments/123456".to_string(),
    ///     id: 123456,
    ///     sha: "abcdef".to_string(),
    /// }));
    ///
    /// let result = extract_change_url(&entry);
    /// assert_eq!(result, "https://github.com/owner/repo/commit/abcdef");
    /// ```
    fn extract_change_url(entry: &ValueItem) -> String {
        let deployment = entry.json_data.deployment.as_ref().unwrap();

        deployment
            .url
            .replace("api.", "")
            .replace("repos/", "")
            .replace("deployments/", "commit/")
            .replace(deployment.id.to_string().as_str(), &deployment.sha)
    }

    /// Extracts a workflow run URL from a deployment entry, if a workflow is present.
    ///
    /// This function takes a `ValueItem` that contains deployment and workflow run information.
    /// If a workflow run with a `workflow_id` is present, it modifies the deployment URL by:
    ///
    /// 1. Removing the "api." portion of the URL.
    /// 2. Replacing "repos/" with an empty string.
    /// 3. Replacing "deployments/" with "actions/runs/".
    /// 4. Replacing the deployment ID with the corresponding workflow ID.
    ///
    /// If no workflow is found or the `workflow_id` is `None`, an empty string is returned.
    ///
    /// # Arguments
    ///
    /// * `entry` - A reference to a `ValueItem` containing the deployment and workflow information.
    ///
    /// # Returns
    ///
    /// A `String` representing the transformed workflow run URL, or an empty string if no workflow run with a `workflow_id` is found.
    ///
    /// # Panics
    ///
    /// This function will panic if the `deployment` field inside the `json_data` of the `ValueItem`
    /// is `None`, or if the required fields inside the deployment (such as `url` or `id`) are missing.
    ///
    /// # Example
    ///
    /// ```
    /// let entry = ValueItem::new(
    ///     Some(Deployment {
    ///         url: "https://api.github.com/repos/owner/repo/deployments/123456".to_string(),
    ///         id: 123456,
    ///         sha: "abcdef".to_string(),
    ///     }),
    ///     Some(WorkflowRun {
    ///         workflow_id: Some(654321),
    ///     })
    /// );
    ///
    /// let result = extract_deployment_url(&entry);
    /// assert_eq!(result, "https://github.com/owner/repo/actions/runs/654321");
    /// ```
    ///
    /// If no workflow run or `workflow_id` is present, an empty string is returned:
    ///
    /// ```
    /// let entry = ValueItem::new(Some(Deployment { ... }), None);
    ///
    /// let result = extract_deployment_url(&entry);
    /// assert_eq!(result, "");
    /// ```
    fn extract_deployment_url(entry: &ValueItem) -> String {
        let deployment = entry.json_data.deployment.as_ref().unwrap();

        if let Some(wf) = entry
            .json_data
            .workflow_run
            .as_ref()
            .filter(|wf| wf.workflow_id.is_some())
        {
            return deployment
                .url
                .replace("api.", "")
                .replace("repos/", "")
                .replace("deployments/", "actions/runs/")
                .replace(
                    deployment.id.to_string().as_str(),
                    wf.workflow_id.unwrap().to_string().as_str(),
                );
        }

        String::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::{
        event_vendor::EventVendorFunctions,
        github::GitHub,
        loki::{Deployment, JsonData, ValueItem, WorkflowRun},
    };

    #[test]
    fn test_extract_change_url() {
        let entry = ValueItem {
            json_data: JsonData {
                deployment: Some(Deployment {
                    url: "https://api.github.com/repos/owner/repo/deployments/123456".to_string(),
                    id: 123456,
                    sha: "abcdef".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };

        let result = GitHub::extract_change_url(&entry);

        assert_eq!(result, "https://github.com/owner/repo/commit/abcdef");
    }

    #[test]
    fn test_extract_deployment_url_with_workflow_run() {
        let entry = ValueItem {
            json_data: JsonData {
                deployment: Some(Deployment {
                    url: "https://api.github.com/repos/owner/repo/deployments/123456".to_string(),
                    id: 123456,
                    sha: "abcdef".to_string(),
                    ..Default::default()
                }),
                workflow_run: Some(WorkflowRun {
                    workflow_id: Some(7890),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };

        let result = GitHub::extract_deployment_url(&entry);

        assert_eq!(result, "https://github.com/owner/repo/actions/runs/7890");
    }

    #[test]
    fn test_extract_deployment_url_without_workflow_run() {
        let entry = ValueItem {
            json_data: JsonData {
                deployment: Some(Deployment {
                    url: "https://api.github.com/repos/owner/repo/deployments/123456".to_string(),
                    id: 123456,
                    sha: "abcdef".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };

        let result = GitHub::extract_deployment_url(&entry);

        assert_eq!(result, "");
    }
}
