# CHANGELOG


## v1.2.4 (2024-11-20)

### Bug Fixes

- **instrumentation**: Set full url for exporter otlp endpoint
  ([`64a8e97`](https://github.com/liatrio/liatrio-dora-api/commit/64a8e979297b530d88098fc916f36de032840532))


## v1.2.3 (2024-10-02)

### Bug Fixes

- Correct this conditional ([#48](https://github.com/liatrio/liatrio-dora-api/pull/48),
  [`d485941`](https://github.com/liatrio/liatrio-dora-api/commit/d48594148a7c85c63db74cc683a5d277098c1868))

* fix: correct this conditional

* chore: remove print statement


## v1.2.2 (2024-10-02)

### Bug Fixes

- Allow env to be prod- ([#46](https://github.com/liatrio/liatrio-dora-api/pull/46),
  [`35729a3`](https://github.com/liatrio/liatrio-dora-api/commit/35729a341a092ab0c0efa2805743d6cd7fd1a7e6))

* fix: allow env to be prod-

* fix: run cargo fmt


## v1.2.1 (2024-10-02)

### Bug Fixes

- Allow tag-o11y ([#47](https://github.com/liatrio/liatrio-dora-api/pull/47),
  [`9ef2685`](https://github.com/liatrio/liatrio-dora-api/commit/9ef2685df20063c49270958bf2383af0f1544650))


## v1.2.0 (2024-10-02)

### Features

- Enable OTLP trace exportation in manifests
  ([#45](https://github.com/liatrio/liatrio-dora-api/pull/45),
  [`abe53b8`](https://github.com/liatrio/liatrio-dora-api/commit/abe53b83bf40082628534406d8b966b39b276072))


## v1.1.16 (2024-09-20)

### Bug Fixes

- Correct deployment query ([#44](https://github.com/liatrio/liatrio-dora-api/pull/44),
  [`623f867`](https://github.com/liatrio/liatrio-dora-api/commit/623f867c4e9873e14d15f7ddc7e7e3fe52cbf811))

### Chores

- Update workflow names, update readme with badges
  ([#43](https://github.com/liatrio/liatrio-dora-api/pull/43),
  [`7036246`](https://github.com/liatrio/liatrio-dora-api/commit/70362460750a58688fac2ff0e58fc114bfe56f40))


## v1.1.15 (2024-09-18)

### Bug Fixes

- Ver bump ([#42](https://github.com/liatrio/liatrio-dora-api/pull/42),
  [`081905e`](https://github.com/liatrio/liatrio-dora-api/commit/081905e5841499c49ecb7c14e8e727c4ab1b3ff3))

### Chores

- Query updates, documentation, tests ([#41](https://github.com/liatrio/liatrio-dora-api/pull/41),
  [`a359cc9`](https://github.com/liatrio/liatrio-dora-api/commit/a359cc973d601e8f755c8c3683cd157651dab223))

* chore: update test data

* chore: move vendor specific functionality, allow cache bypass

* chore: formatting, clippy, disable debug code

* fix: include subsequent success if first was a failure

* chore: add documentation, unit tests and test job for PRs

* chore: fmt and clippy

* chore: more clippy

### Documentation

- Add template for PRs ([#40](https://github.com/liatrio/liatrio-dora-api/pull/40),
  [`44d132f`](https://github.com/liatrio/liatrio-dora-api/commit/44d132fdd309601861c45986950e89f6a997576a))

- Add contributing docs ([#39](https://github.com/liatrio/liatrio-dora-api/pull/39),
  [`1cf0ce1`](https://github.com/liatrio/liatrio-dora-api/commit/1cf0ce157baf6afd1ec7dd40dc577e342bbe7d75))

* docs: add contributing docs

* fix: remove pre-commit config, add conventional commit blurb in contributing doc


## v1.1.14 (2024-09-17)

### Bug Fixes

- Added LICENSE, build, and language badge to repo
  ([#38](https://github.com/liatrio/liatrio-dora-api/pull/38),
  [`07b3dd4`](https://github.com/liatrio/liatrio-dora-api/commit/07b3dd4700df7c9c86a52234748d1156fce441d5))

* fix: added LICENSE, build, and language badge to repo

* docs: reformatted README

### Chores

- Add format check ([#37](https://github.com/liatrio/liatrio-dora-api/pull/37),
  [`94939ea`](https://github.com/liatrio/liatrio-dora-api/commit/94939ea488079fd1de9e34e95db6adeaab52748a))

* chore: add format check

* chore: formatting

- Add Clippy ([#36](https://github.com/liatrio/liatrio-dora-api/pull/36),
  [`933f0d6`](https://github.com/liatrio/liatrio-dora-api/commit/933f0d63d5d1863fac7a43637ce181cfb167530e))

* fix: expand limit to loki query, use different field for merge sha, cleanup warnings

* chore: Add Clippy Job and update code to follow clippy

* fix: remove duplicate code line causing errors

* chore: run fmt

* chore: remove dead code

- Add issue templates ([#35](https://github.com/liatrio/liatrio-dora-api/pull/35),
  [`e31fab1`](https://github.com/liatrio/liatrio-dora-api/commit/e31fab14e3fb5344d10c28635a1447f2cd54c83d))

### Continuous Integration

- Add code scanning (codeql) and badge ([#34](https://github.com/liatrio/liatrio-dora-api/pull/34),
  [`5f57fc1`](https://github.com/liatrio/liatrio-dora-api/commit/5f57fc12de0368ce70de4579f34abf6ec74e000f))

* ci: add code scanning with codeql and badge

* docs: fix spacing in readme

* fix: precommit fix

### Documentation

- Readme updates & pre-commit config ([#33](https://github.com/liatrio/liatrio-dora-api/pull/33),
  [`9c9ada4`](https://github.com/liatrio/liatrio-dora-api/commit/9c9ada43a46e1b6ee667e4169d3f4e5f135cd595))

* feat: add pre-commit config

* fix: style/formatting/pre-commit fixes

* fix: clean up section headings

* fix: convert lists to tables

* fix: spelling fixes

* fix: update link to internal header


## v1.1.13 (2024-08-17)

### Bug Fixes

- Expand limit to loki query, use different field for merge sha, cleanup warnings
  ([#32](https://github.com/liatrio/liatrio-dora-api/pull/32),
  [`b86b124`](https://github.com/liatrio/liatrio-dora-api/commit/b86b12434af3682f1d78cdfc90a9116ca5410176))


## v1.1.12 (2024-08-16)

### Bug Fixes

- Allow a 3rd option for workflow link ([#31](https://github.com/liatrio/liatrio-dora-api/pull/31),
  [`17d50b6`](https://github.com/liatrio/liatrio-dora-api/commit/17d50b68ce6b7573eb07f6b63d304df28bd3cce9))

Co-authored-by: Wolftousen <eliot.t.eikenberry@perilforge.com>


## v1.1.11 (2024-08-16)

### Bug Fixes

- Relied on vscode to much to catch this
  ([#30](https://github.com/liatrio/liatrio-dora-api/pull/30),
  [`580f6d7`](https://github.com/liatrio/liatrio-dora-api/commit/580f6d72c191e7507f373c4915afc9e1c9a6a0af))

Co-authored-by: Wolftousen <eliot.t.eikenberry@perilforge.com>


## v1.1.10 (2024-08-16)

### Bug Fixes

- Allow either workflow_id or url to be used
  ([#29](https://github.com/liatrio/liatrio-dora-api/pull/29),
  [`5754ef9`](https://github.com/liatrio/liatrio-dora-api/commit/5754ef94d6d55656f83bd46f85a639fbdf3a7665))

Co-authored-by: Wolftousen <eliot.t.eikenberry@perilforge.com>


## v1.1.9 (2024-08-16)

### Bug Fixes

- Revert local testing changes ([#28](https://github.com/liatrio/liatrio-dora-api/pull/28),
  [`73d2901`](https://github.com/liatrio/liatrio-dora-api/commit/73d2901022079848e7f24106defe4eaf0bd37f2d))


## v1.1.8 (2024-08-16)

### Bug Fixes

- Build workflow run url ([#27](https://github.com/liatrio/liatrio-dora-api/pull/27),
  [`99ff807`](https://github.com/liatrio/liatrio-dora-api/commit/99ff80789dfc539e3caa1a14f5cca340e3c31280))


## v1.1.7 (2024-08-16)

### Bug Fixes

- We don't need repository ownership ([#26](https://github.com/liatrio/liatrio-dora-api/pull/26),
  [`4c93fb6`](https://github.com/liatrio/liatrio-dora-api/commit/4c93fb67e7737bb302f6ef1fac49ac5b2aab4692))


## v1.1.6 (2024-08-14)

### Bug Fixes

- Add new env var to manifest ([#25](https://github.com/liatrio/liatrio-dora-api/pull/25),
  [`2b042bc`](https://github.com/liatrio/liatrio-dora-api/commit/2b042bc484f8e059c98ab7e266ec1d90ac1f34bd))


## v1.1.5 (2024-08-14)

### Bug Fixes

- Refactor query code to be less Loki specific
  ([#24](https://github.com/liatrio/liatrio-dora-api/pull/24),
  [`70675b4`](https://github.com/liatrio/liatrio-dora-api/commit/70675b44a95f3d0c21dafc36871dfa293c19aad3))

* fix: refactor the code to start splitting loki specific logic

* fix: make small batch size configurable

### Chores

- Update Readme ([#23](https://github.com/liatrio/liatrio-dora-api/pull/23),
  [`6029d54`](https://github.com/liatrio/liatrio-dora-api/commit/6029d54f1727f7cc02697f27759a3d634d6153d8))

Co-authored-by: Wolftousen <eliot.t.eikenberry@perilforge.com>


## v1.1.4 (2024-08-08)

### Bug Fixes

- Check for workflow_run not existing ([#21](https://github.com/liatrio/liatrio-dora-api/pull/21),
  [`b82afb2`](https://github.com/liatrio/liatrio-dora-api/commit/b82afb23ee086fc4f788a41dc7e5a58812e528fc))


## v1.1.3 (2024-08-08)

### Bug Fixes

- Provide issue url ([#20](https://github.com/liatrio/liatrio-dora-api/pull/20),
  [`4c3eb6e`](https://github.com/liatrio/liatrio-dora-api/commit/4c3eb6e37be0580dcabbbe6e8d0c8c523e48cfaf))

* fix: add urls to response

* fix: adjust commit url

* fix: provide issue url


## v1.1.2 (2024-08-08)

### Bug Fixes

- Add urls to response ([#19](https://github.com/liatrio/liatrio-dora-api/pull/19),
  [`8189101`](https://github.com/liatrio/liatrio-dora-api/commit/81891013961e0c73cdf4aba560886d283ef61f3f))

* fix: add urls to response

* fix: adjust commit url


## v1.1.1 (2024-08-01)

### Bug Fixes

- Force ver bump ([#18](https://github.com/liatrio/liatrio-dora-api/pull/18),
  [`bcdee75`](https://github.com/liatrio/liatrio-dora-api/commit/bcdee75157672ed7d8090b9f8668bc548be9b38b))


## v1.1.0 (2024-08-01)

### Features

- Add endpoint to retrieve a list of teams and the repositories they own
  ([#16](https://github.com/liatrio/liatrio-dora-api/pull/16),
  [`de5ca62`](https://github.com/liatrio/liatrio-dora-api/commit/de5ca62c0a24ad206b07ef29d03327c8bc88b0be))


## v1.0.0 (2024-07-29)

### Build System

- 1.0.0
  ([`4dac392`](https://github.com/liatrio/liatrio-dora-api/commit/4dac3926d90eeea76d1efbb7564319cf17c63e9c))
