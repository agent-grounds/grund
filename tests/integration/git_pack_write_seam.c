/* Schedule a real detached pack writer across strict fixture cleanup.
 * §AR-ci.10.4 */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static void exchange(int socket_fd, const char *message) {
    char answer;
    if (write(socket_fd, message, strlen(message)) != (ssize_t)strlen(message) ||
        read(socket_fd, &answer, 1) != 1)
        _exit(92);
}

static int intercepted(const char *symbol, const char *path, int flags, mode_t mode) {
    int (*real_open)(const char *, int, ...) = dlsym(RTLD_NEXT, symbol);
    const char *endpoint = getenv("GRUND_TEST_PACK_SOCKET");
    const char *session = getenv("GRUND_TEST_SESSION");
    int socket_fd = -1;
    if (endpoint && session && getsid(0) != atol(session) &&
        (flags & O_CREAT) && strstr(path, "/pack/tmp_pack_")) {
        struct sockaddr_un address = { .sun_family = AF_UNIX };
        if (strlen(endpoint) >= sizeof(address.sun_path)) _exit(93);
        strcpy(address.sun_path, endpoint);
        socket_fd = socket(AF_UNIX, SOCK_STREAM, 0);
        if (socket_fd < 0 || connect(socket_fd, (struct sockaddr *)&address,
                                    sizeof(address)) < 0)
            _exit(94);
        char ready[4096];
        snprintf(ready, sizeof(ready), "%ld\n%s\n", (long)getpid(), path);
        exchange(socket_fd, ready);
    }
    int fd = real_open(path, flags, mode);
    if (socket_fd >= 0) {
        if (fd < 0) _exit(95);
        exchange(socket_fd, "C");
        close(socket_fd);
    }
    return fd;
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
