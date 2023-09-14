# Instruction layout
```
 00000000111111110000000011111111
 0123456789A012345601234560123456
 ├─────────┘└──┬──┘└──┬──┘└──┬──┘
 │             │      │      │
 │    ┌─ arg0 ─┘      │      │
 │    ├─ arg1 ────────┘      │
 │    ├─ arg1 ───────────────┘
 │    └───────────────┬─ arg: 7 bits ───────────┐
 ├─ instr: 11 bits ─┐ │ 0XXXXXX - register 0-31 │
 │ 0123456789A      │ │ 1000000 - stack         │
 │ 01201201234      │ │ 1111111 - literal       │
 │ └┬┘└──┬───┘      │ └─────────────────────────┘
 │  │    └─ command │
 │  └─── group      │
 └──────────────────┘
```
# Control flow
| code            | name  | args | description           |
|-----------------|-------|------|-----------------------|
| `000 000000000` | jmp   |      | jump to address       |
| `000 000000000` | jeq   |      | jump if equal         |
| `000 000000000` | jne   |      | jump if unequal       |
| `000 000000000` | jgt   |      |                       |
| `000 000000000` | jge   |      |                       |
| `000 000000000` | jls   |      |                       |
| `000 000000000` | jle   |      |                       |
| `000 000000000` | call  |      |                       |
| `000 000000000` | ret   |      |                       |
| `000 000000000` | reti  |      |                       |
| `000 000000000` |       |      |                       |
| `000 000000000` |       |      |                       |

# Atomics and Threads
Atomic operatons are expensive as they pause all threads to avoid race conditions. 
All other threads must report back that they are waiting on the atomic before the operation can commence.

This virtual CPU can have an unlimited number of "real"/"physical" (in the emulation sense) threads. 
Threads can be creatted by forking the current thread. Inter-Thread communication can happen via nornal ram (atomic operations are provided),
or by sending a signal to the `t_sig` register, but only from parent to child

| code            | name      | args        | description                             |
|-----------------|-----------|-------------|-----------------------------------------|
| `000 000000000` | tch_modpr | chid pr val | thread child modify permission register |
| `000 000000000` | tch_getpr | chid pr     | thread child get permission register    |

# Interrupts and I/O
see [interrupt table](layout.md#interrupt-jump-table) for more information

| code           | name | side effects | description and notes |
|----------------|------|--------------|-----------------------|