// Single driver for AFL++/hongfuzz/native that invokes the universal harness:
//   extern "C" int LLVMFuzzerTestOneInput(const uint8_t*, size_t);
// Optional initializer:
//   extern "C" int LLVMFuzzerInitialize(int* argc, char*** argv);
//
// It mirrors the common afl_driver semantics while keeping sanitizer calls optional.

#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <string_view>
#include <sys/stat.h>

#if defined(_WIN32)
  #include <io.h>
  #define FUZZ_FILENO _fileno
  #define FUZZ_DUP2   _dup2
  #define FUZZ_OPEN   _open
  #define FUZZ_O_CREAT _O_CREAT
  #define FUZZ_O_TRUNC _O_TRUNC
  #define FUZZ_O_WRONLY _O_WRONLY
  #define FUZZ_S_IRUSR _S_IREAD
  #define FUZZ_S_IWUSR _S_IWRITE
#else
  #include <unistd.h>
  #include <dirent.h>
  #include <fcntl.h>
  #define FUZZ_FILENO fileno
  #define FUZZ_DUP2   dup2
  #define FUZZ_OPEN   open
  #define FUZZ_O_CREAT O_CREAT
  #define FUZZ_O_TRUNC O_TRUNC
  #define FUZZ_O_WRONLY O_WRONLY
  #define FUZZ_S_IRUSR 0400
  #define FUZZ_S_IWUSR 0200
#endif

extern "C" int LLVMFuzzerTestOneInput(const uint8_t* data, size_t size);
extern "C" int LLVMFuzzerInitialize(int* argc, char*** argv) __attribute__((weak));

// ---------- Detect sanitizers (for optional hooks) ----------
#ifndef FUZZ_HAS_SANITIZER
#  if defined(__has_feature)
#    if __has_feature(address_sanitizer) || __has_feature(undefined_behavior_sanitizer) || __has_feature(thread_sanitizer)
#      define FUZZ_HAS_SANITIZER 1
#    endif
#  endif
#  if !defined(FUZZ_HAS_SANITIZER)
#    if defined(__SANITIZE_ADDRESS__) || defined(__SANITIZE_UNDEFINED__) || defined(__SANITIZE_THREAD__)
#      define FUZZ_HAS_SANITIZER 1
#    endif
#  endif
#endif

#if FUZZ_HAS_SANITIZER
#  include <sanitizer/common_interface_defs.h>
// Keep sanitizer symbols optional so native link won’t fail.
extern "C" void __sanitizer_set_report_fd(void*) __attribute__((weak));
extern "C" void __sanitizer_set_report_path(const char*) __attribute__((weak));
extern "C" void __sanitizer_set_death_callback(void (*)()) __attribute__((weak));
#endif

// -------------------- Utils --------------------
static bool is_dir(const char* p) {
#if defined(_WIN32)
  struct _stat st; return _stat(p, &st) == 0 && (st.st_mode & _S_IFDIR);
#else
  struct stat st; return stat(p, &st) == 0 && S_ISDIR(st.st_mode);
#endif
}

static bool is_file(const char* p) {
#if defined(_WIN32)
  struct _stat st; return _stat(p, &st) == 0 && (st.st_mode & _S_IFREG);
#else
  struct stat st; return stat(p, &st) == 0 && S_ISREG(st.st_mode);
#endif
}

static void list_files_in_dir(const char* dir, std::vector<std::string>& out) {
#if defined(_WIN32)
  // Minimal Windows handling omitted for brevity; native Linux is the common case.
  (void)dir; (void)out;
#else
  DIR* d = opendir(dir);
  if (!d) return;
  while (auto* ent = readdir(d)) {
    if (ent->d_name[0] == '.') continue;
    std::string path = std::string(dir) + "/" + ent->d_name;
    if (is_file(path.c_str())) out.push_back(path);
  }
  closedir(d);
#endif
}

static bool read_all_stream(FILE* f, std::vector<uint8_t>& buf, size_t max_len) {
  unsigned char tmp[4096]; size_t n;
  while ((n = std::fread(tmp, 1, sizeof(tmp), f)) > 0) {
    if (buf.size() + n > max_len) n = max_len - buf.size();
    buf.insert(buf.end(), tmp, tmp + n);
    if (buf.size() >= max_len) break;
  }
  return std::ferror(f) == 0;
}

static bool read_all_file(const char* path, std::vector<uint8_t>& buf, size_t max_len) {
  FILE* f = std::fopen(path, "rb");
  if (!f) return false;
  bool ok = read_all_stream(f, buf, max_len);
  std::fclose(f);
  return ok;
}

// Optional: ensure sanitizer reports get flushed
#if FUZZ_HAS_SANITIZER
static void on_sanitizer_death() { std::fflush(nullptr); }
#endif

// -------------------- Driver --------------------
int main(int argc, char** argv) {
  // Parse simple flags we support (like afl_driver):
  //   -runs=N     limit number of testcase invocations
  // Everything else is treated as a path (file or directory).
  int runs = -1;
  std::vector<std::string> paths;
  for (int i = 1; i < argc; ++i) {
    if (std::strncmp(argv[i], "-runs=", 6) == 0) {
      runs = std::atoi(argv[i] + 6);
    } else {
      paths.emplace_back(argv[i]);
    }
  }

  // Environment knobs
  size_t max_len = 1 << 20; // 1 MiB default
  if (const char* ml = std::getenv("AFL_DRIVER_MAX_LEN")) {
    long v = std::strtol(ml, nullptr, 10);
    if (v > 0) max_len = static_cast<size_t>(v);
  }

#if FUZZ_HAS_SANITIZER
  // Duplicate sanitizer reports to file if requested (compatible with afl_driver)
  if (const char* dup = std::getenv("AFL_DRIVER_STDERR_DUPLICATE_FILENAME")) {
    if (__sanitizer_set_report_path) __sanitizer_set_report_path(dup);
  }
  if (__sanitizer_set_death_callback) __sanitizer_set_death_callback(&on_sanitizer_death);
  // Prefer fd routing if available
  if (__sanitizer_set_report_fd) {
    __sanitizer_set_report_fd(reinterpret_cast<void*>(FUZZ_FILENO(stderr)));
  }
#endif

  // Allow user harness init
  if (LLVMFuzzerInitialize) {
    (void)LLVMFuzzerInitialize(&argc, &argv);
  }

  // Build list of inputs: files from args (expanding directories), or stdin if none.
  std::vector<std::string> files;
  for (const auto& p : paths) {
    if (p == "@@" || p == "___FILE___") continue; // wrappers should substitute these
    if (is_dir(p.c_str())) list_files_in_dir(p.c_str(), files);
    else if (is_file(p.c_str())) files.push_back(p);
  }

  // No inputs? Read stdin once.
  if (files.empty()) {
    std::vector<uint8_t> data;
    if (read_all_stream(stdin, data, max_len)) {
      LLVMFuzzerTestOneInput(data.data(), data.size());
    }
    return 0;
  }

  int executed = 0;
  for (const auto& f : files) {
    if (runs >= 0 && executed >= runs) break;
    std::vector<uint8_t> data;
    if (!read_all_file(f.c_str(), data, max_len)) continue;
    LLVMFuzzerTestOneInput(data.data(), data.size());
    ++executed;
  }

  return 0;
}
