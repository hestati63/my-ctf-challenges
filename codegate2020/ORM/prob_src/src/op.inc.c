#ifndef WORD
#include "run.h"
#define WORD u32
#define SWORD int
WORD a1_32(struct state *s, u8 x) {
    return 0;
}
WORD a2_32(struct state *s, u8 x) {
    return 0;
}
#define A1 a1_32
#define A2 a2_32
#define fetch fetch32
#define pop_front pop_front_32
#define pop_back pop_back_32
#define push_front push_front_32
#define push_back push_back_32
#define DEFH(name) o_##name(struct state *s, u8 a1, u8 a2)
#define BIT32
#endif
#include "device.h"

int DEFH(inv) {
    s->err = -INVOP;
    return 1;
}

int DEFH(pop) {
    if (a2 != 0) {
        s->err = -INVOP;
        return 1;
    }
    switch (a1) {
        case 0: //popf
            s->r = pop_front(s->q);
            break;
        case 1:
            s->r = pop_back(s->q);
            break;
    }
    return 0;
}

int DEFH(neg_not) {
    switch (a2) {
        case 0:
            s->r = -A1(s, a1);
            return 0;
        case 1:
            s->r = ~A1(s, a1);
            return 0;
        default:
            s->err = -INVOP;
            return 1;
    }
}

int DEFH(add) {
    s->r = A1(s, a1) + A2(s, a2);
    return 0;
}

int DEFH(sub) {
    s->r = A1(s, a1) - A2(s, a2);
    return 0;
}

int DEFH(div) {
    s->r = A1(s, a1) / A2(s, a2);
    return 0;
}

int DEFH(mul) {
    s->r = A1(s, a1) * A2(s, a2);
    return 0;
}

int DEFH(mod) {
    s->r = A1(s, a1) % A2(s, a2);
    return 0;
}

int DEFH(shr) {
    s->r = A1(s, a1) >> A2(s, a2);
    return 0;
}

int DEFH(sar) {
    SWORD a = A1(s, a1);
    s->r = a >> A2(s, a2);
    return 0;
}

int DEFH(shl) {
    s->r = A1(s, a1) << A2(s, a2);
    return 0;
}

int DEFH(and) {
    s->r = A1(s, a1) & A2(s, a2);
    return 0;
}

int DEFH(or) {
    s->r = A1(s, a1) | A2(s, a2);
    return 0;
}

int DEFH(xor) {
    s->r = A1(s, a1) ^ A2(s, a2);
    return 0;
}

int DEFH(eq) {
    s->r = A1(s, a1) == A2(s, a2);
    return 0;
}

int DEFH(neq) {
    s->r = A1(s, a1) != A2(s, a2);
    return 0;
}

int DEFH(gt) {
    s->r = A1(s, a1) > A2(s, a2);
    return 0;
}

int DEFH(ge) {
    s->r = A1(s, a1) >= A2(s, a2);
    return 0;
}

int DEFH(sgt) {
    SWORD aa1 = A1(s, a1);
    SWORD aa2 = A2(s, a2);
    s->r = aa1 > aa2;
    return 0;
}

int DEFH(sge) {
    SWORD aa1 = A1(s, a1);
    SWORD aa2 = A2(s, a2);
    s->r = aa1 >= aa2;
    return 0;
}

int DEFH(j) {
    WORD branch;
    if (a1 != 0) {
        s->err = -INVOP;
        return 1;
    }
    switch (a2) {
        case 0:
            branch = fetch(s);
            if (s->err) return 1;
            break;
        default:
            branch = A2(s, a2);
            break;
    }
    s->pc = branch;
    return 0;
}

int DEFH(jz) {
    WORD branch;
    switch (a2) {
        case 0:
            branch = fetch(s);
            if (s->err) return 1;
            break;
        default:
            branch = A2(s, a2);
            break;
    }
    if (A1(s, a1) == 0) s->pc = branch;
    return 0;
}

int DEFH(jnz) {
    WORD branch;
    switch (a2) {
        case 0:
            branch = fetch(s);
            if (s->err) return 1;
            break;
        default:
            branch = A2(s, a2);
            break;
    }
    if (A1(s, a1) != 0) s->pc = branch;
    return 0;
}

int DEFH(push) {
    WORD r;
    switch (a2) {
        case 0: // const
            r = fetch(s);
            if (s->err) return 1;
            break;
        case 3: // register
            r = s->r;
            break;
        default:
            s->err = -INVOP;
            return 1;
    }
    switch (a1) {
        case 0: // pushf
            if (push_front(s->q, r) < 0) {
                s->err = -SEGFAULT;
                return 1;
            }
            break;
        case 1: // pushb
            if (push_back(s->q, r) < 0) {
                s->err = -SEGFAULT;
                return 1;
            }
            break;
    }
    return 0;
}

int DEFH(lar) {
    if (a2 != 0) {
        s->err = -INVOP;
        return 1;
    }
    switch (a1) {
        case 0: // front
            s->r = (WORD) s->q->this_slot->base +
                        (WORD) s->q->front * (WORD)s->q->c;
            break;
        case 1: // back
            s->r = (WORD) s->q->this_slot->base +
                        (WORD) s->q->back * (WORD)s->q->c;
            break;
    }
    return 0;
}

