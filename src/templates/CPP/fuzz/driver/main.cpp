// Single driver for AFL++/hongfuzz/native that invokes the universal harness:
//   extern "C" int LLVMFuzzerTestOneInput(const uint8_t*, size_t);
// Optional initializer:
//   extern "C" int LLVMFuzzerInitialize(int* argc, char*** argv);
//
// It mirrors the common afl_driver semantics while keeping sanitizer calls optional.

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdint>
#include <string>
#include <vector>
#include <string_view>
#include <sys/stat.h>

#if defined(_WIN32)

#include <io.h>
// makes windows low-level IO posix-compatible
#define open _open
#define read _read
#define stat _stat
#define close _close
#define O_RDONLY _O_RDONLY
#define S_IFMT _S_IFMT
#define S_IFDIR _S_IFDIR
#define S_IFREG _S_IFREG
#else

#include <unistd.h>
#include <dirent.h>
#include <fcntl.h>
#include <sys/stat.h>
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
static int is_dir(const char* p) {
  struct stat st;
  return stat(p, &st) == 0 && (st.st_mode & S_IFMT) == S_IFDIR;
}

static bool is_file(const char* p) {
  struct stat st;
  return stat(p, &st) == 0 && (st.st_mode & S_IFMT) == S_IFREG;
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

static uint8_t *read_data(int fd, size_t *len, size_t max_len) {
  uint8_t *ptr = NULL;
  *len = 0;
  while (*len <= max_len) {
    uint8_t buf[BUFSIZ];
    int n = read(fd, buf, sizeof(buf));
    if (n == -1) {
      std::perror("read");
      std::exit(1);
    }
    if (n == 0) {
      break;
    }

    n = std::min<int>(n, max_len - *len);
    *len += n;
    ptr = (uint8_t *)realloc(ptr, *len);
    if (ptr == NULL) {
      std::perror("realloc");
      std::exit(1);
    }
    std::memcpy(ptr + *len - n, buf, n);
  }
  return ptr;
}

static uint8_t *read_file(const char* path, size_t *len, size_t max_len) {
  int fd = open(path, O_RDONLY);
  if (fd == -1) {
    fprintf(stderr, "can't open file %s: %s\n", path, strerror(errno));
    return NULL;
  }
  uint8_t *data = read_data(fd, len, max_len);
  close(fd);
  return data;
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

  size_t len;
  // No inputs? Read stdin once.
  if (files.empty()) {
    uint8_t *data = read_data(0, &len, max_len);
    LLVMFuzzerTestOneInput(data, len);
    free(data);
    return 0;
  }

  int executed = 0;
  for (const auto& f : files) {
    if (runs >= 0 && executed >= runs) {
      break;
    }
    uint8_t *data = read_file(f.c_str(), &len, max_len);
    if (data == NULL) {
      continue;
    }
    LLVMFuzzerTestOneInput(data, len);
    free(data);
    ++executed;
  }

  return 0;
}
