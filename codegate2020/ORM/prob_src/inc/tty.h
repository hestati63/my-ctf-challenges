#ifndef __TTY_H__
#define __TTY_H__
#include "device.h"
#include <pthread.h>

struct mmio_status {
    u32 mmio_base;
    pthread_t tid;
    struct orm *st;
    u8 exit_req: 1;
    u8 data_out: 1;
    u8 data_in: 6;
};

#define REQ_WRITE 1
#define REQ_READ 2
#define REQ_READ_AUX 3
#define REQ_ERR 4

#define STATUS_WAITREQ 0
#define STATUS_ONREAD  1

union mmio_req {
    struct in {
        u64 write;
        u64 read;
        u64 read_aux;
        u64 __unused1;
        u64 __unused2;
    } in;
    struct out {
        u64 __unused1;
        u64 __unused2;
        u64 __unused3;
        u64 out;
        u64 err;
    } out;
} __attribute__((aligned(0x1000), packed));

struct tty_aux {
    enum {STATUS_OFF, STATUS_ON} status;
    enum {UNDEF, IO, MMIO} io_type;
    u8 err;
    union {
        struct mmio_status mmio;
        u8 io_buf;
    } io_aux;
};

#define SUCCESS 0
#define ERR_WAIT_STOP 1
#define ERR_INV_CONF 2
#define ERR_UNKNOWN 3

int tty_in(struct orm *, struct device *dev, u32 port, u32 data);
int tty_out(struct orm *, struct device *dev, u32 port, u32 data);
int tty_realize(struct device *dev);

#define PORT_BASE 0x4F8
#define TRANS_CTRL 0
#define TRANS_CTRL_AUX 1
#define DEV_STATUS 2
// MMIO_DISABLE
#define CHAR_IN 3
#define CHAR_OUT 4
#define ERR_REG 5

// TRANS_CTRL
#define TYPE_IO IO
#define TYPE_MMIO MMIO
// DEV_STATUS
#define TTY_CTRL_OFF 0
#define TTY_CTRL_ON 1
#endif
