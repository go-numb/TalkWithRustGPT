#!/usr/bin/env bash
set -euo pipefail

SRC_DIR="${SRC_DIR:-/src}"
OUT_DIR="${OUT_DIR:-/out}"
BUILD_DIR="${BUILD_DIR:-/build}"
TARGET="${TARGET:-x86_64-pc-windows-gnu}"

echo "[1/4] Prepare build dir..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR" "$OUT_DIR"

# Copy sources into container FS (keeps host/WSL clean: no node_modules/target on host)
rsync -a --delete \
  --exclude node_modules \
  --exclude target \
  --exclude dist \
  --exclude .git \
  "${SRC_DIR}/" "${BUILD_DIR}/"

cd "$BUILD_DIR"

echo "[2/4] Frontend build (npm)..."
if [ -f package-lock.json ]; then
  npm ci
else
  npm install
fi

echo "[3/4] Tauri build (Windows .exe via npm tauri build)..."
# NOTE:
# - Use `--no-bundle` to avoid Windows installer creation (MSI/NSIS), which generally requires Windows.
# - `tauri build` will execute beforeBuildCommand (npm run build) from tauri.conf.json.
export TAURI_SKIP_UPDATE_CHECK=1
npm run tauri build -- --target "$TARGET" --no-bundle

echo "[4/4] Collect artifacts..."
EXE_DIR="src-tauri/target/${TARGET}/release"
shopt -s nullglob
EXES=("${EXE_DIR}"/*.exe)

if [ ${#EXES[@]} -eq 0 ]; then
  echo "ERROR: .exe not found in ${EXE_DIR}"
  echo "Hint: build output directory listing:"
  ls -la "${EXE_DIR}" || true
  exit 1
fi

# Copy the main exe(s)
cp -fv "${EXES[@]}" "${OUT_DIR}/"

# Helper: try to locate a DLL by name in common MinGW and build output locations.
find_dll() {
  local dll_name="$1"

  # Prefer build output (e.g., WebView2Loader.dll produced by a build script)
  local from_build
  from_build="$(find "src-tauri/target/${TARGET}" -type f -name "${dll_name}" -print -quit 2>/dev/null || true)"
  if [ -n "${from_build}" ]; then
    echo "${from_build}"
    return 0
  fi

  # Common MinGW runtime locations
  local gcc_root
  gcc_root="$(ls -d /usr/lib/gcc/x86_64-w64-mingw32/* 2>/dev/null | head -n 1 || true)"
  local candidates=(
    "/usr/x86_64-w64-mingw32/bin/${dll_name}"
    "/usr/lib/gcc/x86_64-w64-mingw32/${dll_name}"
    "${gcc_root}/${dll_name}"
  )
  local path
  for path in "${candidates[@]}"; do
    if [ -n "${path}" ] && [ -f "${path}" ]; then
      echo "${path}"
      return 0
    fi
  done

  return 1
}

# 1) Ensure WebView2Loader.dll is co-located with the .exe (required at runtime).
if dll_path="$(find_dll WebView2Loader.dll 2>/dev/null)"; then
  cp -fv "${dll_path}" "${OUT_DIR}/WebView2Loader.dll"
else
  echo "WARN: WebView2Loader.dll was not found in the build output or toolchain."
  echo "      The produced .exe will fail to start on Windows without it."
fi

# 2) For GNU builds, also collect MinGW runtime DLL dependencies.
#    This makes the produced folder runnable on a clean Windows machine.
collect_deps_for_exe() {
  local exe_path="$1"

  if ! command -v x86_64-w64-mingw32-objdump >/dev/null 2>&1; then
    echo "WARN: objdump not available; skipping dependency collection for ${exe_path}" >&2
    return 0
  fi

  local dll
  # Extract referenced DLL names
  mapfile -t deps < <(x86_64-w64-mingw32-objdump -p "${exe_path}" \
    | awk -F': ' '/DLL Name/ {print $2}' \
    | tr -d '\r' \
    | sort -u)

  for dll in "${deps[@]}"; do
    # Skip Windows system DLLs
    case "${dll^^}" in
      KERNEL32.DLL|USER32.DLL|GDI32.DLL|SHELL32.DLL|ADVAPI32.DLL|OLE32.DLL|OLEAUT32.DLL|COMDLG32.DLL|COMCTL32.DLL|SHLWAPI.DLL|WS2_32.DLL|WINMM.DLL|VERSION.DLL|CRYPT32.DLL|UXTHEME.DLL|DWMAPI.DLL|IMM32.DLL|NTDLL.DLL)
        continue
        ;;
    esac

    # Already collected?
    if [ -f "${OUT_DIR}/${dll}" ]; then
      continue
    fi

    if dll_path="$(find_dll "${dll}" 2>/dev/null)"; then
      cp -fv "${dll_path}" "${OUT_DIR}/${dll}"
    fi
  done
}

for exe in "${EXES[@]}"; do
  collect_deps_for_exe "${exe}"
done

echo "OK: copied to ${OUT_DIR}"
ls -la "${OUT_DIR}"
