#include "queue.h"
#include <stdlib.h>
#include <stdio.h>

__attribute__((noreturn))
void queue_panic() {
    fprintf(stderr, "panicked.");
    exit(-1);
}

struct queue *new_queue(u64 size, conf c, struct mem_slot *slot) {
    if (c == BYTE4 || c == BYTE8) {
        struct queue *q = (struct queue *) malloc(sizeof(struct queue));
        if (!q) return NULL;

        *q = (struct queue) {
            .front = -1,
            .back = 0,
            .d.d32 = (u32 *)slot->d,
            .this_slot = slot,
            .c = c,
        };
        return q;
    }
    return NULL;
}
