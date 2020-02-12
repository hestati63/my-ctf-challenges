#include "orm.h"
#include "loader.h"
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>

//#define __DEBUG__

void init() {
    setvbuf(stdin, 0, 2, 0);
    setvbuf(stdout, 0, 2, 0);
    setvbuf(stderr, 0, 2, 0);
}

void usage(char *fname) {
    fprintf(stderr, "%s <binary>\n", fname);
    exit(-1);
}

extern handler handlers32[];
extern handler handlers64[];

int single_step(struct orm *st) {
    union inst inst = {
        .c = fetch8(st),
    };
#ifdef __DEBUG__
    fprintf(stderr, "d: %02x | op: %x %x %x\n", inst.c, inst.d.ops, inst.d.arg1, inst.d.arg2);
#endif
    switch (st->byte) {
        case 4:
            return handlers32[inst.d.ops](st, inst.d.arg1, inst.d.arg2);
        case 8:
            return handlers64[inst.d.ops](st, inst.d.arg1, inst.d.arg2);
        default:
            st->err = -INVST;
            return 1;
    }
}

#ifdef __DEBUG__
void debug(struct orm *st) {
    fprintf(stderr, "==== Registers ====\n");
    fprintf(stderr, "pc: 0x%016lx\tr: 0x%016lx\tf:0x%016x\tb:0x%016x\n",
            st->pc, st->r,
                st->q->front * st->q->c + st->q->this_slot->base,
                st->q->back * st->q->c + st->q->this_slot->base);
    fprintf(stderr, "queue: \n");
    for (int i = 0; i < 5; i++) {
        u64 idx = (st->q->front + i) % CAP(st->q);
        fprintf(stderr, "\t%d(%d): %llx\n", i, idx, st->q->d.d64[idx]);
        if (idx == st->q->back) break;
    }
    fprintf(stderr, "...\n");
    for (int i = 0; i < 5; i++) {
        u64 idx = (st->q->back - 4 + i) % CAP(st->q);
        fprintf(stderr, "\t%d(%d): %llx\n", -5+i, idx, st->q->d.d64[idx]);
        if (idx == st->q->front) break;
    }

}
#else
__attribute__((always_inline))
static __inline void debug(struct orm *st) {
}
#endif

void run(struct orm *st) {
    while (debug(st), !single_step(st));

    switch (-st->err) {
        case 0:
            fprintf(stderr, "ORM halted.\n");
            exit(0);
        case INVOP:
            fprintf(stderr, "Invalid opcode.\n");
            goto err_exit;
        case SEGFAULT:
            fprintf(stderr, "Segmentation fault.\n");
            goto err_exit;
        case INVST:
            fprintf(stderr, "Invalid State.\n");
            goto err_exit;
        case DEVERR:
            fprintf(stderr, "Device error.\n");
            goto err_exit;
    }
err_exit:
    exit(-1);
}

int main(int argc, char **argv) {
    struct orm st;
    if (argc < 2) usage(argv[0]);

    init();

    load_binary(argv[1], &st);

    if (!st.q) {
        fprintf(stderr, "fatal: fail to initialize ORM.\n");
        exit(-1);
    }

    run(&st);
    return 0;
}
