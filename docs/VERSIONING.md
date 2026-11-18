# Versioning and release policy

TexCompiler uses semantic versions with an explicit development stage for the
Rust migration. A major migration milestone does not mean a stable major
release has shipped.

## Current version identities

| Identity | Source | Meaning |
|---|---|---|
| Python runtime `1.3.0` | app/config.py APP_VERSION | Version currently exposed by Python health/IDE/OpenAPI |
| Existing tag `v1.3.1` | Git release history | Latest existing v1 tag; preserve its history |
| Rust `2.0.0-dev.2` | Cargo.toml and Cargo.lock | Unreleased experimental native worker milestone |
| Future `v2.0.0` | No tag yet | Stable release only after migration gates pass |

The v1 tag/runtime mismatch is recorded, not silently corrected. Reconcile
runtime metadata with the intended release in a focused release PR before
publishing another v1 version. Do not rename existing tags.

## Choosing a version

- Major: incompatible public contracts or substantial runtime/deployment
  transitions. The Python-to-Rust production transition is assigned v2 while
  preserving the legacy API wherever practical.
- Minor: compatible user-facing additions within a stable major line.
- Patch: compatible fixes within a stable major line.
- Development milestones: `2.0.0-dev.N`, incrementing N for an explicitly
  designated milestone, not every documentation commit or merged PR.
- Alpha, beta and release candidates: `2.0.0-alpha.N`, `2.0.0-beta.N`, then
  `2.0.0-rc.N` only when their documented scope has actually been verified.

`/api/v1` identifies the versioned HTTP contract and is independent of the
service's v2 implementation version. Do not rename API paths simply because
Cargo's major version changes.

## Release workflow

1. Use a focused `chore/release-<version>` branch from validated dev. Update
   Cargo.toml/Cargo.lock together for Rust, or APP_VERSION for a v1 release;
   update README status and CHANGELOG with implemented behavior and limitations.
2. Run relevant format, lint, test, compatibility and deployment checks. Review
   and merge the PR into dev only after required checks pass. Never commit the
   version bump directly on dev.
3. Create a release tag only for the exact validated commit and explicitly
   authorized publication. This policy does not authorize tagging or publishing.
4. Mark every development/alpha/beta/rc GitHub release as a prerelease. The
   existing tag-triggered workflow does not enforce this; update and verify its
   prerelease classification and test gates in a focused workflow PR before
   publishing any v2 prerelease. Do not trigger it with a development tag now.
5. Publish stable `v2.0.0` only after the compatibility, isolation/security,
   native-engine/font/project, real-client, deployment and rollback gates in
   MIGRATION.md are verified. Remove the development suffix only then.

Keep an Unreleased changelog entry between releases. Describe major work
prominently in README even while unreleased; separate implemented behavior,
remaining work and known risks. Keep benchmark claims tied to measurements.
No version, release tag, or deployment is changed by this documentation update.
