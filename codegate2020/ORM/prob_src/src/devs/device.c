#include "device.h"
#include "fs.h"
#include "tty.h"

#define __DEV(realf, inpf, outf, T) \
    { .io = { .in = (inpf), \
              .out = (outf) }, \
      .realize = (realf), \
      .device_type = T }

#define RW_DEV(realf, inpf, outf) __DEV(realf, inpf, outf, RW)
#define R_DEV(realf, inpf) __DEV(realf, inpf, NULL, R)
#define W_DEV(realf, outf) __DEV(realf, NULL, outf, W)

struct device devices[] =
{
    RW_DEV(tty_realize, tty_in, tty_out),
};

struct forward_info {
    struct forward_info *next;
    u32 port;
    struct device *dev;
};

struct forward_info *info_head;

__attribute__((constructor))
void __device_init(void) {
    for (int i = 0; i < sizeof(devices) / sizeof(struct device); i++) {
        struct device *dev = &devices[i];
        if (dev->realize(dev)) {
            fprintf(stderr, "fail to initialize the device.\n");
            exit(-1);
        }
    }
}

__attribute__((always_inline))
static __inline struct device *find_device_by_port(u32 port) {
    for (struct forward_info *info = info_head;
            info;
            info = info->next) {
        if (info->port == port) return info->dev;
    }
    return NULL;
}

int register_device(u32 port, struct device *dev) {
    for (struct forward_info *info = info_head;
            info;
            info = info->next) {
        if (info->port == port) return 1; // fail to register
    }
    struct forward_info *thisinfo =
        (struct forward_info *)malloc(sizeof(struct forward_info));
    *thisinfo = (struct forward_info) {
        .next = info_head,
        .port = port,
        .dev = dev,
    };
    info_head = thisinfo;
    return 0;
}

int device_handle_in(struct orm *s, u32 port, u32 data) {
    struct device *dev = find_device_by_port(port);
    if (dev && ((dev->device_type & W) == W)) {
        return dev->io.in(s, dev, port, data);
    }
    return 1; // fail
}

int device_handle_out(struct orm *s, u32 port, u32 data, u32 *out) {
    struct device *dev = find_device_by_port(port);
    if (dev && ((dev->device_type & R) == R)) {
        *out = dev->io.out(s, dev, port, data);
        return 0;
    }
    return 1; // fail
}
