/* Controlled invalid-byte creation refusal and real argv evidence (§AR-ci.10.6).
 * This Linux-only test seam targets the 0xff names used by the Rust fixtures. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static void record(const char *message) {
    const char *path = getenv("GRUND_TEST_NAME_TRACE");
    int (*real_open)(const char *, int, ...) = dlsym(RTLD_NEXT, "open");
    if (!path) _exit(90);
    int fd = real_open(path, O_WRONLY | O_APPEND);
    size_t size = strlen(message);
    if (fd < 0 || write(fd, message, size) != (ssize_t)size) _exit(91);
    close(fd);
}

__attribute__((constructor)) static void record_argv(void) {
    char args[16384], line[32776];
    int fd = open("/proc/self/cmdline", O_RDONLY);
    if (fd < 0) _exit(92);
    ssize_t size = read(fd, args, sizeof(args));
    close(fd);
    if (size <= 0 || size == sizeof(args)) _exit(93);
    memcpy(line, "exec:", 5);
    static const char hex[] = "0123456789abcdef";
    for (ssize_t i = 0; i < size; ++i) {
        unsigned char byte = (unsigned char)args[i];
        line[5 + i * 2] = hex[byte >> 4];
        line[6 + i * 2] = hex[byte & 15];
    }
    line[5 + size * 2] = '\n';
    line[6 + size * 2] = '\0';
    record(line);
}

static int intercepted(const char *symbol, const char *path, int flags, mode_t mode) {
    const char *refusal = getenv("GRUND_TEST_NAME_REFUSAL");
    int (*real_open)(const char *, int, ...) = dlsym(RTLD_NEXT, symbol);
    if ((flags & O_CREAT) && strchr(path, 0xff) && refusal) {
        if (strcmp(refusal, "spelling") == 0) {
            char *changed = strdup(path);
            if (!changed) _exit(94);
            for (unsigned char *p = (unsigned char *)changed; *p; ++p)
                if (*p == 0xff) *p = 'X';
            record("refused:spelling\n");
            int fd = real_open(changed, flags, mode);
            free(changed);
            return fd;
        }
        int permission = strcmp(refusal, "permission") == 0;
        record(permission ? "refused:permission\n" : "refused:encoding\n");
        errno = permission ? EACCES : EILSEQ;
        return -1;
    }
    return real_open(path, flags, mode);
}

int open(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap; va_start(ap, flags); mode = va_arg(ap, int); va_end(ap);
    }
    return intercepted("open", path, flags, mode);
}

int open64(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap; va_start(ap, flags); mode = va_arg(ap, int); va_end(ap);
    }
    return intercepted("open64", path, flags, mode);
}
