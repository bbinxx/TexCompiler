# Installation

The README contains the quick install commands. Native workers are validated
on Linux x86_64 with distro TeX installed under /usr/bin and /usr/share.
An upstream TeX installation under /usr/local/texlive is not currently mounted
by the worker. Use the pinned Docker image profile for a consistent environment.

## Required components

| Component | Why |
|---|---|
| Rust 1.100.0 + Cargo | Build HTTP and worker executables |
| Bubblewrap 0.13.1 | Private user/PID/mount/network namespaces and sized tmpfs |
| util-linux prlimit | Process address-space/CPU/file-descriptor/output limits |
| pdfLaTeX, XeLaTeX, LuaLaTeX | Authoritative native engines |
| latexmk | Multipass/bibliography orchestration; rc loading disabled |
| BibTeX, Biber, MakeIndex | Bibliography/index tools where documents require them |
| TeX packages and fonts | Template, graphics, math, language and font compatibility |
| Poppler tools | Test text, pages, fonts and renderings |
| Node 22+ and Chrome | Optional frontend tests and browser smoke verification |

## Bubblewrap on Ubuntu 24.04

The bundled version may lack the validated behavior. After installing Meson,
Ninja, gcc, pkg-config, libcap-dev, curl and xz-utils as shown in the README,
build the checksum-verified release in a temporary directory:

```sh
mkdir -p /tmp/texcompiler-bwrap-install
cd /tmp/texcompiler-bwrap-install
curl -fsSL --retry 3 https://github.com/containers/bubblewrap/releases/download/v0.13.1/bubblewrap-0.13.1.tar.xz -o bubblewrap.tar.xz
echo '0e85c8702248b8dd64144f594147553650cdd0e4c951690345faace79c217c4d  bubblewrap.tar.xz' | sha256sum --check
tar -xf bubblewrap.tar.xz
meson setup build bubblewrap-0.13.1 -Dman=disabled -Dtests=false -Dselinux=disabled
meson compile -C build
```

The service currently uses /usr/bin/bwrap. An administrator can install the
verified binary there, deliberately replacing the distro copy:

```sh
sudo install -m 0755 build/bwrap /usr/bin/bwrap
/usr/bin/bwrap --version
```

Track that local replacement when upgrading the distribution; a package update
can replace it. Prefer a managed package or the supplied container for production.
No setuid bit or privileged worker is required. The launcher never installs
packages, replaces system tools or changes host kernel settings automatically.

## Namespace policy

The host must permit unprivileged user namespaces. Ubuntu's AppArmor policy can
block them or block network-namespace setup despite an installed Bubblewrap.
An administrator must provide an appropriate service policy. Do not globally
disable host protection merely to make /ready green. The CI workflow adjusts
only a disposable VM; that command is not an installation recommendation.

The Compose policy is described in deploy/README.md. It drops host capabilities
and uses a scoped seccomp profile for namespace setup. A host/platform that
cannot support this policy is unsupported; the service fails closed.

## Verify dependencies and startup

```sh
rustc --version
/usr/bin/bwrap --version
prlimit --version
pdflatex --version
xelatex --version
lualatex --version
latexmk -version
biber --version
bibtex --version
command -v makeindex
```

Some TeX tools use different version/help switches; their presence is also
checked by the equipped native suite. Start the service and inspect /health
and /ready. A successful startup compile, rather than executable discovery
alone, establishes compilation_ready. Do not bypass failed isolation.

## Packages and fonts

The compatible Docker profile is the tested practical package set. The full
profile additionally includes texlive-fonts-extra and texlive-lang-all. Neither
means all external-command CTAN workflows are enabled. Install additional
trusted packages by rebuilding an image; never install from submitted documents.
Use XeLaTeX/LuaLaTeX with uploaded project-relative OTF/TTF files or installed
font families. Missing fonts are errors; unrelated substitution is not applied.

Python, pip, FastAPI and a virtual environment are not TexCompiler prerequisites.
Distribution TeX bundles may bring Python-dependent utilities on a native host;
the application does not call them. The final container runtime omits Python
interpreters and libraries and does not support those external helpers.
