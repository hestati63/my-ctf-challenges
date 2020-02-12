#ifndef __QUEUE__
#define __QUEUE__

#include <stdlib.h>
#include <stdio.h>

#include "region.h"

// double endeded queue.
typedef enum {BYTE4 = 4, BYTE8 = 8} conf;

struct queue {
    u32 front;
    u32 back;
    union {
        u32 *d32;
        u64 *d64;
    } d;
    struct mem_slot *this_slot;
    conf c;
};
#define CAP(q) ((q)->this_slot->len / (q)->c)

__attribute__((always_inline))
static __inline int is_full(struct queue *q) {
    return (q->front == 0 && q->back == CAP(q) - 1) ||
        q->front == q->back + 1;
}

__attribute__((always_inline))
static __inline int is_empty(struct queue *q) {
    return q->front == -1;
}
__attribute__((noreturn)) void queue_panic();

#define def_push_front(T) \
__attribute__((always_inline)) \
static __inline int push_front_##T(struct queue *q, u##T d) { \
    if (is_full(q)) return -1; \
    if (is_empty(q)) q->front = q->back = 0; \
    else if (q->front == 0) q->front = CAP(q) - 1; \
    else q->front = q->front - 1; \
    q->d.d##T[q->front] = d; \
    return 0; \
}

#define def_push_back(T) \
__attribute__((always_inline)) \
static __inline int push_back_##T(struct queue *q, u##T d) { \
    if (is_full(q)) return -1; \
    if (is_empty(q)) q->front = q->back = 0; \
    else if (q->back == CAP(q) - 1) q->back = 0; \
    else q->back = q->back + 1; \
    q->d.d##T[q->back] = d; \
    return 0; \
}

#define def_pop_front(T) \
__attribute__((always_inline)) \
static __inline u##T pop_front_##T(struct queue *q) { \
    if (is_empty(q)) queue_panic(); \
    u##T r = q->d.d##T[q->front]; \
    if (q->front == q->back) q->front = q->back = -1; \
    else if (q->front == CAP(q) - 1) q->front = 0; \
    else q->front = q->front + 1; \
    return r; \
}

#define def_pop_back(T) \
__attribute__((always_inline)) \
static __inline u##T pop_back_##T(struct queue *q) { \
    if (is_empty(q)) queue_panic(); \
    u##T r = q->d.d##T[q->back]; \
    if (q->front == q->back) q->front = q->back = -1; \
    else if (q->back == 0) q->back = CAP(q) - 1; \
    else q->back = q->back - 1; \
    return r; \
}

#define def_front(T) \
__attribute__((always_inline)) \
static __inline u##T front_##T(struct queue *q) { \
    if (is_empty(q)) return -1; \
    return q->d.d##T[q->front]; \
}

#define def_back(T) \
__attribute__((always_inline)) \
static __inline u##T back_##T(struct queue *q) { \
    if (is_empty(q)) return -1; \
    return q->d.d##T[q->back]; \
}

#define def_per_conf(T) \
    def_push_front(T) \
    def_pop_front(T) \
    def_push_back(T) \
    def_pop_back(T) \
    def_front(T) \
    def_back(T)

def_per_conf(32)
def_per_conf(64)

#undef def_push_front
#undef def_pop_front
#undef def_push_back
#undef def_pop_back
#undef def_per_conf
struct queue *new_queue(u64 size, conf c, struct mem_slot *slot);
#endif
