## Specifications
- 32 bit memory
- double buffered screen 320 x 200 with RGB(A) 32 bit colors (64000 * 2)<br>
or 8x8 font (40x25 chars on screen) (1000 * 2)
- 2 display modes: text and color
- in-memory font for text mode to load custom font
- builtin font which can be overridden by setting memory in asm
- mouse input: 2 memory locations
- keyboard input
- system clock
- stack and regs

## Registers/Stack
- normal regs (0x00-0x29 / 0-47)
- program stack (S, reg 0x30 / 48)
- instruction pointer (I, reg 0x31 / 49)
- frame pointer (L, reg 0x32 / 50)<br>
(points to base of current stack frame) 
- carry register (C, reg 0x33 / 51)
- flag register (F, reg 0x34 / 52)
- interrupt info (Q, reg 0x35 / 53)
- interrupt device id (D, reg 0x36 / 54)

### Flag Register
```
32 bit
 00000000111111110000000011111111
         ^^      ^            ^^^
         EB      M            CSZ
Z: Zero/Equal Flag         (jz/jnz)
S: Sign Flag (1: negative) (jl/lnl)
C: Carry Flag              (jc/jnc)
M: Carry In: whether to carry in the overflow/carry of the last operation for this one

B: Actve buffer (0: buffer 1, 1: buffer 2)
E: Screen Mode (0: text, 1: video)
```

## Memory layout
```py
from math import ceil
def pad(x, p=2):
    return int(ceil(x / 16**p) * 16**p - x)

sections = [
    ('screen buffer 1', 320 * 200 * 4),
    ('screen buffer 2', 320 * 200 * 4),
    ('text buffer 1', 40 * 25),
    ('padding', pad(40 * 25)),
    ('text buffer 2', 40 * 25),
    ('padding', pad(40 * 25)),
    ('font bitmask', 256 * 8*8),
    ('stack', pad(256 * 8*8, p=4) + 0x600),
    ('interrupt handler', 0x200)
]

def pfmt():
    pos = 0 
    for section in sections:
        ppos = pos
        pos += section[1]
        print(f'0x{ppos:08X}-0x{pos-1:08X} {section[0]:20} ({section[1]:6} / 0x{section[1]:06X})')
    print(f'0x{pos:08X}            program start')
    print()
    print(f'=> images start at address 0x{ppos:08X}')

pfmt()
```
```
0x00000000-0x0003E7FF screen buffer 1      (256000 / 0x03E800)
0x0003E800-0x0007CFFF screen buffer 2      (256000 / 0x03E800)
0x0007D000-0x0007D3E7 text buffer 1        (  1000 / 0x0003E8)
0x0007D3E8-0x0007D3FF padding              (    24 / 0x000018)
0x0007D400-0x0007D7E7 text buffer 2        (  1000 / 0x0003E8)
0x0007D7E8-0x0007D7FF padding              (    24 / 0x000018)
0x0007D800-0x000817FF font bitmask         ( 16384 / 0x004000)
0x00081800-0x0008DDFF stack                ( 50688 / 0x00C600) (12672 / 0x3180 stack elements)
0x0008DE00-0x0008DFFF interrupt handler    (   512 / 0x000200) (  128 / 0x0080 instructions)
0x0008E000            program start

=> images start at address 0x0008DE00
```

## Instructions
```
32 bit
00000000111111110000000011111111
^         ^^     ^^     ^^     ^
L---------JL-----JL-----JL-----J
11         7      7      7
| instr    | arg0 | arg1 | arg2
```

### Args
```
0ABCDEF: register
1000000: stack
1111111: literal value
```


### Instruction Groups

11 Bits: `ABCDEFGH`

| ABC   |                        |
|-------|------------------------|                 
| `000` | integer arithmetic     |
| `001` | float arithmetic       |
| `010` | control flow           |
| `100` | stack/registers        |
| `111` | I/O / System           |

| `___ ABCDEFGH` | casm | arg0 | arg1 | arg2 |
|----------------|------|------|------|------|
| `000 00000000` | noop | ---  | ---  | ---  |

### Integer Arithmetic

#### Operations implemented for both signed and unsigned ints
The D bit specifices whether to perform an unsigned or signed operation:<br>
`0: unsigned (example: addu)`<br>
`1: signed (example: addi)`

The carry register contains the over/underflow of the operation

