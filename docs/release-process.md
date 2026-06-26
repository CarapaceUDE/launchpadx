# Release Process

## Model

| Channel | What ships | Who gets it |
| ------- | ---------- | ----------- |
| GitHub (`master`) | Source code (MIT) | Everyone |
| GitHub Release (tag) | Source archive + release notes | Everyone |
| Patreon | Official pre-built binaries | Supporters |
| Build from source | Your own binary | Anyone with Rust/Node |

Official binaries are **not** attached to public GitHub Releases. See [OFFICIAL_BUILDS.md](../OFFICIAL_BUILDS.md).

## Branching

- `master` — always releasable
- `feature/*`, `fix/*` — short-lived branches merged via PR

## Day-to-day change flow

1. Branch from `master`
2. Implement + run the [testing checks](../README.md#testing) (and `cd web && npm run build` if UI changed)
3. Open PR → wait for CI green → squash merge
4. Update `CHANGELOG.md` for user-visible changes

## Shipping a version

### 1. Prepare `master`

- Bump `version` in `Cargo.toml` and `web/package.json` if needed
- Update `CHANGELOG.md`
- Merge all pending PRs

### 2. Tag the release

```powershell
git checkout master
git pull origin master
git tag v0.1.0
git push origin v0.1.0
```

Pushing a `v*` tag triggers:

- **Release** workflow — public GitHub Release (source only, no binaries)
- **Build Official Binaries** workflow — private CI artifacts for Patreon upload

### 3. Publish to Patreon

1. Open **Actions** → **Build Official Binaries** → select the tag run
2. Download each platform artifact
3. Upload to your Patreon post / supporter download area
4. Optionally note the version in your Patreon changelog

### 4. Verify

- GitHub **Releases** page shows the tag with release notes (no `.exe` attached)
- You can install from a downloaded artifact before posting to Patreon
- Update Patreon post with version + changelog link

## GitHub settings (recommended)

- Protect `master`: require PR + passing CI
- Squash merge PRs
- Do not attach binaries to public releases manually

## Versioning

Use [SemVer](https://semver.org/):

- `v0.1.x` — pre-1.0 development
- Patch — bug fixes
- Minor — new features
- Major — breaking changes or 1.0 stability promise