int DEFH(store) {
#ifdef BIT32
    if (a2 == 3) {
        s->err = -INVOP;
        return 1;
    }
#endif
    WORD r = a1 == 0 ? pop_front(s->q) : pop_back(s->q);
    WORD maddr = fetch(s);
    if (maddr & 0x7) {
        // NOT ALIGNED
        s->err = -INVOP;
        return 1;
    }

    int succ;

    switch (a2) {
        case 0:
            succ = store_byte(s->mm, maddr, PERM_W, r);
            break;
        case 1:
            succ = store_word(s->mm, maddr, PERM_W, r);
            break;
        case 2:
            succ = store_dword(s->mm, maddr, PERM_W, r);
            break;
        case 3:
            succ = store_qword(s->mm, maddr, PERM_W, r);
            break;
    }
    if (!succ) {
        s->err = -SEGFAULT;
        return 1;
    }
    return 0;
}

int DEFH(store2) {
#ifdef BIT32
    if (a2 == 3) {
        s->err = -INVOP;
        return 1;
    }
#endif
    WORD r = a1 == 0 ? pop_front(s->q) : pop_back(s->q);
    WORD maddr = s->r;
    if (maddr & 0x7) {
        // NOT ALIGNED
        s->err = -INVOP;
        return 1;
    }

    int succ;

    switch (a2) {
        case 0:
            succ = store_byte(s->mm, maddr, PERM_W, r);
            break;
        case 1:
            succ = store_word(s->mm, maddr, PERM_W, r);
            break;
        case 2:
            succ = store_dword(s->mm, maddr, PERM_W, r);
            break;
        case 3:
            succ = store_qword(s->mm, maddr, PERM_W, r);
            break;
    }
    if (!succ) {
        s->err = -SEGFAULT;
        return 1;
    }
    return 0;
}

int DEFH(load) {
#ifdef BIT32
    if (a2 == 3) {
        s->err = -INVOP;
        return 1;
    }
#endif
    WORD maddr = a1 == 0 ? pop_front(s->q) : pop_back(s->q);
    if (maddr & 0x7) {
        // NOT ALIGNED
        s->err = -SEGFAULT;
        return 1;
    }

    int succ;
    switch (a2) {
        case 0:
            s->r = load_byte(s->mm, maddr, PERM_R, &succ);
            break;
        case 1:
            s->r = load_word(s->mm, maddr, PERM_R, &succ);
            break;
        case 2:
            s->r = load_dword(s->mm, maddr, PERM_R, &succ);
            break;
        case 3:
            s->r = load_qword(s->mm, maddr, PERM_R, &succ);
            break;
    }
    if (!succ) {
        s->err = -SEGFAULT;
        return 1;
    }

    return 0;
}

int DEFH(io) {
    WORD port = A2(s, a2);
    u32 o;
    switch (a1) {
        case 0: // in
            if (device_handle_in(s, port, s->r)) {
                s->err = -DEVERR;
                return 1;
            }
            break;
        case 1: // out
            if (device_handle_out(s, port, s->r, &o)) {
                s->err = -DEVERR;
                return 1;
            }
            s->r = (WORD) o;
            break;
    }
    return 0;
}

int DEFH(call_ret) {
    WORD t;
    switch (a1) {
        case 0: // call
            switch (a2) {
                case 0:
                    t = fetch(s);
                    if (s->err) return 1;
                    break;
                default:
                    t = A2(s, a2);
                    break;
            }
            s->r = s->pc;
            s->pc = t;
            break;
        case 1: // ret
            s->pc = A2(s, a2);
            break;
    }
    return 0;
}

int DEFH(hlt) {
    return 1;
}

#ifdef BIT32
#define HANDLER handlers32
#define N(x) op32_##x
#else
#define HANDLER handlers64
#define N(x) op64_##x
#endif

handler HANDLER[] = {
    N(hlt),      /* 00: hlt */
    N(push),     /* 01: push front/back */
    N(pop),      /* 02: pop front/back */
    N(neg_not),  /* 03: neg/not */
    N(add),      /* 04: add */
    N(sub),      /* 05: sub */
    N(mul),      /* 06: mul */
    N(div),      /* 07: div */
    N(mod),      /* 08: modular */
    N(shr),      /* 09: shr */
    N(sar),      /* 10: sar */
    N(shl),      /* 11: shl */
    N(and),      /* 12: and */
    N(or),       /* 13: or */
    N(xor),      /* 14: xor */
    N(eq),       /* 15: eq */
    N(neq),      /* 16: neq */
    N(gt),       /* 17: gt */
    N(ge),       /* 18: ge */
    N(sgt),      /* 19: sgt */
    N(sge),      /* 20: sge */
    N(j),        /* 21: jump */
    N(jz),       /* 22: jz */
    N(jnz),      /* 23: jnz */
    N(lar),      /* 24: load address into register */
    N(io),       /* 25: in/out */
    N(call_ret), /* 26: call/ret */
    N(store),    /* 27: store */
    N(store2),   /* 28: reserved */
    N(load),     /* 29: reserved */
    N(inv),      /* 30: reserved */
    N(inv),      /* 31: reserved */
};
#undef N
#undef HANDLER
