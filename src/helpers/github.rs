use super::{event_vendor::EventVendorFunctions, loki::ValueItem};

pub struct GitHub {}

impl EventVendorFunctions for GitHub {
  fn extract_change_url(entry: &ValueItem) -> String {
    let deployment = entry.json_data.deployment.as_ref().unwrap();
    
    return deployment.url
        .replace("api.", "")
        .replace("repos/", "")
        .replace("deployments/", "commit/")
        .replace(
            deployment.id.to_string().as_str(),
            &deployment.sha,
        );
  }
  
  fn extract_deployment_url(entry: &ValueItem) -> String {
    let deployment = entry.json_data.deployment.as_ref().unwrap();
    
    if let Some(wf) = entry.json_data.workflow_run.as_ref().filter(|wf| wf.workflow_id.is_some()) {
        return deployment.url
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