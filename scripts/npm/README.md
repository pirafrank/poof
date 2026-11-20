# npm packaging and publishing tools

These scripts prepare npm distributions from an already-published,
signed GitHub release. They do not build Rust binaries or modify the release's
assets. They are used in the GitHub Actions workflow defined in
[`.github/workflows/npm.yml`](../../.github/workflows/npm.yml).

## Pipeline scripts

- **`verify_release.py`** establishes that the downloaded native binaries belong
  to the requested signed release. It checks release identity, checksums, signatures,
  and attestations, then writes evidence for the packaging steps.
- **`stage_packages.py`** validates those binaries and packages them into npm
  tarballs. It checks archive contents and binary format/architecture before
  creating the packages; it does not publish them.
- **`verify_registry.py`** compares the staged package bytes with the npm
  registry. Before publishing, it decides whether each package/version is safe
  to publish or should be skipped; after publishing, it can require and verify
  that all packages are present and match the staged bytes.
- **`release_manifest.py`** creates an auditable record connecting the verified
  release inputs, native artifacts, and npm packages. It is available for manual
  use; the current workflow does not generate or upload a publication manifest.
- **`common.py`** holds shared constants, types, and logic used by these scripts,
  including package metadata, version handling, and target mappings.

## Tests

Run `./scripts/npm/run_tests.sh` from the repository root to run both test suites.
The GitHub Actions workflow `.github/workflows/ci_npm.yml` runs them independently
in separate jobs, and only for changes to npm-related files. To run either suite
on its own, use `scripts/npm/run_python_tests.sh` for packaging tests or
`scripts/npm/run_launcher_tests.sh` for launcher tests. The packaging tests require
Python 3.11 or later; launcher tests require Node.js 18 or later.

## Stage and approve

Merge infrastructure changes to `main` before testing them. Manually run
`npm.yml` (Release to npm) with the full signed release tag, including the `v`
prefix (for example, `v0.6.4-rc.3`). The workflow resolves one
`main` commit for all packaging scripts, and uses the verified release commit for
`npm/`, `matrix.jsonc`, and `LICENSE.md`. Rust binaries come from the signed native
release, without rebuilding. Packing uses Node 24; submission uses Node 24.22.0
or later. Both use npm 11.21.3. A Git tag alone is insufficient: the workflow
requires a published, immutable GitHub Release with the expected assets,
checksums, signatures, and attestations, plus compatible npm sources at the tag.

Enable `npm stage publish` for all five package trusted publishers, with the
`npm.yml` workflow and existing `publish-npm` environment. No npm token is needed.
The workflow runs `npm stage publish` directly in Bash and never approves packages.
Submission outcomes and npm output are available in the Actions job logs.
Matching public versions are skipped and mismatching public versions stop the run.
Pending stage conflicts also fail: inspect and approve or reject them on npmjs
before retrying. Stable releases use the `latest` npm dist-tag; prereleases use
`next`. The workflow rejects publication that would regress `latest`.

Set the optional `dry_run` input to `true` to run `npm stage publish --dry-run`
instead of submitting packages. It defaults to `false`. Dry runs still perform
release verification, packaging, and registry reconciliation, including skipping
matching published versions and rejecting mismatches. They still require a signed
GitHub Release; they do not build binaries from a bare tag.

For an acceptance test, stage a signed prerelease and inspect all five submissions
on npmjs. Approve the four platform packages before the launcher. Approval is not
atomic: check every package's result. To discard a test instead, reject its stages.

After approval, check publication of all five package versions on npmjs yourself.
There is no separate post-approval verification workflow.

The workflow retains three artifacts for 30 days: `npm-verification` (downloaded
assets, release evidence, and source/infrastructure revisions), `npm-staging`
(tarballs and packaging report), and `npm-reconciliation` (registry observations
and publication decisions). It does not change native GitHub release assets.