| `___ ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation        |
|-------------------|------|------|------|------|------------------|
| `000 000 X 0 001` | addx | lhs  | rhs  | res  | res = lhs + rhs  |
| `000 000 X 0 010` | subx | lhs  | rhs  | res  | res = lhs - rhs  |
| `000 000 X 0 011` | mulx | lhs  | rhs  | res  | res = lhs * rhs  |
| `000 000 X 0 100` | divx | lhs  | rhs  | res  | res = lhs / rhs  |
| `000 000 X 0 101` | modx | lhs  | rhs  | res  | res = lhs % rhs  |
| `000 000 X 0 111` | cmpx | lhs  | rhs  | res  | set S+Z of F reg |

#### Operations only for signed ints
| `___ ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation        |
|-------------------|------|------|------|------|------------------|
| `000 000 1 1 000` | absi | arg  | res  | ---  | res = \|arg\|    |
| `000 000 1 1 001` | powi | lhs  | rhs  | res  | res = lhs ** rhs |

#### Bitwise operations
| `___ ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation                  |
|-------------------|------|------|------|------|----------------------------|
| `000 000 0 1 000` | and  | lhs  | rhs  | res  | res = lhs & rhs            |
| `000 000 0 1 001` | or   | lhs  | rhs  | res  | res = lhs | rhs            |
| `000 000 0 1 010` | xor  | lhs  | rhs  | res  | res = lhs ^ rhs            |
| `000 000 0 1 011` | not  | arg  | res  | ---  | res = ~arg                 |
| `000 000 0 1 100` | shl  | lhs  | rhs  | res  | res = lhs << rhs           |
| `000 000 0 1 101` | shr  | lhs  | rhs  | res  | res = lhs >> rhs           |
| `000 000 0 1 110` | rol  | lhs  | rhs  | res  | res = lhs <<< rhs (rotate) |
| `000 000 0 1 111` | ror  | lhs  | rhs  | res  | res = lhs >>> rhs (rotate) |

### Conversions
| `___ ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation       |
|-------------------|------|------|------|------|-----------------|
| `000 000 1 1 100` | itu  | arg  | res  | ---  | int -> unsigned |
| `000 000 1 1 101` | uti  | arg  | res  | ---  | unsigned -> int |
| `000 000 1 1 110` | itf  | arg  | res  | ---  | int -> float    |
| `000 000 1 1 111` | fti  | arg  | res  | ---  | float -> int    |

### Floating point Arithmetic

| `___ ABC DEFGH` | casm  | arg0 | arg1 | arg2 | operation          |
|-----------------|-------|------|------|------|--------------------|
| `000 001 00000` | addf  | lhs  | rhs  | res  | res = lhs + rhs    |
| `000 001 00001` | subf  | lhs  | rhs  | res  | res = lhs - rhs    |
| `000 001 00010` | mulf  | lhs  | rhs  | res  | res = lhs * rhs    |
| `000 001 00011` | divf  | lhs  | rhs  | res  | res = lhs / rhs    |
| `000 001 00100` | modf  | lhs  | rhs  | res  | res = lhs % rhs    |
| `000 001 00101` | absf  | arg  | res  | ---  | res = \|arg\|      |
| `000 001 00110` | powfi | lhs  | rhs  | res  | res = lhs ** i_rhs |
| `000 001 10110` | powff | lhs  | rhs  | res  | res = lhs ** rhs   |
| `000 001 00111` | cmpf  | lhs  | rhs  | res  | set S+Z of F reg   |
| `000 001 01000` | sqrt  | arg  | res  | ---  | res = sqrt(arg)    |
| `000 001 01001` | exp   | arg  | res  | ---  | res = arg**e       |
| `000 001 01010` | log   | lhs  | rhs  | res  | res = log(lhs, rhs)|
| `000 001 11010` | ln    | arg  | res  | ---  | res = ln(arg)      |
| `000 001 01011` | sin   | arg  | res  | ---  | res = sin(arg)     |
| `000 001 01100` | asin  | arg  | res  | ---  | res = asin(arg)    |
| `000 001 01101` | cos   | arg  | res  | ---  | res = cos(arg)     |
| `000 001 01110` | tan   | arg  | res  | ---  | res = tan(arg)     |
| `000 001 01111` | atan  | arg  | res  | ---  | res = atan(arg)    |
| `000 001 10000` | sinh  | arg  | res  | ---  | res = sinh(arg)    |
| `000 001 10001` | asih  | arg  | res  | ---  | res = asinh(arg)   |
| `000 001 10010` | cosh  | arg  | res  | ---  | res = cosh(arg)    |
| `000 001 10011` | acoh  | arg  | res  | ---  | res = acosh(arg)   |

