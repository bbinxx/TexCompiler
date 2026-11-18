# Deployment and operations

## Supported deployment

Use a Linux x86_64 host that permits unprivileged user namespaces and the worker
mount/network policy. The validated profile is Compose on Linux, with pinned
Rust/Ubuntu/Bubblewrap versions and a recorded TeX package environment. See
INSTALLATION.md and deploy/README.md for prerequisites. No worker fallback runs
outside its sandbox when host configuration is incompatible.

The service's `production_ready` field remains false pending stable-release
acceptance. This repository migration does not switch public traffic or create
a stable tag. Test your own client/template corpus before a public rollout.

## Deployment files

| File | Purpose |
|---|---|
| Dockerfile | Multi-stage Rust build, pinned TeX environment and Python-free runtime |
| compose.yaml | Non-root, read-only, capability/resource/seccomp container policy |
| deploy/seccomp.json | Docker-derived syscall policy with required namespace operations |
| deploy/systemd/texcompiler.service | Native Linux service template with aggregate limits |
| deploy/texcompiler.env.example | Environment variable example; no real secret |
| Procfile | Prebuilt Rust process entry point for compatible platform hosts |
| scripts/dev/run-rust.sh | Optional development build/run helper |
| .github/workflows/container.yml | Exact container API/font/bibliography/browser gate |
| .github/workflows/release.yml | Release publication only after required checks |

The old Python Dockerfile, Python launchers, Python CI and Render blueprint are
removed. Standard hosted platforms that deny namespace setup cannot run this
worker profile. A Procfile alone does not install TeX or grant those permissions.

## Docker Compose

