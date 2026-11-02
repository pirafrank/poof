# Security policy

Security fixes target the latest stable patch of the current release series
(currently 0.6.x). Older patches and earlier series do not receive backports;
upgrade to the latest stable release. Development builds are not supported releases.

## Report a vulnerability privately

Use [GitHub private vulnerability reporting](https://github.com/pirafrank/poof/security/advisories/new).
Include the affected version, OS and architecture, reproduction steps, impact,
and a minimal sample archive or release metadata when relevant. Do not include
tokens, credentials, or other private data. Please keep exploit details out of
public issues until a fix and disclosure have been coordinated.

## Release authenticity

Starting with the next release built by the updated release workflow, each target
has an SPDX JSON SBOM and GitHub Artifact Attestations for its binary and archive.
The SBOM inventories the build's locked source dependencies; it may include
dependencies that are not linked on that target. Older releases may lack these files.
Checksums detect corruption but do not establish who produced an asset.

Download the desired archive from the official release, then verify it before
extracting or running it. Substitute the actual asset filename:

```sh
gh attestation verify ./poof-VERSION-TARGET.tar.gz \
  --repo pirafrank/poof \
  --signer-workflow pirafrank/poof/.github/workflows/release.yml \
  --source-ref refs/tags/vVERSION

gh attestation verify ./poof-VERSION-TARGET.tar.gz \
  --repo pirafrank/poof \
  --signer-workflow pirafrank/poof/.github/workflows/release.yml \
  --source-ref refs/tags/vVERSION \
  --predicate-type https://spdx.dev/Document/v2.3
```

After extraction, the same commands can verify `./poof`. Verification needs an
authenticated GitHub CLI and network access. See
[GitHub's verification documentation](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).

## Release checks

Before merging release preparation or creating a tag, require passing Linux and
macOS CI, MSRV checks, and the security audit. Run the Release workflow manually
on the candidate commit to test the complete target matrix with `--locked`, SBOM
generation, signing, and verification. Its manual run uploads candidate artifacts
without creating a release. Review that exact commit and workflow run before
tagging; a tag push repeats these gates before publication. Configure repository
branch protection/rulesets to require the CI and MSRV status checks before merging.

Archive extraction uses fresh staging directories and enforces a combined 2 GiB
expanded payload limit and 100,000 member limit. Links, special files, unsafe
paths, and overwrites of existing extracted files are rejected. Failed extraction
removes its staging directory; abrupt process termination can leave temporary data
that can be removed with `poof clean` when it is in the download cache.
