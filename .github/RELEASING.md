# Releasing

A release starts with a tag push and ends with you publishing the draft. Two workflows handle everything in between.

## Setup

Two repository secrets, both under Settings > Secrets and variables > Actions.

`CARGO_REGISTRY_TOKEN` is a crates.io API token with the `publish-update` scope, created at <https://crates.io/settings/tokens>. Without it the crate publish fails.

`RELEASE_TOKEN` is a fine-grained personal access token limited to this repository with `Contents: Read and write`, created at <https://github.com/settings/personal-access-tokens>. It makes the release show up as authored by you instead of by `github-actions[bot]`, which is what watchers see in their notification emails. Without it the release is still created, just under the bot's name. Fine-grained tokens expire, so this one needs renewing when it does.

## Steps

1. Bump `version` in `Cargo.toml`, both under `[package]` and under `[package.metadata.bundle]`, and merge to `main`.
2. Tag the release and push the tag:

   ```bash
   git tag -s vX.Y.Z -m "vX.Y.Z"
   git push origin vX.Y.Z
   ```

3. `release.yml` checks the tag against both versions in `Cargo.toml`, runs formatting, Clippy, and the tests, then builds the binaries and opens a draft release with them attached:

   | Asset | Platform |
   | --- | --- |
   | `budget-tracker-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Linux x86_64, glibc 2.35+ |
   | `budget-tracker-X.Y.Z-x86_64-unknown-linux-musl.tar.gz` | Linux x86_64, static |
   | `budget-tracker-X.Y.Z-aarch64-unknown-linux-gnu.tar.gz` | Linux arm64, glibc 2.35+ |
   | `budget-tracker-X.Y.Z-aarch64-unknown-linux-musl.tar.gz` | Linux arm64, static |
   | `budget-tracker-X.Y.Z-aarch64-apple-darwin.tar.gz` | macOS arm64 |
   | `budget-tracker-X.Y.Z-x86_64-pc-windows-msvc.zip` | Windows x86_64 |
   | `SHA256SUMS` | Checksums for all of the above |

   Each archive holds the binary, `LICENSE`, and `README.md`.

4. Edit the draft. The title and the generated changelog are already filled in; add the installer, the summary, and the screenshot.
5. Publish the release. `publish-crate.yml` then checks out the tag and runs `cargo publish`.

The in-app update check reads the latest published release, so a draft stays invisible to users until step 5.

## Notes

- Re-running the release workflow on an existing release replaces the assets and leaves the title, body, and any manually uploaded files alone.
- Do not replace an asset on a release that is already published. Package manager manifests pin the SHA-256 of each archive, so changing the bytes under a published tag breaks every downstream install. Cut a patch version instead.
- To abandon a release before publishing, delete the draft and delete the tag with `git push --delete origin <tag>`. Nothing has reached crates.io yet.
- If `cargo publish` fails, run it again from Actions > Publish to crates.io > Run workflow, passing the tag.
- A release marked as a pre-release gets a draft and binaries, but is not published to crates.io.
- One-time, delete after 1.5.1 ships: the crate was renamed from `budget_tracker_tui` to `budget-tracker-tui` on 2026-08-31, and crates.io holds a deleted name for 24 hours. `cargo publish --dry-run` does not check name availability, so confirm the name is claimable before publishing the draft that triggers the real publish.
