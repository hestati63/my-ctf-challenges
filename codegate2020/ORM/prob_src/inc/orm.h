#ifndef __ORM__
#define __ORM__
#include "queue.h"
#include "region.h"
#include <stdlib.h>

struct orm {
    u8 byte;
    u64 pc;
    u64 r;
    struct mm *mm;
    struct queue *q;

    enum { INVOP = 1, SEGFAULT, INVST, DEVERR } err;
};

union inst {
    struct {
        u8 arg2: 2; // F: 1, B: 2, R: 3, UNDEF:0
        u8 arg1: 1; // F: 0, B: 1
        u8 ops: 5; // 2 ** 5 = 32
    } d;
    u8 c;
};

typedef int (*handler)(struct orm *, u8 arg1, u8 arg2);

void run(struct orm *st);

#include "helper.h"
#endif
