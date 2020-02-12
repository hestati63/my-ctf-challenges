#include "helper.h"

#define def_fetch(T) \
u##T fetch##T(struct orm *s) { \
    u##T *v= (u##T *)translate_addr(s->mm, s->pc, NULL, PERM_E); \
    if (v) { \
        s->pc += sizeof(u##T); \
        return *v;\
    } \
    s->err = -SEGFAULT; \
    return 0; \
}

def_fetch(8)
def_fetch(16)
def_fetch(32)
def_fetch(64)
#undef def_fetch
