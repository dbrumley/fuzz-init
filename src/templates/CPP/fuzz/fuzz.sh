#!/usr/bin/env bash
set -euo pipefail

ENGINES=("bin" "libfuzzer" "afl" "hongfuzz")

usage() {
  cat <<'USAGE'
Usage:
  fuzz.sh build [ENGINE]      # Build one engine, or all if ENGINE omitted
  fuzz.sh test  [ENGINE] [S]  # Quick sanity fuzz (defaults: all engines, S=10 seconds)

Engines:
  bin        Build plain binaries with your default compiler (no sanitizers) -- for Mayhem/external fuzzers
  libfuzzer  Build with clang/++ and -fsanitize=fuzzer (ASan/UBSan optional)
  afl        Build with AFL++ (afl-clang-fast/++)
  hongfuzz   Build with Hongfuzz (hfuzz-clang/++)

Examples:
  ./fuzz.sh build
  ./fuzz.sh build afl
  ./fuzz.sh test libfuzzer 5
USAGE
}

is_engine() {
  local x="$1"
  for e in "${ENGINES[@]}"; do [[ "$e" == "$x" ]] && return 0; done
  return 1
}

cmake_build() {
  local engine="$1"
  local preset="fuzz-$engine"
  echo "+ cmake --preset $preset"
  cmake --preset "$preset"
  echo "+ cmake --build --preset $preset"
  cmake --build --preset "$preset"
}

find_bins() {
  local engine="$1"
  local bdir="build-fuzz-$engine"
  [[ "$engine" == "afl" ]] && bdir="build-afl"
  [[ "$engine" == "hongfuzz" ]] && bdir="build-hongfuzz"
  [[ "$engine" == "libfuzzer" ]] && bdir="build-fuzz-libfuzzer"
  [[ "$engine" == "bin" ]] && bdir="build-fuzz-bin"
  local bindir="$bdir/bin"
  if [[ ! -d "$bindir" ]]; then
    bindir="$bdir"
  fi
  find "$bindir" -maxdepth 2 -type f -perm -111 2>/dev/null || true
}

ensure_seeds() {
  mkdir -p fuzz/seeds
  [[ -f fuzz/seeds/empty ]] || : > fuzz/seeds/empty
}

test_libfuzzer() {
  local secs="${1:-10}"
  ensure_seeds
  local outdir="build-fuzz-libfuzzer/out"
  mkdir -p "$outdir"
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    local corpus="build-fuzz-libfuzzer/corpus-$name"
    mkdir -p "$corpus"
    echo "+ [libfuzzer] $name for ${secs}s"
    "$bin" -max_total_time="$secs" -print_final_stats=1 "$corpus" fuzz/seeds || true
  done < <(find_bins libfuzzer)
}

test_bin() {
  local secs="${1:-10}"
  ensure_seeds
  while IFS= read -r bin; do
    [[ -z "$bin" ]] && continue
    local name; name="$(basename "$bin")"
    echo "+ [bin] smoke-test $name using seeds (timeout ${secs}s)"
    for s in fuzz/seeds/*; do
      timeout -k 1 "${secs}"s bash -c "cat \"$s\" | \"$bin\"" || true
    done
  done < <(find_bins bin)
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

build_all() {
  cmake_build bin
  cmake_build libfuzzer
  if command -v afl-clang-fast++ >/dev/null 2>&1; then
    cmake_build afl
  else
    echo "AFL++ wrappers not found; skipping AFL++ build."
  fi
  if command -v hfuzz-clang++ >/dev/null 2>&1; then
    cmake_build hongfuzz
  else
    echo "hongfuzz wrappers not found; skipping hongfuzz build."
  fi
}

test_all() {
  local secs="${1:-10}"
  test_bin "$secs"
  test_libfuzzer "$secs"
  test_afl "$secs"
  test_hongfuzz "$secs"
  echo "Quick tests complete."
}

main() {
  local cmd="${1:-}"
  case "$cmd" in
    build)
      local engine="${2:-}"
      if [[ -z "$engine" ]]; then
        build_all
      else
        if ! is_engine "$engine"; then echo "Unknown engine: $engine"; usage; exit 1; fi
        cmake_build "$engine"
      fi
      ;;
    test)
      local engine="${2:-}"
      local secs="${3:-10}"
      if [[ -z "$engine" ]]; then
        test_all "$secs"
      else
        if ! is_engine "$engine"; then echo "Unknown engine: $engine"; usage; exit 1; fi
        case "$engine" in
          bin)       test_bin "$secs" ;;
          libfuzzer) test_libfuzzer "$secs" ;;
          afl)       test_afl "$secs" ;;
          hongfuzz)  test_hongfuzz "$secs" ;;
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
}

main "$@"
