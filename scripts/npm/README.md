# npm packaging and publishing tools

These scripts package and publish npm distributions from an already-published,
signed GitHub release. They do not build Rust binaries or modify the release's
assets. They are used in the GitHub Actions workflow defined in
[`.github/workflows/npm.yml`](../../.github/workflows/npm.yml).

## Pipeline scripts

- **`verify_release.py`** establishes that the downloaded native binaries belong
'to the requested signed release. It checks release identity, checksums, signatures,
  and attestations, then writes evidence for the packaging steps.
- **`stage_packages.py`** validates those binaries and packages them into npm
  tarballs. It checks archive contents and binary format/architecture before
  creating the packages; it does not publish them.
- **`verify_registry.py`** compares the staged package bytes with the npm
  registry. Before publishing, it decides whether each package/version is safe
  to publish or should be skipped; after publishing, it can require and verify
  that all packages are present and match the staged bytes.
- **`release_manifest.py`** creates an auditable record connecting the verified
  release inputs, native artifacts, and npm packages. The workflow uploads this
  manifest as an artifact rather than adding it to the GitHub release.
- **`common.py`** holds shared constants, types, and logic used by these scripts,
  including package metadata, version handling, and target mappings.

## Tests

Run `./scripts/npm/run_tests.sh` from the repository root to run the Python
packaging tests and Node launcher tests. The scripts require Python 3.11 or later;
launcher tests require Node.js 18 or later, albeit Node.js 24 is recommended.
