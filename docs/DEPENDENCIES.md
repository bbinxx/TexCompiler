# Dependency and license review

Rust uses the pinned toolchain and Cargo.lock. The dependency workflow installs
cargo-audit 0.23.0 and checks the current RustSec database without ignored
advisories. The migration evidence records the exact reviewed advisory revision;
it does not replace future audits after new advisories or dependency updates.

scripts/verification/dependencies.cjs checks exact reviewed license expressions
from Cargo metadata, including target-specific dependencies. A new/unknown or
missing dependency license fails until reviewed. Permissive alternatives are
selected where offered; combined MIT/BSD/Apache/Unicode notice obligations
remain. This policy does not assign a license to the application itself.

The runtime and release package preserve collected Rust dependency notices.
The container also retains distribution package copyright/common-license data;
do not remove /usr/share/doc merely to reduce image size. The build package
manifest includes build-time dependencies, which is distinct from runtime
interpreter availability. The final rootfs excludes Python interpreters/libs.

The Docker base digests and Bubblewrap archive checksum are pinned. The image records resolved package versions, and dated snapshots are optional. TeX Live/packages/fonts may have varied licenses; preserve their bundled
notices and review redistribution obligations when adding fonts/packages or
publishing images. The derived Docker seccomp profile's Apache license is
included under deploy/LICENSE.seccomp. User-supplied font license obligations
remain the uploader's responsibility; no uploaded font is added to the image.
