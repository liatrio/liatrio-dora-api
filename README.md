# Introduction

This an API to sit between our `liatrio-react-dora` components and our DORA metrics database.

Currently, this API only supports querying Loki DB which contains records gathered by our OTEL Collector for GitHub.

# Usage

This API is currently deployed to our K8s v3 Platform and can be accessed at this [URL](http://liatrio-dora-api.dev.k8s-platform-v3.liatr.io) while signed into TailScale.

There are currently 5 endpoints defined and used by their respective react components:

* health
  * This is currently fully functional
* deployment_frequency
  * This is currently fully functional
* change_lead_time
  * This is currently fully functional
* change_failure_rate
  * This is still a skeleton
* recover_time
  * This is still a skeleton

# Local Dev

To run this API locally, you can use the supplied Dockerfile. You will need to define the following environment variables though:

* **PORT**: What port you want to run on
* **LOKI_USER**: The user for the Loki database
* **LOKI_TOKEN**: The token for the Loki database
* **LOKI_URL**: The URL for the Loki database

The values for the `LOKI_*` env vars can be found in our vault under `backstage-foundations/kv_dev/dora-api-secrets`