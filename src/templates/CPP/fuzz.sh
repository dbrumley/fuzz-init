#!/usr/bin/env bash
set -euo pipefail

# Engines we support via workflow presets
ENGINES=("libfuzzer" "afl" "hongfuzz" "native")

usage() {
  cat <<'USAGE'
Usage:
  ./fuzz.sh build [ENGINE]      # Build all engines (default) or just one
  ./fuzz.sh test  [ENGINE] [S]  # Quick sanity fuzz; S seconds (default 10)

Engines:
  libfuzzer   → workflow preset: fuzz-build-libfuzzer
  afl         → workflow preset: fuzz-build-afl
  hongfuzz    → workflow preset: fuzz-build-hongfuzz
  native      → workflow preset: fuzz-build-native

Examples:
  ./fuzz.sh build
  ./fuzz.sh build afl
  ./fuzz.sh test
  ./fuzz.sh test libfuzzer 5
USAGE
}

is_engine() {
  local x="${1:-}"
  for e in "${ENGINES[@]}"; do [[ "$e" == "$x" ]] && return 0; done
  return 1
}

# -------- Paths / helpers --------

# Map engine → binary dir produced by your CMakePresets
bindir_for() {
  case "$1" in
    libfuzzer) echo "build-fuzz-libfuzzer/bin" ;;
    afl)       echo "build-afl/bin" ;;
    hongfuzz)  echo "build-hongfuzz/bin" ;;
    native)    echo "build-fuzz-bin/bin" ;;
    *)         return 1 ;;
  esac
}

# Find executables in the engine's bin dir
find_bins() {
  local engine="$1"
  local dir
  dir="$(bindir_for "$engine")"
  # Fallback if projects didn't set RUNTIME_OUTPUT_DIRECTORY
  if [[ ! -d "$dir" ]]; then
    dir="${dir%/bin}"
  fi
  if [[ -d "$dir" ]]; then
    find "$dir" -maxdepth 2 -type f -perm -111 2>/dev/null || true
  fi
}

ensure_seeds() {
  mkdir -p fuzz/seeds
  [[ -f fuzz/seeds/empty ]] || : > fuzz/seeds/empty
}

# -------- Build --------

build_engine() {
  local engine="$1"
  local preset="fuzz-build-$engine"
  echo "+ cmake --workflow --preset $preset"
  cmake --workflow --preset "$preset"
}

build_all() {
  build_engine libfuzzer
  build_engine afl
  build_engine hongfuzz
  build_engine native
}

# -------- Test (quick sanity) --------

test_libfuzzer() {
  local secs="${1:-10}"
  ensure_seeds
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    local corpus="build-fuzz-libfuzzer/corpus-$name"
    mkdir -p "$corpus"
    echo "+ [libfuzzer] $name for ${secs}s"
    "$bin" -max_total_time="$secs" -print_final_stats=1 "$corpus" fuzz/seeds || true
  done < <(find_bins libfuzzer)
}

test_afl() {
  local secs="${1:-10}"
  if ! command -v afl-fuzz >/dev/null 2>&1; then
    echo "!! afl-fuzz not found; skipping AFL++ test"
    return 0
  fi
  ensure_seeds
  local out="build-afl/out"
  mkdir -p "$out"
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    local work="$out/$name"
    mkdir -p "$work"
    echo "+ [AFL++] $name for ${secs}s"
    afl-fuzz -V "$secs" -i fuzz/seeds -o "$work" -- "$bin" @@ || true
  done < <(find_bins afl)
}

test_hongfuzz() {
  local secs="${1:-10}"
  if ! command -v hongfuzz >/dev/null 2>&1; then
    echo "!! hongfuzz not found; skipping hongfuzz test"
    return 0
  fi
  ensure_seeds
  local out="build-hongfuzz/out"
  mkdir -p "$out"
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    local work="$out/$name"
    mkdir -p "$work"
    echo "+ [hongfuzz] $name for ${secs}s"
    hongfuzz -i fuzz/seeds -o "$work" -t "$secs" -- "$bin" ___FILE___ || true
  done < <(find_bins hongfuzz)
}

test_native() {
  local secs="${1:-10}"
  ensure_seeds
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    echo "+ [native] smoke-test $name using seeds (timeout ${secs}s each)"
    for s in fuzz/seeds/*; do
      timeout -k 1 "${secs}"s bash -c "cat \"$s\" | \"$bin\"" || true
    done
  done < <(find_bins native)
}

test_all() {
  local secs="${1:-10}"
  test_libfuzzer "$secs"
  test_afl "$secs"
  test_hongfuzz "$secs"
  test_native "$secs"
  echo "Quick tests complete."
}

# -------- Main --------

cmd="${1:-}"; shift || true
case "$cmd" in
  build)
    engine="${1:-}"
    if [[ -z "$engine" ]]; then
      build_all
    else
      if ! is_engine "$engine"; then usage; exit 1; fi
      build_engine "$engine"
    fi
    ;;
  test)
    engine="${1:-}"
    secs="${2:-10}"
    if [[ -z "$engine" ]]; then
      test_all "$secs"
    else
      if ! is_engine "$engine"; then usage; exit 1; fi
      case "$engine" in
        libfuzzer) test_libfuzzer "$secs" ;;
        afl)       test_afl "$secs" ;;
        hongfuzz)  test_hongfuzz "$secs" ;;
        native)    test_native "$secs" ;;
      esac
    fi
    ;;
  -h|--help|"")
    usage
    ;;
  *)
    echo "Unknown command: $cmd"
    usage
    exit 1
    ;;
esac
