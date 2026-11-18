# Runtime policy

Compose enforces a non-root UID, no capabilities, no privilege escalation,
read-only root, a 512 MiB private temporary filesystem, 128 processes, 3 GiB
aggregate memory and two CPUs. The Rust supervisor additionally limits jobs,
TeX execution time, address space, output and workspace inputs. Worker namespaces
have no host project directories, credentials, network or writable system paths.

seccomp.json derives from https://github.com/moby/profiles/blob/main/seccomp/default.json
(Apache-2.0). It retains the default deny policy and adds the namespace/mount
syscalls Bubblewrap needs in its own unprivileged user namespace; it does not
grant host capabilities or permit bpf/perf/keyring operations. Docker's outer
system-path masks are disabled for this service so Bubblewrap can set its own
new user namespace's max_user_namespaces limit and block further nesting. This removes ALL Docker
masked/read-only system-path defaults, not just one sysctl; the outer service
is still non-root without capabilities. CI checks host namespace limits are
unchanged and a host-global sysctl write is denied. The worker does not mount
/proc at all and cannot create another user namespace. Docker's outer
AppArmor mount restriction is disabled only for this service. On AppArmor hosts,
an administrator must separately allow unprivileged user namespaces. The service
never changes host kernel settings. CI does so only on its disposable runner.

Do not run this image privileged or mount a Docker socket, host home, credentials
or persistent user project directories into it. No fallback bypasses failed
namespace setup: /ready reports 503 and compilation remains unavailable.
Platforms that cannot configure nested user namespaces are unsupported for this
worker profile; use a compatible Linux host instead of disabling isolation.

Dockerfile.rust assembles a runtime from a pinned Ubuntu image and canonical
HTTPS package repositories, recording exact resolved package versions in /app.
Set build argument UBUNTU_SNAPSHOT to a timestamp for dated archive builds when
the snapshot service is available. Fresh default builds can receive distribution
security updates; deploy immutable image digests to retain an exact environment.
Python is a
build-time dependency of Ubuntu's TeX bundle and Meson. Interpreters and Python
libraries are excluded from the final runtime rootfs; the final stage inherits
no Python installation layers. Python-dependent TeX utilities (including Python
syntax-highlighting helpers) and shell-escape workflows are intentionally
unsupported. The application and all compilation orchestration are Rust.

The compatible package profile covers the tested native engines, fonts,
custom packages/classes, graphics, tables and BibTeX/Biber. Select
TEXCOMPILER_TEX_PROFILE=full for the broader font/language installation; it is
larger and must be validated against your document corpus. This is not an
assumption that all CTAN or font combinations are compatible.
