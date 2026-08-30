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
   | `Budget_Tracker_linux` | Linux x86_64 |
   | `Budget_Tracker_MacOS` | macOS arm64 |
   | `Budget_Tracker.exe` | Windows x86_64 |

4. Edit the draft. The title and the generated changelog are already filled in; add the installer, the summary, and the screenshot.
5. Publish the release. `publish-crate.yml` then checks out the tag and runs `cargo publish`.

The in-app update check reads the latest published release, so a draft stays invisible to users until step 5.

## Notes

- Re-running the release workflow on an existing release replaces the binaries and leaves the title, body, and any manually uploaded files alone.
- To abandon a release before publishing, delete the draft and delete the tag with `git push --delete origin <tag>`. Nothing has reached crates.io yet.
- If `cargo publish` fails, run it again from Actions > Publish to crates.io > Run workflow, passing the tag.
- A release marked as a pre-release gets a draft and binaries, but is not published to crates.io.
