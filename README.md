# Introduction

[![CodeQL](https://github.com/liatrio/liatrio-dora-api/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/liatrio/liatrio-dora-api/actions/workflows/codeql.yml)

This an API to sit between our `liatrio-react-dora` components and your DORA metrics database.

## Metric Storage Compatibility

For our initial purposes, we chose to use Loki DB for the storage of our observability events.  Due to that, this API is loosely tied to Loki DB.

If you desire to use a time series database other than Loki DB, it should only be a small lift to fork this repository and implement that change.

## Installation

### Docker

If are using Loki DB and want to use our public pre-built Docker Image, you can find it [here](https://github.com/liatrio/liatrio-dora-api/pkgs/container/liatrio-dora-api)

### Building from Source

You will need the following:

* [Rust Compiler](https://www.rust-lang.org/tools/install)

When running the API locally, it does support the use of a `.env` file at the root of the project for providing [Environment Variables](#environment-variables)

If you are unfamiliar with Rust, you can build the application using `cargo build` and run the application using `cargo run`.

## Routes

The API supplies the following routes:

### `/health`

Method: `GET`

Used for standard health checks

### `/data`

Method: `POST`

This returns all the data necessary for a given start/end time frame and set or repositories and or team. The request body should be a JSON blob containing the following:

| Key            | Description                                                        | Required |
|----------------|--------------------------------------------------------------------|----------|
| `start`        | The UTC time to begin querying for metrics                         | true     |
| `end`          | The UTC time to end querying for metrics                           | true     |
| `repositories` | An array of repository names that you want to query the metrics of | false    |
| `team`         | A specific team name you want to query metrics for                 | false    |

The response will be a JSON blob containing with a `records` key containing an array of deployment records. Each record contains the following:

| Key          | Description                                                         |
|--------------|---------------------------------------------------------------------|
| `repository` | The repository this record belongs to                               |
| `team`       | The team that owns the repository                                   |
| `title`      | The commit message of the change                                    |
| `user`       | The user that committed the change                                  |
| `sha`        | The commit sha of the change                                        |
| `status`     | The deployment status                                               |
| `failed_at`  | When the deployment failed, if it did                               |
| `merged_at`  | When the change was merged to `main`                                |
| `created_at` | When the deployment started                                         |
| `fixed_at`   | When an issue with a failed deployment was resolved                 |
| `fixed_url`  | A link to the deployment that resolved the failure                  |
| `deploy_url` | A link to the current deployment                                    |
| `issue_url`  | A link to the issue that was created to track the failed deployment |
| `change_url` | A link to the change that caused the deployment                     |

### `/teams`

Method: `GET`

This will return a list of teams and their associated repositories from the GitHub organization specified in `GITHUB_ORG`.

The response will be a JSON blob with a `teams` key containing an array of team records. Each record contains the following:

| Key            | Description                               |
|----------------|-------------------------------------------|
| `name`         | The name of the team                      |
| `repositories` | A list of the repositories this team owns |

## Environment Variables

The following variables are required to run this API:

| Variable       | Description                                       |
|----------------|---------------------------------------------------|
| `PORT`         | What port you want to run on                      |
| `GITHUB_ORG`   | The GitHub Org used to host your repositories     |
| `GITHUB_TOKEN` | A GitHub Token with access to the Org (see below) |

The `GITHUB_TOKEN` must have the following scopes:

| Scope      | Description                                     |
|------------|-------------------------------------------------|
| `repo`     | Full control of private repositories            |
| `read:org` | Read org and team membership, read org projects |

Full control is required because GitHub does not provide a way of listing a teams repos without it.

If you are using Loki DB for your metrics storage, you will also need to provide the following environment variables:

| Variable     | Description                                                            |
|--------------|------------------------------------------------------------------------|
| `LOKI_URL`   | The URL for the Loki database                                          |
| `LOKI_USER`  | The user for the Loki database. _Required if your Loki DB is secured_  |
| `LOKI_TOKEN` | The token for the Loki database. _Required if your Loki DB is secured_ |