### Control Flow
#### Jumps
| `___ ABC DEFGH` | casm | arg0 | arg1 | arg2 | predicate (F reg bit) |
|-----------------|------|------|------|------|-----------------------|
| `000 010 00000` | jmp  | addr | ---  | ---  | true                  |
| `000 010 00010` | jz   | addr | ---  | ---  | Z == 1                |
| `000 010 00011` | jnz  | addr | ---  | ---  | Z == 0                |
| `000 010 00100` | jl   | addr | ---  | ---  | S == 1                |
| `000 010 00101` | jnl  | addr | ---  | ---  | S == 0                |
| `000 010 00110` | jc   | addr | ---  | ---  | C == 1                |
| `000 010 00111` | jnc  | addr | ---  | ---  | C == 0                |

#### Procedures
| `___ ABC D EFGH` | casm | arg0 | arg1 | arg2 | notes                                    |
|------------------|------|------|------|------|------------------------------------------|
| `000 010 1 0000` | call | addr | ---  | ---  | pushes reg I+F to stack, jumps, sets F   |
| `000 010 1 0001` | ret  | ---  | ---  | ---  | jumps to addr on stack, restores F       |

### Stack and Registers
| `___ ABC DEFGH` | casm  | arg0 | arg1 | arg2 | notes                                                     |
|-----------------|-------|------|------|------|-----------------------------------------------------------|
| `000 100 00000` | move  | src  | dst  | ---  | move src to dest, popping if src is stack                 |
| `000 100 00001` | ld    | dst  | src  | ---  | set dst to memory @src                                    |
| `000 100 00011` | st    | src  | dst  | ---  | set memory @dst to src                                    |
| `000 100 00101` | ldb   | src  | dst  | ---  | set dst to memory @sr (1 byte)                            |
| `000 100 00111` | stb   | src  | dst  | ---  | set memory @dst to src (1 byte)                           |
| `000 100 01000` | dup   | ---  | ---  | ---  | duplicate topmost stack elem                              |
| `000 100 01001` | over  | ---  | ---  | ---  | dup second to topmost elem                                |
| `000 100 01010` | srl   | ---  | ---  | ---  | rotates the top 3 elems left:  ___ ABC -> BCA             |
| `000 100 01011` | srr   | ---  | ---  | ---  | rotates the top 3 elems right: ___ ABC -> CAB             |
| `000 100 01100` | enter | ---  | ---  | ---  | saves L to stack, sets L to S                             |
| `000 100 01101` | leave | ---  | ---  | ---  | set S to L, restores L from stack                         |
| `000 100 01110` | pshar | ---  | ---  | ---  | push all regs in order onto stack, except for I and F     |
| `000 100 01111` | resar | ---  | ---  | ---  | restore all regs to values from stack, except for I and F |

## Interrupts and System
| `___ ABC DE FGH` | casm   | arg0   | arg1  | arg2 | notes                                                               |
|------------------|--------|--------|-------|------|---------------------------------------------------------------------|
| `000 111 00 000` | time   | upper  | lower |      | writes the current u64 unix time in millis to reg @upper and @lower |
| `000 111 00 001` | wait   | c      | ---   | ---  | wait c instr cycles, then interrupt with id 0. c = 0 cancels        |
| `000 111 01 001` | dread  | did    | start | len  | reads len bytes to mem starting at start from device. len=0 cancels |
| `000 111 01 010` | dwrite | did    | start | len  | writes len bytes starting at start to device. len=0 cancels         |
| `000 111 01 011` | dstate | did    | read  | write| returns the read and write bytes left                               |

### Interrupts/Devices
A device can trigger the following events:
- connected: the device was connected             (0000)
- read_complete: the read buffer read completely  (0001)
- write_complete: the write buffer was filled     (0010)
- disconnected: the device was disconnected       (0011)

On an interrupt % is filled set to the device id and interrupt type.
interrupt type is the topmost 4 bits and device id is the remaining bits.

As device id gets incremented with every device attaching, this results in a maximum of 268435455 device attachments (28 bits)


## Debugging
| `___ ABCDEFGH` | casm       | arg0       | arg1     | arg2    | notes                                |
|----------------|------------|------------|----------|---------|--------------------------------------|
| `111 11111111` | breakpoint | optional: str tag of length 0-2 | hint for the debugger, ignored by VM |

`breakpoint` or `breakpoint "xx"`