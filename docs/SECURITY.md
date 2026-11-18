# Security model and review

Submitted TeX and assets are untrusted input. Rust memory safety is not the TeX
security boundary. The native worker's namespaces, cleared environment, fixed
execution policy and deployment resource controls are all required. The RC's
production_ready flag remains false pending final public security/client review.

## Enforced boundaries

- Allowlisted pdfLaTeX/XeLaTeX/LuaLaTeX; fixed arguments, no shell escape,
  no project latexmk rc loading and no arbitrary client executable paths.
- Private regular-file projects: no absolute/parent/drive/UNC/backslash/control/
  shell-metacharacter paths, symlinks, ZIP duplicate/special entries or collisions.
  Inputs are bounded by body bytes, decoded/expanded bytes and file count.
- Bounded HTTP admission and blocking preparation; cancellation retains the
  preparation slot until work ends. Two shared compilation slots fail fast.
- New user/PID/mount/network/IPC/UTS namespaces, no /proc inside the worker,
  dropped capabilities, disabled further user namespaces and parent-death cleanup.
- Only approved read-only system paths and a private project are visible.
  Tokens, home directories, cloud credentials and unrelated projects are absent.
  The worker has no network; font/TeX caches stay in its bounded temporary area.
- Per-process address-space/CPU/file/descriptor limits, aggregate output/tmpfs
  bounds, protocol/log/PDF bounds, a deadline, exit-status checking and PDF parsing.
  The supervisor retains workspace/permit ownership through process reaping.
- Compose adds a non-root, read-only, no-capability/no-new-privilege container
  with aggregate memory/CPU/PID/temp-space limits and the checked-in seccomp policy.
- Remote listeners require a strong bearer token. Auth runs before preparation;
  tokens/source are not logged or inherited by workers. Use TLS at the gateway.

Exact mounts and limits are in native-worker-contract.md. The approved system
mounts include /etc/alternatives for Debian's TeX tool symlinks; this is not a
general /etc bind. Shell escape is not exposed as a request option.

## Deployment requirements and limitations

Host user-namespace/LSM configuration is part of the deployment boundary. The
application never disables host protection or bypasses failed isolation. Docker
alone with default settings may not permit nested Bubblewrap; use the reviewed
profile. Its scoped outer AppArmor exception permits mounts while seccomp and
host capability restrictions remain. The scoped systempaths=unconfined option
removes Docker's masked/read-only system paths so Bubblewrap can enforce the
new namespace's nesting limit; CI checks that host-global writes remain denied
and the host namespace limit is unchanged. Administrators should review the profile
for their host kernel. Never run privileged or mount a host Docker socket.

Native shell launchers do not create aggregate cgroups. For remote/untrusted
work use the container policy or validate a cgroup-bounded systemd installation.
A single API token authenticates a service principal, not individual end users;
tenant authorization/rate limits belong at the gateway. No shared persistent
cache/job store exists, so there is no cross-tenant artifact lookup interface.

The runtime has no Python interpreter. TeX packages requiring Python helpers,
shell escape, network downloads or external commands outside the fixed workflow
are unsupported. Do not silently substitute missing fonts/packages or enable
those capabilities to make a document compile. Add any future capability only
through a separately reviewed worker policy and explicit authorization.

## Regression evidence

Tests cover path/ZIP/decompression bounds, explicit engine handling, nonzero
exit codes, PDF validation, protocol overflow, timeout/cancellation descendants,
workspace/slot cleanup, read-only system/input mounts, cleared environment,
network isolation, bounded output tmpfs and failed-isolation fail-closed behavior.
Auth tests cover absent/wrong/scheme-mismatched tokens and public health routes.
Preparation cancellation is tested against the actual blocking-task lifecycle.
Container CI verifies the exact restricted runtime with real engines/fonts/
bibliography/browser PDFs and no Python command/library directories.

The initial Python implementation had unrestricted shell escape, arbitrary
compiler fallback, traversal/ZIP/resource issues, ignored exit status and no
OS isolation. Those vulnerable paths are removed rather than emulated. Git
history retains the audit/reference; it is not an alternative public runtime.

## Dependency and release review

Cargo.lock, the Rust toolchain, image digests, Ubuntu snapshot and Bubblewrap
checksum are pinned. RustSec audit evidence records the advisory database
revision and reports no known vulnerabilities for the reviewed lock file.
CI rechecks advisories rather than assuming an old report stays current.
Distribution license notices and the derived seccomp license remain in the
image/repository. Application release publication waits for all required checks.

Before a stable public release review kernel/LSM exposure, actual host controls,
client-owned template inputs, tenant gateway policy and dependency updates.
No claim of a completed hostile-document penetration test or external production
rollout is made by this source migration.