Follow the [Docker installation steps](INSTALLATION.md#docker-host-setup)
first. From the repository root, create a local `.env` once. This command
refuses to overwrite an existing file. If you already have one, add
`TEXCOMPILER_API_TOKEN` to it manually and keep its permissions restricted.

```sh
(set -euC; umask 077; token=$(openssl rand -hex 32); printf 'TEXCOMPILER_API_TOKEN=%s\n' "$token" > .env)
docker compose config --quiet
docker compose up --build -d --wait
docker compose ps
curl --fail http://127.0.0.1:8001/ready
```

Compose reads `.env` from the project directory. The file is ignored by Git
and excluded from the image build context. Use `config --quiet` for validation
so the interpolated token stays out of terminal output. Exported shell variables
override `.env` values when both are set.

Open `http://127.0.0.1:8001` and enter the token in **Settings**, or generate a
PDF through the API. The example below assumes `.env` contains the token line
created above. Adjust the URL if you change the host port.

```sh
token=$(sed -n 's/^TEXCOMPILER_API_TOKEN=//p' .env)
curl --fail-with-body http://127.0.0.1:8001/compile \
  -H "Authorization: Bearer $token" \
  -H 'Content-Type: application/json' \
  --data-binary '{"code":"\\documentclass{article}\\begin{document}Hello from Docker\\end{document}","compiler":"pdflatex"}' \
  -o docker-test.pdf
unset token
```

Keep the generated token in your secret manager; do not paste it into source
control, public frontend bundles or URLs. Put it in the IDE Settings for a
trusted browser session, or attach an Authorization: Bearer header from a
trusted client/gateway. Compose publishes only loopback port 8001 by default.

```sh
# Different host port:
TEXCOMPILER_PORT=9000 docker compose up -d
# Additional fonts/languages (larger image):
TEXCOMPILER_TEX_PROFILE=full docker compose up --build -d --wait
# Inspect or stop:
docker compose ps
docker compose logs --tail=100
docker compose down
```

For a persistent port or profile, set `TEXCOMPILER_PORT=9000` or
`TEXCOMPILER_TEX_PROFILE=full` in `.env`. The port setting changes the host
port only. The container still listens on 8000. If startup fails, inspect
`docker compose logs --tail=100 texcompiler`. A namespace or mount error means
the host policy needs to support Bubblewrap. See the [runtime policy](../deploy/README.md).

The compatible profile is the tested practical set; full adds broader fonts
and languages. The TeX footprint remains substantial. Record the image ID and
package manifest/snapshot for each rollout. Images are immutable: add trusted
packages/fonts by rebuilding rather than installing from submitted source.

## Docker without Compose

Use the same isolation and resource policy:

```sh
docker build -t texcompiler:local .
export TEXCOMPILER_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm --init --name texcompiler \
  --user 10001:10001 --read-only --cap-drop ALL \
  --security-opt no-new-privileges:true \
  --security-opt seccomp=deploy/seccomp.json \
  --security-opt apparmor=unconfined \
  --security-opt systempaths=unconfined \
  --pids-limit 128 --memory 3g --cpus 2 \
  --tmpfs /tmp:rw,nosuid,nodev,size=536870912,mode=1777 \
  -p 127.0.0.1:8001:8000 -e TEXCOMPILER_API_TOKEN \
  texcompiler:local
```

The scoped AppArmor exception permits nested mounts. systempaths=unconfined
removes Docker's masked/read-only system-path defaults so Bubblewrap can
block further user namespaces within its own new namespace. This broadens the
outer service's visible system paths; it is not a host-wide policy change. The seccomp filter and dropped host capabilities remain active.
Never use privileged mode or mount the Docker socket/host credentials. Never
disable the inner worker's isolation to make an unsupported host work.

## Native binary deployment without shell launchers

Install native dependencies from INSTALLATION.md, then build both executables:

```sh
cargo build --locked --release --bins
TEXCOMPILER_BIND=127.0.0.1:8001 ./target/release/texcompiler
```

Both release binaries must be kept beside one another. TEXCOMPILER_WORKER can
override the helper path only as an administrator-controlled same-build path.
For a platform-managed port on a compatible Linux host:

```sh
export TEXCOMPILER_API_TOKEN="$(openssl rand -hex 32)"
PORT=10000 ./target/release/texcompiler
```

PORT binds 0.0.0.0 and therefore requires authentication. TEXCOMPILER_BIND takes
precedence; it must be a numeric IPv4/IPv6 socket address. The Procfile assumes
the two release binaries and native dependencies were built/installed already.

## systemd template

Create a dedicated texcompiler system user and install both binaries into
/usr/local/lib/texcompiler. Install deploy/systemd/texcompiler.service into the
system manager's unit directory. Copy deploy/texcompiler.env.example to
/etc/texcompiler/texcompiler.env, set the token/listener, and restrict that file
to root (mode 0600). Review the unit's hardening against your host namespace
policy; this template is not an automatic host installer.

For a first installation after building the release binaries:

```sh
sudo useradd --system --home-dir /nonexistent --shell /usr/sbin/nologin texcompiler
sudo install -d -m 0755 /usr/local/lib/texcompiler /etc/texcompiler
sudo install -m 0755 target/release/texcompiler target/release/texcompiler-worker /usr/local/lib/texcompiler/
sudo install -m 0644 deploy/systemd/texcompiler.service /etc/systemd/system/texcompiler.service
sudo install -m 0600 deploy/texcompiler.env.example /etc/texcompiler/texcompiler.env
sudoedit /etc/texcompiler/texcompiler.env
```

Create the account only once. Set a real token before using a remote bind or
forwarding authenticated gateway traffic, then enable the unit:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now texcompiler
sudo systemctl status texcompiler
sudo journalctl -u texcompiler -f
```

The unit limits memory, CPU and tasks, uses a private temporary directory,
read-only system paths and an unprivileged account. It intentionally does not
block user namespaces, which Bubblewrap requires. Host AppArmor/user-namespace
policy remains the administrator's responsibility. The systemd template must
be validated on your host before claiming a production deployment.

## Reverse proxy, TLS and authentication

Expose a TLS reverse proxy in front of the loopback listener. Limit request
bodies to 32 MiB or less, bound connections and use an upstream timeout longer
than the 120-second compilation deadline (the service request deadline is
150 seconds). Send the bearer token to the API without logging it. Disable
proxy caching for PDFs/errors. Add tenant authentication/authorization at the
gateway; a shared service token is not an end-user identity model.

The API retains no shared job cache or persistent project data. Each request
gets a private workspace that is cleaned after the process tree is reaped.
Service logs exclude submitted source and tokens; client-facing diagnostics
refer only to sandbox paths. Use /health for liveness and /ready for startup
compilation readiness; the latter returns 503 when isolation/TeX probing fails.

## Upgrade and rollback

1. Record the current image digest, configuration and secret references.
2. Build a new image from a reviewed commit and keep its immutable digest.
3. Run CI and your template corpus, then start the candidate on a second port.
4. Check readiness, authenticated compile, all engines/fonts/bibliography and
   the browser. Switch proxy traffic only after those acceptance checks pass.
5. Keep the previous instance/image during the observation window. Roll back
   by restoring its routing/configuration; never overwrite user documents.

The recorded migration rehearsal changed local routing Rust → Python → Rust;
it was not a rehearsal of your external production infrastructure. The last
Python source snapshot is commit 8f87470, recoverable in a separate worktree.
It retains known vulnerabilities and is suitable only for a controlled rollback
window with trusted input. Retain your own prior deployed image digest if you
intend to restore the historical deployment. No stable release is tagged here.

## Troubleshooting

| Symptom | Check |
|---|---|
| Startup requires token | Non-loopback bind/PORT requires a 32–256 character ASCII token |
| /ready returns 503 | Logs, Bubblewrap version, user namespaces, host security policy, helper path and installed TeX |
| 401 compile | Bearer header or IDE Settings token |
| 429 compile | All worker/preparation slots are busy; retry with bounded backoff |
| Missing package/font | Upload project-relative assets or rebuild the trusted TeX/font profile |
| Bibliography unresolved | latexmk, BibTeX/Biber and bibliography assets must be installed/present |
| Docker build fills disk | TeX is large; build on adequately sized storage and avoid changing the host daemon's data root blindly |
| Shell-escape/Python utility fails | That capability is intentionally disabled; do not enable it on untrusted documents |

Caching, asynchronous job APIs, metrics and optional WASM are separate future
features; no multi-host worker orchestration is assumed by this deployment.
