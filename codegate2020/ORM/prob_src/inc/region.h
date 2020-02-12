#ifndef __REGION__
#define __REGION__

#include <stdint.h>
typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

#define BIT(x) (1<<(x))
#define PERM_R BIT(0)
#define PERM_W BIT(1)
#define PERM_E BIT(2)
#define QUEUE  BIT(3)

#define LOAD 0
#define STORE 1

struct mem_slot {
    u64 base;
    u64 len;
    u8 perm;
    struct mem_slot_callback {
        void (*cb_head)(u8 type, u64 addr, u64 len, void *aux);
        void (*cb_tail)(u8 type, u64 addr, u64 len, void *aux);
    } cb;
    void *aux;
    struct mem_slot *next;

    char *d; // the data
};


struct mm {
    struct mem_slot *head;
    // LOCK
};

struct mem_slot *add_slot(struct mm *mm, u64 bas, u64 len, u8 perm);
struct mm *new_mm();
struct mem_slot *find_slot(struct mm *mm, u64 addr);
u64 translate_addr(struct mm *mm, u64 addr, u64 *size, u8 perm);

u8 load_byte(struct mm *mm, u64 addr, u8 perm, int *succ);
u16 load_word(struct mm *mm, u64 addr, u8 perm, int *succ);
u32 load_dword(struct mm *mm, u64 addr, u8 perm, int *succ);
u64 load_qword(struct mm *mm, u64 addr, u8 perm, int *succ);

int store_byte(struct mm *mm, u64 addr, u8 perm, u8 v);
int store_word(struct mm *mm, u64 addr, u8 perm, u16 v);
int store_dword(struct mm *mm, u64 addr, u8 perm, u32 v);
int store_qword(struct mm *mm, u64 addr, u8 perm, u64 v);
void register_slotcb(struct mm *mm, struct mem_slot *slot,
        struct mem_slot_callback *cb, void *aux);
void unregister_slotcb(struct mm *mm, struct mem_slot *slot);

#endif
