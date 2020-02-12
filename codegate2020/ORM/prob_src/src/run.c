#include "orm.h"
#include "queue.h"
#include <stdlib.h>
#include <stdio.h>

__attribute__ ((noreturn))
void unreachable() {
    printf("UNREACHABLE: FATAL ERR\n");
    exit(-2);
}

__attribute__((always_inline))
static __inline u32 a1_32(struct orm *st, u8 a1) {
    u32 r = 0;
    switch (a1) {
        case 0:
            r = pop_front_32(st->q);
            break;
        case 1:
            r = pop_back_32(st->q);
            break;
        default:
            unreachable();
    }
    return r;
}

__attribute__((always_inline))
static __inline u64 a1_64(struct orm *st, u8 a1) {
    u64 r = 0;
    switch (a1) {
        case 0:
            r = pop_front_64(st->q);
            break;
        case 1:
            r = pop_back_64(st->q);
            break;
        default:
            unreachable();
    }
    return r;
}

__attribute__((always_inline))
static __inline u32 a2_32(struct orm *st, u8 a2) {
    u32 r = 0;
    switch (a2) {
        case 0:
            st->err = -INVOP;
            return 0;
        case 1:
            r = pop_front_32(st->q);
            break;
        case 2:
            r = pop_back_32(st->q);
            break;
        case 3:
            r = st->r;
            break;
        default:
            unreachable();
    }
    return r;
}

__attribute__((always_inline))
static __inline u64 a2_64(struct orm *st, u8 a2) {
    u64 r = 0;
    switch (a2) {
        case 0:
            st->err = -INVOP;
            return 0;
        case 1:
            r = pop_front_64(st->q);
            break;
        case 2:
            r = pop_back_64(st->q);
            break;
        case 3:
            r = st->r;
            break;
        default:
            unreachable();
    }
    return r;
}


#define WORD u32
#define SWORD int32_t
#define A1 a1_32
#define A2 a2_32
#define fetch fetch32
#define pop_front pop_front_32
#define pop_back pop_back_32
#define push_front push_front_32
#define push_back push_back_32
#define DEFH(name) op32_##name(struct orm *s, u8 a1, u8 a2)
#define BIT32
#include "op.inc.c"
#undef WORD
#undef SWORD
#undef A1
#undef A2
#undef fetch
#undef pop_front
#undef pop_back
#undef push_front
#undef push_back
#undef DEFH
#undef BIT32

#define WORD u64
#define SWORD int64_t
#define A1 a1_64
#define A2 a2_64
#define fetch fetch64
#define pop_front pop_front_64
#define pop_back pop_back_64
#define push_front push_front_64
#define push_back push_back_64
#define DEFH(name) op64_##name(struct orm *s, u8 a1, u8 a2)
#define BIT64
#include "op.inc.c"
#undef WORD
#undef SWORD
#undef A1
#undef A2
#undef fetch
#undef pop_front
#undef pop_back
#undef push_front
#undef push_back
#undef DEFH
#undef BIT64
