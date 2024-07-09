# Introduction

This an API to sit between our `liatrio-react-dora` components and our DORA metrics database.

Currently, this API only supports querying Loki DB which contains records gathered by our OTEL Collector for GitHub.

# Usage

This API is currently deployed to our K8s v3 Platform and can be accessed at this [URL](http://liatrio-dora-api.dev.k8s-platform-v3.liatr.io) while signed into TailScale.

There are currently 2 endpoints defined:

* health
  * Use for standard health checks
* data
  * This returns all the data necessary for a given start/end time frame and set or repositories and or team.

# Local Dev

To run this API locally, you can use the supplied Dockerfile. You will need to define the following environment variables though:

* **PORT**: What port you want to run on
* **LOKI_USER**: The user for the Loki database
* **LOKI_TOKEN**: The token for the Loki database
* **LOKI_URL**: The URL for the Loki database

The values for the `LOKI_*` env vars can be found in our vault under `backstage-foundations/kv_dev/dora-api-secrets`
