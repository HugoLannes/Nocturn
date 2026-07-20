# Contributing to Nocturn

Issues and pull requests are welcome. This document covers local setup and the
release/versioning steps maintainers use when cutting a new version.

## Local development

```bash
# prerequisites: Node >=18, Rust, Tauri CLI
npm install
npm run tauri:dev      # development
npm run tauri:build    # production installer
```

Before opening a pull request, make sure the following pass locally:

```bash
npm run build                                    # tsc + vite build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Reporting bugs / requesting features

Open an issue at [github.com/HugoLannes/Nocturn/issues](https://github.com/HugoLannes/Nocturn/issues)
with as much detail as you can (repro steps, monitor setup, Nocturn version from the
titlebar).

## Release process

Releases are cut from `main` via the `Release` GitHub Actions workflow
(`.github/workflows/release.yml`), which is manually triggered
(`workflow_dispatch`). Before triggering it:

1. Bump the version **in all three places** and keep them identical:
   - `package.json` (`version`)
   - `src-tauri/tauri.conf.json` (`version`)
   - `src-tauri/Cargo.toml` (`version`)
2. Commit the bump to `main`. The workflow verifies all three versions match and
   fails the release otherwise.
3. Trigger the `Release` workflow from `main`. It will:
   - fail if a tag for that version already exists;
   - fail if the updater signing secrets are not configured;
   - build the Windows installer, sign updater artifacts, and publish the
     GitHub release once all assets (including `latest.json`) are attached.

The updater's only source of truth is the `releases/latest` GitHub release, so a
release should never be published manually without `latest.json` attached.
