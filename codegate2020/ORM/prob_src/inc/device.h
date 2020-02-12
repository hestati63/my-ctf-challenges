#ifndef __DEV__H_
#define __DEV__H_
#include "orm.h"

struct device;

struct io {
    int (*in)(struct orm *, struct device *, u32 port, u32 data);
    int (*out)(struct orm *, struct device *, u32 port, u32 data);
};

struct device {
    struct io io;
    int (*realize)(struct device *);
    enum {R = 0b1, W = 0b10, RW = 0b11} device_type;
    void *aux;
};

int register_device(u32 port, struct device *dev);
int device_handle_in(struct orm *, u32 port, u32 data);
int device_handle_out(struct orm *, u32 port, u32 data, u32 *out);
#endif
