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

Download the archive, checksum manifest, and its Sigstore bundle from the same
[GitHub release](https://github.com/pirafrank/poof/releases). Verify the manifest
signature and archive checksum before extracting or running an asset. Replace the
example version and filename with the release you downloaded:

```sh
cosign verify-blob poof-VERSION-checksums.txt \
  --bundle poof-VERSION-checksums.txt.sigstore.json \
  --certificate-identity 'https://github.com/pirafrank/poof/.github/workflows/release.yml@refs/tags/vVERSION' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com'
sha256sum --check poof-VERSION-checksums.txt
```

Run the checksum command in the directory containing every manifest entry. On
macOS, use `shasum -a 256 --check`. If verifying one asset in a wrapper, select
its exact filename from the manifest and require exactly one matching entry.

GitHub attestations verify the archive and its SPDX SBOM against the release tag:

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

Attestations can also verify `./poof` after extraction. Verification requires
network access and an authenticated GitHub CLI. See [GitHub's verification
documentation](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).

Each archive also has a build-environment JSON record covered by the signed
checksum manifest. It records artifact identity, source and workflow/run details,
runner and Rust toolchain information, build inputs and configuration, plus host
and cross-compilation toolchains. Unavailable probes are null; sensitive
environment values and credentials are not collected. This record aids build
investigation but does not prove reproducibility; independent rebuilds and output
comparison are needed for that.

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
