#!/usr/bin/env bash
set -euo pipefail

# Engines we support
ENGINES=("libfuzzer" "afl" "honggfuzz" "standalone")

# Base "fuzz" directory
{{#if minimal}}
FUZZ_DIR="."
{{else}}
FUZZ_DIR="fuzz"
{{/if}}

TESTSUITE=${FUZZ_DIR}/testsuite
RESULTS=${FUZZ_DIR}/results


usage() {
  cat <<'USAGE'
Usage:
  ./fuzz.sh build [ENGINE]      # Build all engines (default) or just one
  ./fuzz.sh test  [ENGINE] [S]  # Quick sanity fuzz; S seconds (default 10)

Engines:
  libfuzzer   → Build with libFuzzer (requires clang++)
  afl         → Build with AFL++ (requires afl-clang-fast++)
  honggfuzz    → Build with HonggFuzz (requires hfuzz-clang++)
  standalone  → Build standalone binary (no fuzzing engine)

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

# Map engine → binary dir based on build system
bindir_for() {
  case "$1" in
{{#if (eq integration 'cmake')}}
    libfuzzer) echo "build/libfuzzer/bin" ;;
    afl)       echo "build/afl/bin" ;;
    honggfuzz)  echo "build/honggfuzz/bin" ;;
    standalone) echo "build/standalone/bin" ;;
{{else if (eq integration 'make')}}
    libfuzzer|afl|honggfuzz|standalone) echo "fuzz/build" ;;
{{/if}}
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


# -------- Build --------

build_engine() {
  local engine="$1"
  local log="${RESULTS}/${engine}-build.log"
{{#if (eq integration 'cmake')}}
  local preset="fuzz-$engine"
  printf "%-60s" "+ cmake --preset $preset"
  if cmake --preset "$preset" > $log 2>&1; then
      echo "[OK]"
      printf "%-60s" "+ cmake --build --preset $preset"
      if cmake --build --preset "$preset" >> $log 2>&1; then
          echo "[OK]"
      else
          echo "[FAIL]"
          cat $log
          echo "Failed to build fuzzer, the above log is stored at"
          echo $(realpath $log)
      fi
  else
      echo "[SKIP]"
      echo "- Note: engine $engine is not supported"
      echo "- Note: see `realpath $log` for details"
  fi
{{else if (eq integration 'make')}}
  printf "%-60s" "+ make fuzz-$engine"
  if make "fuzz-$engine" > $log 2>&1; then
      echo "[OK]"
  else
      echo "[FAIL]"
      cat $log
      echo "- Warning: Failed to build fuzzer, the above log is stored at"
      echo $(realpath $log)
  fi
{{/if}}
}

build_all() {
  build_engine libfuzzer
  build_engine afl
  build_engine honggfuzz
  build_engine standalone
}

# -------- Test (quick sanity) --------

test_libfuzzer() {
  local secs="${1:-10}"
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name="$(basename "$bin")"
    local output="${RESULTS}/$name"
    mkdir -p "$output"
    cp -r "$TESTSUITE"/* "$output"
    echo "+ [libfuzzer] $name for ${secs}s"
    "$bin" -max_total_time="$secs" -print_final_stats=1 "$output" || true
  done < <(find_bins libfuzzer)
}

test_afl() {
  local secs="${1:-10}"
  if ! command -v afl-fuzz >/dev/null 2>&1; then
    echo "!! afl-fuzz not found; skipping AFL++ test"
    return 0
  fi
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name="$(basename "$bin")"
    local work="$RESULTS/$name"
    mkdir -p "$work"
    echo "+ [AFL++] $name for ${secs}s"
    afl-fuzz -m none -V "$secs" -i "$TESTSUITE" -o "$work" -- "$bin" @@ || true
  done < <(find_bins afl)
}

test_honggfuzz() {
  local secs="${1:-10}"
  if ! command -v honggfuzz >/dev/null 2>&1; then
    echo "!! honggfuzz not found; skipping honggfuzz test"
    return 0
  fi
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name="$(basename "$bin")"
    local work="$RESULTS/$name"
    mkdir -p "$work"
    echo "+ [honggfuzz] $name for ${secs}s"
    timeout -k 1 $secs honggfuzz -i "$TESTSUITE" -o "$work" -- "$bin" ___FILE___ || true
  done < <(find_bins honggfuzz)
}

test_standalone() {
  local secs="${1:-10}"
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name="$(basename "$bin")"
    echo "+ [standalone] $name using testsuite (timeout ${secs}s each)"
    echo timeout -k 1 $secs "./$bin" "$TESTSUITE"
    timeout -k 1 $secs "./$bin" "$TESTSUITE"
  done < <(find_bins standalone)
}

test_all() {
  local secs="${1:-10}"
  test_libfuzzer "$secs"
  test_afl "$secs"
  test_honggfuzz "$secs"
  test_standalone "$secs"
  echo "Quick tests complete."
}

# -------- Main --------

mkdir -p ${TESTSUITE}
mkdir -p ${RESULTS}
printf 'A%.0s' {1..256} > ${TESTSUITE}/default

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
        honggfuzz)  test_honggfuzz "$secs" ;;
        standalone) test_standalone "$secs" ;;
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
