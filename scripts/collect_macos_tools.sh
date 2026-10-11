#!/bin/bash
# Collect the current Mac architecture's Homebrew CLI tools and rewrite their
# non-system dylib dependencies so the app bundle is self-contained.
#
# Two sets are collected:
#   resources/bin|lib          — image CLI tools (shared by Direct and Swift)
#   resources/video/bin|lib    — ffmpeg + ffprobe for the Direct video engine

set -euo pipefail

DEST_ROOT="${1:-$(cd "$(dirname "$0")/../src-tauri/resources" && pwd)}"

CURRENT_BIN_DIR=""

resolve_dependency() {
  local binary="$1"
  local dependency="$2"
  local loader_dir
  local candidate
  loader_dir="$(dirname "${binary}")"

  case "${dependency}" in
    /System/*|/usr/lib/*)
      return 1
      ;;
    /*)
      [ -f "${dependency}" ] && { printf '%s\n' "${dependency}"; return 0; }
      ;;
    @loader_path/*)
      candidate="${loader_dir}/${dependency#@loader_path/}"
      [ -f "${candidate}" ] && { printf '%s\n' "${candidate}"; return 0; }
      ;;
    @executable_path/*)
      candidate="${CURRENT_BIN_DIR}/${dependency#@executable_path/}"
      [ -f "${candidate}" ] && { printf '%s\n' "${candidate}"; return 0; }
      ;;
    @rpath/*)
      while IFS= read -r rpath; do
        rpath="${rpath//@loader_path/${loader_dir}}"
        rpath="${rpath//@executable_path/${CURRENT_BIN_DIR}}"
        candidate="${rpath}/${dependency#@rpath/}"
        [ -f "${candidate}" ] && { printf '%s\n' "${candidate}"; return 0; }
      done < <(otool -l "${binary}" | awk '/path .*\(offset/{print $2}')
      ;;
  esac

  if command -v brew >/dev/null; then
    candidate="$(find "$(brew --cellar)" -name "$(basename "${dependency}")" -print -quit)"
    [ -n "${candidate}" ] && { printf '%s\n' "${candidate}"; return 0; }
  fi

  echo "Unable to resolve dependency ${dependency} for ${binary}" >&2
  return 2
}

# collect_set BIN_DIR LIB_DIR LIB_ID_PREFIX TOOL...
# LIB_ID_PREFIX is how a dylib refers to LIB_DIR relative to the *binary's own
# directory* (@executable_path), e.g. "lib" for Resources/bin → Resources/lib
# and for Resources/video/bin → Resources/video/lib.
collect_set() {
  local bin_dir="$1"
  local lib_dir="$2"
  local lib_prefix="$3"
  shift 3
  local tools=("$@")

  mkdir -p "${bin_dir}" "${lib_dir}"
  find "${bin_dir}" -mindepth 1 -maxdepth 1 -type f -delete
  find "${lib_dir}" -mindepth 1 -maxdepth 1 -type f -delete

  for tool in "${tools[@]}"; do
    local source_path
    source_path="$(command -v "${tool}" || true)"
    if [ -z "${source_path}" ]; then
      echo "Missing required tool: ${tool}" >&2
      exit 1
    fi
    cp -L "${source_path}" "${bin_dir}/${tool}"
    chmod 755 "${bin_dir}/${tool}"
  done

  CURRENT_BIN_DIR="${bin_dir}"

  # Process newly copied libraries in subsequent passes. This avoids
  # dylibbundler's unbounded retry loop when Homebrew uses @rpath install names
  # (notably mozjpeg's @rpath/libjpeg.8.dylib on macOS 26 runners).
  while :; do
    local copied=0
    for binary in "${bin_dir}"/* "${lib_dir}"/*.dylib; do
      [ -f "${binary}" ] || continue
      while IFS= read -r dependency; do
        [ -n "${dependency}" ] || continue
        local status=0
        local resolved
        resolved="$(resolve_dependency "${binary}" "${dependency}")" || status=$?
        if [ "${status}" -eq 1 ]; then
          continue
        fi
        [ "${status}" -eq 0 ] || exit "${status}"
        local destination="${lib_dir}/$(basename "${resolved}")"
        if [ ! -f "${destination}" ]; then
          cp -L "${resolved}" "${destination}"
          chmod 755 "${destination}"
          install_name_tool -id "@executable_path/../${lib_prefix}/$(basename "${destination}")" \
            "${destination}" 2>/dev/null
          copied=1
        fi
        install_name_tool -change "${dependency}" \
          "@executable_path/../${lib_prefix}/$(basename "${destination}")" "${binary}" 2>/dev/null
      done < <(otool -L "${binary}" | tail -n +2 | awk '{print $1}')
    done
    [ "${copied}" -eq 1 ] || break
  done

  local expected_arch
  expected_arch="$(uname -m)"
  for binary in "${bin_dir}"/* "${lib_dir}"/*.dylib; do
    [ -f "${binary}" ] || continue
    file "${binary}" | grep -q "${expected_arch}" || {
      echo "Architecture mismatch: ${binary}" >&2
      exit 1
    }
    # install_name_tool invalidates the existing signature; without a fresh
    # (ad-hoc) signature Apple Silicon kills the binary on launch with
    # SIGKILL. notarize.sh later re-signs everything with the real identity.
    codesign --force --sign - "${binary}" 2>/dev/null \
      || { echo "Ad-hoc codesign failed: ${binary}" >&2; exit 1; }
  done

  echo "Collected ${#tools[@]} tools -> ${bin_dir} ($(ls "${lib_dir}" | wc -l | tr -d ' ') dylibs)"
}

collect_set "${DEST_ROOT}/bin" "${DEST_ROOT}/lib" "lib" \
  pngquant oxipng cjpeg gifsicle cwebp cjxl avifenc

# FFmpeg's license depends on the build configuration. A Homebrew ffmpeg
# typically enables GPL encoders (libx264/x265) and inherits the GPL. Before a
# public release, the release owner must either bundle an LGPL build or provide
# the corresponding source — see THIRD_PARTY_NOTICES.md.
collect_set "${DEST_ROOT}/video/bin" "${DEST_ROOT}/video/lib" "lib" \
  ffmpeg ffprobe

echo "Collected macOS tools for $(uname -m): ${DEST_ROOT}"
