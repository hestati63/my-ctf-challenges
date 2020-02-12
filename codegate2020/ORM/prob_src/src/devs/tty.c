#include "tty.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <immintrin.h>

int tty_realize(struct device *dev) {
    struct tty_aux *aux = malloc(sizeof(struct tty_aux));
    register_device(PORT_BASE + TRANS_CTRL, dev);
    register_device(PORT_BASE + TRANS_CTRL_AUX, dev);
    register_device(PORT_BASE + DEV_STATUS, dev);
    register_device(PORT_BASE + CHAR_IN, dev);
    register_device(PORT_BASE + CHAR_OUT, dev);
    register_device(PORT_BASE + ERR_REG, dev);
    aux->status = STATUS_OFF;
    aux->io_type = UNDEF;
    aux->err = 0;
    dev->aux = aux;
    return 0;
}

void slot_cb_tail(u8 type, u64 addr, u64 len, void *aux) {
    struct tty_aux *tty = (struct tty_aux *)aux;
    if ((addr & 0xfff) == 0x20) return;
    tty->io_aux.mmio.data_out = 0;
    tty->io_aux.mmio.data_in = 1 + ((addr & 0xfff) >> 3); // notify data in.
    if (type == LOAD) { // Wait immediate response.
        while (!tty->io_aux.mmio.data_out) _mm_pause();
        tty->io_aux.mmio.data_out = 0;
    } else {
        // Wait until locked.
        while (tty->io_aux.mmio.data_in) _mm_pause();
    }
}

struct mem_slot_callback cb = {
    .cb_head = NULL,
    .cb_tail = slot_cb_tail
};

struct mmio_read_lock {
    u64 start;
    u64 end;
    struct mem_slot *this_slot;
    struct mmio_status *mmio;
    struct mmio_read_lock *n;
};

void locked(u8 type, u64 addr, u64 len, void *aux) {
    struct mmio_read_lock *lock = (struct mmio_read_lock *)aux;
    // addr ~ addr + len - 1
    if (!(lock->start >= addr + len || addr >= lock->end)) { // overlap
        while (!lock->mmio->data_out) _mm_pause();
        lock->mmio->data_out = 0;
    }
}

void tty_handle_read(union mmio_req *req, struct mmio_status *mmio) {
    u32 size = req->in.read_aux;
    u64 seglen;
    u32 readbyte = 0;
    u32 idx = 0;
    char *buf;
    struct mmio_read_lock *head = NULL;
    struct mem_slot_callback locked_cb = {
        .cb_head = locked,
        .cb_tail = NULL,
    };
    req->out.err = STATUS_ONREAD;
    // 1. lock the regions
    while (idx < size) {
        struct mem_slot *slot = find_slot(mmio->st->mm, req->in.read + idx);
        if (!slot) break;

        u64 rem = slot->base + slot->len - (req->in.read + idx);
        u64 cur_slot_occupied = rem > size - idx ? size - idx : rem;
        idx += cur_slot_occupied;
        struct mmio_read_lock *aux =
            (struct mmio_read_lock *) malloc(sizeof(struct mmio_read_lock));
        *aux = (struct mmio_read_lock) {
            .start = req->in.read + idx,
            .end = req->in.read + idx + cur_slot_occupied,
            .this_slot = slot,
            .mmio = mmio,
            .n = head
        };
        register_slotcb(mmio->st->mm, slot, &locked_cb, aux);
        head = aux;
    }

    idx = 0;
    mmio->data_in = 0;

    while (idx < size) {
        // BUG: out.read can be changed.
        // NO PERMISSION CHECK
        // Misuse of translate_addr
        // Actual semantic: return the size of current segment.
        // Expected semantic: return the writable length of current segment.
        buf = (char *) translate_addr(mmio->st->mm,
                        req->in.read + idx, &seglen, 0);
        if (!buf) break;
        u64 onread = seglen < size ? seglen : size;
        readbyte = read(0, buf, onread);
        idx += readbyte;
        if (onread != readbyte) break;
    }

    req->out.out = idx;
    req->out.err = STATUS_WAITREQ;
    mmio->data_out = 1;

    struct mmio_read_lock *x = NULL;
    for (struct mmio_read_lock *l = head; l;) {
        unregister_slotcb(mmio->st->mm, l->this_slot);
        x = l;
        l = l->n;
        free(x);
    }
}

void *tty_job(void *arg) {
    struct tty_aux *tty = (struct tty_aux *)arg;
    struct mmio_status *mmio = &tty->io_aux.mmio;

    union mmio_req *req =
        (union mmio_req *) translate_addr(mmio->st->mm, mmio->mmio_base,
                                            NULL, PERM_R | PERM_W);
    if (!req) goto EXIT_JOB;

    while (!mmio->exit_req) {
        if (mmio->data_in) {
            switch (mmio->data_in) {
                case REQ_READ:
                    tty_handle_read(req, mmio);
                    break;
                case REQ_WRITE:
                    write(1, (void *)&req->in.write, 1);
                default:
                    mmio->data_in = 0;
                    mmio->data_out = 1;
                    break;
            }
        }
        _mm_pause(); // __spin
    }
EXIT_JOB:
    pthread_exit(NULL);
}

