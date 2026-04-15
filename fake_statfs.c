#define _GNU_SOURCE
#include <sys/statvfs.h>
#include <sys/vfs.h>
#include <dlfcn.h>
#include <string.h>
#include <stdio.h>

typedef int (*statfs_fn)(const char *, struct statfs *);
typedef int (*statvfs_fn)(const char *, struct statvfs *);

int statfs(const char *path, struct statfs *buf) {
    statfs_fn orig = (statfs_fn)dlsym(RTLD_NEXT, "statfs");
    int ret = orig(path, buf);
    if (ret == 0) {
        buf->f_bsize = 4096;
        buf->f_blocks = 10000000;
        buf->f_bfree = 8000000;
        buf->f_bavail = 8000000;
    }
    return ret;
}

int statvfs(const char *path, struct statvfs *buf) {
    statvfs_fn orig = (statvfs_fn)dlsym(RTLD_NEXT, "statvfs");
    int ret = orig(path, buf);
    if (ret == 0) {
        buf->f_bsize = 4096;
        buf->f_blocks = 10000000;
        buf->f_bfree = 8000000;
        buf->f_bavail = 8000000;
    }
    return ret;
}
