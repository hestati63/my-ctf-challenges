#include "region.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

struct mm *new_mm() {
    struct mm *mm = (struct mm *)malloc(sizeof(struct mm));
    if (mm) {
        mm->head = NULL;
    }
    return mm;
}

struct mem_slot *add_slot(struct mm *mm, u64 base, u64 len, u8 perm) {
    struct mem_slot *r = (struct mem_slot *)malloc(sizeof(struct mem_slot));
    if (r) {
        *r = (struct mem_slot) {
            .base = base,
            .len = len,
            .d = malloc(len),
            .perm = perm,
            .cb.cb_head = NULL,
            .cb.cb_tail = NULL,
            .aux = NULL,
            .next = mm->head,
        };
        if (r->d) {
            mm->head = r;
            memset(r->d, 0, len);
            goto RET;
        }
        free(r);
        r = NULL;
        goto RET;
    }
RET:
    return r;
}

struct mem_slot *find_slot(struct mm *mm, u64 addr) {
    // FIXME: LOCK
    for (struct mem_slot *r = mm->head; r; r = r->next) {
        if (addr >= r->base && addr < r->base + r->len) {
            return r;
        }
    }
    return NULL;
}
void register_slotcb(struct mm *mm, struct mem_slot *slot,
        struct mem_slot_callback *cb, void *aux) {
    // FIXME: LOCK
    slot->cb.cb_head = cb->cb_head;
    slot->cb.cb_tail = cb->cb_tail;
    slot->aux = aux;
}

void unregister_slotcb(struct mm *mm, struct mem_slot *slot) {
    // FIXME: LOCK
    slot->cb.cb_head = NULL;
    slot->cb.cb_tail = NULL;
    slot->aux = NULL;
}

u64 translate_addr(struct mm *mm, u64 addr, u64 *size, u8 perm) {
    struct mem_slot *r = find_slot(mm, addr);
    if (r && (r->perm & perm) == perm) {
        u64 off = addr - r->base;
        if (size) *size = r->len;
        return (u64) r->d + off;
    }
    return 0;
}

#define def_load(T, n) \
T load_##n(struct mm *mm, u64 addr, u8 perm, int *succ) { \
    struct mem_slot *r = find_slot(mm, addr); \
    T v = 0; \
    if (r && (r->perm & perm) == perm) { \
        u64 off = addr - r->base; \
        if (r->cb.cb_head) r->cb.cb_head(LOAD, addr, sizeof(T), r->aux); \
        v = *(T *)(r->d + off); \
        if (r->cb.cb_tail) r->cb.cb_tail(LOAD, addr, sizeof(T), r->aux); \
        *succ = 1; \
    } else { \
        *succ = 0; \
    } \
    return v; \
}

def_load(u8, byte)
def_load(u16, word)
def_load(u32, dword)
def_load(u64, qword)

#define def_store(T, n) \
int store_##n(struct mm *mm, u64 addr, u8 perm, T v) { \
    struct mem_slot *r = find_slot(mm, addr); \
    if (r && (r->perm & perm) == perm) { \
        u64 off = addr - r->base; \
        if (r->cb.cb_head) r->cb.cb_head(STORE, addr, sizeof(T), r->aux); \
        *(T *)(r->d + off) = v; \
        if (r->cb.cb_tail) {\
            r->cb.cb_tail(STORE, addr, sizeof(T), r->aux); \
        }\
        return 1; \
    } else { \
        return 0; \
    } \
}

def_store(u8, byte)
def_store(u16, word)
def_store(u32, dword)
def_store(u64, qword)
