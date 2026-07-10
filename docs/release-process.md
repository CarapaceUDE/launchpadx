# Release Process

LaunchPadX uses one GitHub Actions workflow to build and publish native Windows, macOS, and Linux archives. A release stays in draft until all three builds succeed.

## First-time repository settings

1. Open **Settings → Actions → General** on GitHub.
2. Under **Workflow permissions**, select **Read and write permissions**.
3. Optionally protect `master` under **Settings → Branches** and require the `rust-checks` CI job.

The workflow uses only GitHub's built-in `GITHUB_TOKEN`; no release secret is required.

Windows releases are unsigned unless a trusted signing method is configured. See [Windows code signing](windows-signing.md) for Azure Artifact Signing setup and the PFX alternative.

## Ship a version

1. Make sure CI is green on `master`.
2. Update the versions in `Cargo.toml` and `web/package.json` and update `CHANGELOG.md`.
3. Commit and push those changes.
4. Create and push a matching version tag:

   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```

5. Open the repository's **Actions** tab and watch **Build and release**.
6. Download each archive from the new GitHub Release and smoke-test it.

Pushing a `v*` tag automatically creates a draft release, builds all platforms, uploads their archives directly to that release, and publishes it only after all builds pass. Direct release uploads avoid GitHub Actions artifact-storage quotas.

## Retry a failed release

Fix the problem on the tagged commit (or move the tag only if the release has not been announced), then open **Actions → Build and release → Run workflow** and enter the existing tag. Uploads use `--clobber`, so retrying safely replaces same-named assets.

## Reuse in another Rust project

Copy `.github/workflows/release.yml`. Its reusable interface accepts:

- `tag`: an existing `v*` tag
- `binary-name`: the Cargo binary name, defaulting to `launchpadx`

Another workflow can call it like this:

```yaml
jobs:
  release:
    permissions:
      contents: write
    uses: your-owner/your-repo/.github/workflows/release.yml@master
    with:
      tag: ${{ github.ref_name }}
      binary-name: your-app
```

The copied project must use the same Rust-plus-`web/` layout. For a different frontend layout, edit the Node setup and **Build web UI** step.

## Versioning

Use semantic versions: patch for fixes, minor for backwards-compatible features, and major for breaking changes or the 1.0 stability promise.
