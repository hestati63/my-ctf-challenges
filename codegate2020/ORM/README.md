# ORM-app (pwn) (810pts)

## Description

Exploit the app that is running on the brand-new cpu, ORM.

## Comments

By reverse engineering the binary, you can find that the machine has only one
register and double-ended queue rather than stack. Also, You can easily write
the disassembler for orm by finding the each opcode-instruction mappings.

In user program, there exist two bugs.
The first one is that index out of bound in project pointer array.
Its actual size is 8, but you can allocate more than 8.

The second one is buffer overflow in migrate menu.
In migrate menu, the program copies project's name into back portion of queue
with strcpy-like machanism, since the maximum length of the project's name is 7.
However, when moving the data into new project's name, it just pop one qword.

Since the memory to store project information is lay right after the project
pointer array, you can overwrite the project's name field. It leads buffer
overflow.

The remaining exploit technique is very trivial. You may refer the phase1.py.



# ORM (pwn) (1000pts)

## Description

Pwn the ORM.


## Comments

1. GET Arbitrary Code Execution in VM.
You can find that tty's read does not check the permission of memory segments.
As you overwrite the data of .text section, you can easily get arbitrary code
execution.

2. Get the shell.
Another bug is also layed on the tty's read logic. The code misuses the
`translate_addr` function. This results you to get heap overflow.

The remaining steps are simple.
You can leak the base of the binary by partially overwriting the `region->base`.
After leaking the base, you also get the base of libc as same way.
Since there are function pointers on the heap, i.e. `region->cb->cb_head`, you
can get the shell by jumping into the oneshot gadgets in the libc.

For more detail, you can refer `phase2.py`.
