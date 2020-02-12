#include "orm.h"
#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>

struct orm_hdr {
    u32 magic;
    u32 byte;
    u64 entry;
    u32 q_size;
    u32 seg_cnt;
};

struct orm_seg64 {
    u64 base;
    u64 len_in_mem;
    u32 len_in_bin;
    u32 perm;
};

struct orm_seg32 {
    u32 base;
    u32 len_in_mem;
    u32 len_in_bin;
    u32 perm;
};

__attribute__((always_inline))
static void __inline read_force(int fd, void *buf, int size) {
    if (read(fd, buf, size) != size) {
        fprintf(stderr, "read error\n");
        exit(-1);
    }
}

__attribute__((noreturn))
void load_err() {
    fprintf(stderr, "Invalid binary.\n");
    exit(-1);
}

__attribute__((always_inline))
static int __inline validate_segment64(struct orm_seg64 *seg) {
    if (seg->base + seg->len_in_mem < seg->base) return 0;
    if (seg->len_in_mem < seg->len_in_bin) return 0;
    if ((seg->perm & 0x7) != seg->perm) return 0;
    if ((seg->len_in_mem & 0xfff) != 0) return 0;
    if ((seg->base & 0xfff) != 0) return 0;
    return 1;
}

__attribute__((always_inline))
static int __inline validate_segment32(struct orm_seg32 *seg) {
    if (seg->base + seg->len_in_mem < seg->base) return 0;
    if (seg->len_in_mem < seg->len_in_bin) return 0;
    if ((seg->perm & 0x7) != seg->perm) return 0;
    if ((seg->len_in_mem & 0xfff) != 0) return 0;
    if ((seg->base & 0xfff) != 0) return 0;
    return 1;
}

struct mm *load_segments64(int fd, int seg_cnt) {
    struct mm *mm = new_mm();
    struct orm_seg64 seg;

    for (int i = 0; i < seg_cnt; i++) {
        read_force(fd, &seg, sizeof(struct orm_seg64));
        if (!validate_segment64(&seg)) load_err();
        for (u64 addr = seg.base;
             addr < seg.base + seg.len_in_mem; addr += 0x1000) {
             if (find_slot(mm, addr)) load_err();
        }
        struct mem_slot *r =
            add_slot(mm, seg.base, seg.len_in_mem, seg.perm);
        if (r && r->d) read_force(fd, r->d, seg.len_in_bin);
        else load_err();
    }
    return mm;
}

struct mm *load_segments32(int fd, int seg_cnt) {
    struct mm *mm = new_mm();
    struct orm_seg32 seg;

    for (int i = 0; i < seg_cnt; i++) {
        read_force(fd, &seg, sizeof(struct orm_seg32));
        if (!validate_segment32(&seg)) load_err();
        for (u32 addr = seg.base;
             addr < seg.base + seg.len_in_mem; addr += 0x1000) {
             if (find_slot(mm, addr)) load_err();
        }
        struct mem_slot *r =
            add_slot(mm, seg.base, seg.len_in_mem, seg.perm);
        if (r && r->d) read_force(fd, r->d, seg.len_in_bin);
        else load_err();
    }
    return mm;
}



void load_binary(char *bin, struct orm *s) {
    int fd, urand;
    struct orm_hdr hdr;

    if ((fd = open(bin, O_RDONLY)) < 0) {
        fprintf(stderr, "No such file or directory: %s\n", bin);
        exit(-1);
    }

    read_force(fd, &hdr, sizeof(struct orm_hdr));
    if (hdr.magic != 0x4d4d524f ||
            (hdr.byte != 4 && hdr.byte != 8) ||
            (hdr.q_size & 0xfff) != 0 ||
            hdr.q_size < 0x1000) load_err();
    *s = (struct orm) {
        .byte = hdr.byte,
        .pc = hdr.entry,
        .r = 0,
        .mm = hdr.byte == 4 ?
                load_segments32(fd, hdr.seg_cnt) :
                load_segments64(fd, hdr.seg_cnt),
        .q = NULL,
        .err = 0,
    };

    if ((urand = open("/dev/urandom", O_RDONLY)) < 0) {
        fprintf(stderr, "No such file or directory: /dev/urandom\n");
        exit(-1);
    }
    u64 entropy;
    read_force(urand, &entropy, sizeof(entropy));
    close(urand);
    entropy = s->byte == 4 ? (entropy & 0xfffff000) : (entropy & (~0xfff));

    struct mem_slot *r =
        add_slot(s->mm, entropy, hdr.q_size, PERM_W | PERM_R | QUEUE);

    if (r) s->q = new_queue(hdr.q_size, hdr.byte, r);
    else load_err();
    close(fd);
}
