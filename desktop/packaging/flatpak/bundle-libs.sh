#!/usr/bin/env bash
# bundle-libs.sh: Identify host libraries required by the binary that are missing
# from the Flatpak runtime (org.gnome.Platform//47) and bundle them into packaging/flatpak/lib/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN="${APP_DIR}/src-tauri/target/release/roletect"
DEST_LIB="${SCRIPT_DIR}/lib"

if [ ! -f "${BIN}" ]; then
  echo "Error: Binary not found at ${BIN}. Please run 'cargo build --release' in src-tauri first." >&2
  exit 1
fi

mkdir -p "${DEST_LIB}"

echo "Analyzing dynamic library dependencies of ${BIN}..."

# Get all libraries linked to the binary (absolute paths)
HOST_LIBS=$(ldd "${BIN}" | awk '/=> \// {print $3}' | sort -u)

# Find which libraries are missing inside the org.gnome.Platform//47 runtime
MISSING_SONAMES=()

if command -v flatpak &>/dev/null && flatpak info org.gnome.Platform//47 &>/dev/null; then
  echo "Checking against org.gnome.Platform//47 runtime sandbox..."
  for lib in ${HOST_LIBS}; do
    soname="$(basename "${lib}")"
    # Check if this soname exists in the Flatpak runtime /usr/lib
    if ! flatpak run --command=sh org.gnome.Platform//47 -c "find /usr/lib /usr/lib64 /usr/lib/x86_64-linux-gnu -name '${soname}' 2>/dev/null | grep -q ." &>/dev/null; then
      MISSING_SONAMES+=("${soname}")
    fi
  done
else
  echo "Warning: Flatpak runtime org.gnome.Platform//47 not found or flatpak CLI unavailable."
  echo "Falling back to bundling ICU and known non-standard desktop libraries."
  MISSING_SONAMES=("libicuuc.so.78" "libicui18n.so.78" "libicudata.so.78")
fi

echo "Missing libraries detected: ${#MISSING_SONAMES[@]}"

for soname in "${MISSING_SONAMES[@]}"; do
  echo " -> Bundling: ${soname}"
  # Locate on host
  host_path=$(ldd "${BIN}" | awk -v s="${soname}" '$1 == s {print $3}')
  if [ -z "${host_path}" ] || [ ! -f "${host_path}" ]; then
    host_path=$(find /lib /usr/lib /usr/local/lib -name "${soname}" 2>/dev/null | head -n1 || true)
  fi

  if [ -n "${host_path}" ] && [ -f "${host_path}" ]; then
    # Resolve real file if symlink
    real_target=$(readlink -f "${host_path}")
    real_name=$(basename "${real_target}")
    cp -a "${real_target}" "${DEST_LIB}/${real_name}"
    # If the soname is a symlink, create or copy the symlink too
    if [ "${soname}" != "${real_name}" ]; then
      ln -sf "${real_name}" "${DEST_LIB}/${soname}"
    fi
  else
    echo "    Warning: Could not locate ${soname} on host system."
  fi
done

echo "Bundling complete. Libraries placed in ${DEST_LIB}:"
ls -lh "${DEST_LIB}"
