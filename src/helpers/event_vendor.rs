use super::loki::ValueItem;

pub trait EventVendorFunctions {
  fn extract_change_url(entry: &ValueItem) -> String;
  fn extract_deployment_url(entry: &ValueItem) -> String;
}