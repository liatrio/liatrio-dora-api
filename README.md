# Introduction

This an API to sit between our `liatrio-react-dora` components and your DORA metrics database.

# Metric Storage Compatibility

For our initial purposes, we chose to use Loki DB for the storage of our observability events.  Due to that, this API is loosely tied to Loki DB.

If you desire to use a time series database other than Loki DB, it should only be a small lift to fork this repository and implement that change.

# Installation

## Docker

If are using Loki DB and want to use our public pre-built Docker Image, you can find it [here](https://github.com/liatrio/liatrio-dora-api/pkgs/container/liatrio-dora-api)

## Building from Source

You will need the following:

* [Rust Compiler](https://www.rust-lang.org/tools/install)

When running the API locally, it does support the use of a `.env` file at the root of the project for providing [Environment Variables](https://github.com/liatrio/liatrio-dora-api/tree/main?tab=readme-ov-file#environment-variables).

If you are unfamiliar with Rust, you can build the application using `cargo build` and run the application using `cargo run`.

# Usage

* health
  * Used for standard health checks

* data
  * This returns all the data necessary for a given start/end time frame and set or repositories and or team.
  * Method: `POST`
  * The expected body is a JSON blob containing the following:
      * `start`: The UTC time to begin querying for metrics
      * `end`: The UTC time to end querying for metrics
      * `repositories` (Optional): An array of repository names that you want to query the metrics of
      * `team` (Optional):  A specific team name you want to query metrics for
  * The response will be a JSON blob containing the following:
    * `records`: An array of deployment records
      * `repository`: The repository this record belongs to
      * `team`: The team that owns the repository
      * `title`: The commit message of the change
      * `user`: The user that commited the change
      * `sha`: The commit sha of the change
      * `status`: The deployment status
      * `failed_at` (Optional): When the deployment failed, if it did
      * `merged_at`: When the change was merged to `main`
      * `created_at`: When the deployment started
      * `fixed_at`: When an issue with a failed deployment was resolved
      * `fixed_url`: A link to the deployment that resolved the failure
      * `deploy_url`: A link to the current deployment
      * `issue_url`: A link to the issue that was created to track the failed deployment
      * `change_url`: A link to the change that caused the deployment

* teams
  * This will return a list of teams and their associated repositories from the GitHub Org you have supplied as an env var.
  * Method: `GET`
  * Parameters: None
  * The response will be a JSON blob containing the following:
    * `teams`: An array of team records
      * `name`: The name of the team
      * `repositories`: A list of the repositories this team owns

# Environment Variables

The following variables are required to run this API:

* **PORT**: What port you want to run on
* **GITHUB_ORG**: The GitHub Org used to host your repositories
* **GITHUB_TOKEN**: A GitHub Token that has access to the following:
  * **repo** - 'Full control of private repositories'
    * Unfortintely GitHub does not provide a way of listing a teams repos without full access
  * **read:org** - 'Read org and team membership, read org projects'

If you are using Loki DB for your metrics storage, you will need the following:

* **LOKI_URL**: The URL for the Loki database

Optionally, if you have secured the REST API on your Loki DB, you will need to provide the following:

* **LOKI_USER**: The user for the Loki database
* **LOKI_TOKEN**: The token for the Loki database