__attribute__((always_inline))
static __inline int handle_trans_ctrl(struct tty_aux *tty, u32 data) {
    if (tty->status != STATUS_OFF) tty->err = ERR_WAIT_STOP;
    else if (data == IO || data == MMIO) tty->io_type = data;
    else tty->err = ERR_INV_CONF;
    return 0;
}

__attribute__((always_inline))
static __inline int handle_trans_ctrl_aux(struct tty_aux *tty, u32 data) {
    if (tty->status == STATUS_OFF && tty->io_type == MMIO) {
        if ((data & 0xfff) != 0) tty->err = ERR_INV_CONF;
        else tty->io_aux.mmio.mmio_base = data;
    }
    return 0;
}

void mmio_cleanup(struct orm *s, struct tty_aux *tty) {
    if (tty->io_aux.mmio.tid) {
        pthread_join(tty->io_aux.mmio.tid, NULL);
        tty->io_aux.mmio.tid = 0;
        struct mem_slot *slot = find_slot(s->mm, tty->io_aux.mmio.mmio_base);
        if (slot) unregister_slotcb(s->mm, slot);
    }
}

void mmio_startup(struct orm *s, struct tty_aux *tty) {
    struct mem_slot *slot = find_slot(s->mm, tty->io_aux.mmio.mmio_base);
    if (!slot) tty->err = ERR_INV_CONF;
    tty->io_aux.mmio.st = s;
    register_slotcb(s->mm, slot, &cb, tty);

    if (pthread_create(&tty->io_aux.mmio.tid, NULL, tty_job, tty)) {
        tty->err = ERR_UNKNOWN;
    }
}

__attribute__((always_inline))
static __inline int handle_dev_status(struct orm *s, struct tty_aux *tty, u32 data) {
    if (tty->status != STATUS_OFF && data == STATUS_OFF) {
        tty->status = STATUS_OFF;
        if (tty->io_type == MMIO) mmio_cleanup(s, tty);
    } else if (tty->status != STATUS_ON && data == STATUS_ON) {
        if (tty->io_type == MMIO) {
            if (!tty->io_aux.mmio.mmio_base) tty->err = ERR_INV_CONF;
            else mmio_startup(s, tty);
        }
        if (!tty->err) tty->status = STATUS_ON;
    } else {
        tty->err = ERR_INV_CONF;
    }
    return 0;
}

__attribute__((always_inline))
static __inline int handle_char_out(struct tty_aux *tty, u32 data) {
    if (tty->status == STATUS_ON && tty->io_type == IO) putchar(data);
    else tty->err = ERR_INV_CONF;
    return 0;
}

int tty_in(struct orm *s, struct device *dev, u32 port, u32 data) {
    struct tty_aux *tty = (struct tty_aux *)dev->aux;
    tty->err = SUCCESS;
    switch (port) {
        case PORT_BASE + TRANS_CTRL:
            return handle_trans_ctrl(tty, data);
        case PORT_BASE + TRANS_CTRL_AUX:
            return handle_trans_ctrl_aux(tty, data);
        case PORT_BASE + DEV_STATUS:
            return handle_dev_status(s, tty, data);
        case PORT_BASE + CHAR_OUT:
            return handle_char_out(tty, data);
        default:
            return 1;
    }
}

int tty_out(struct orm *s, struct device *dev, u32 port, u32 data) {
    struct tty_aux *tty = (struct tty_aux *)dev->aux;
    tty->err = port == PORT_BASE + ERR_REG ? tty->err : SUCCESS;
    switch (port) {
        case PORT_BASE + TRANS_CTRL:
            switch (data) {
                case 0:
                    return tty->status;
                case 1:
                    return tty->io_type;
                case 2:
                    if (tty->io_type == MMIO) {
                        return tty->io_aux.mmio.mmio_base;
                    } else {
                        tty->err = ERR_INV_CONF;
                        return -1;
                    }
                default:
                    tty->err = ERR_INV_CONF;
                    return -1;
            }
        case PORT_BASE + ERR_REG:
            return tty->err;
        case PORT_BASE + CHAR_IN:
            if (tty->io_type == IO) {
                if (read(0, &tty->io_aux.io_buf, 1) != 1) {
                    fprintf(stderr, "device panicked.");
                    exit(-1);
                } else {
                    return tty->io_aux.io_buf;
                }
            } else {
                tty->err = ERR_INV_CONF;
                return -1;
            }
        default:
            return -1;
    }
    return 0;
}
