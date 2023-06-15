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
- interrupt id (Q, reg 0x35 / 53)

### Flag Register
```
32 bit
 00000000111111110000000011111111
 ^       ^^                  ^^^^
 K       SB                  OCSZ
Z: Zero Flag     (jz/jnz)
S: Sign Flag (1: negative)    (jl/lnl)
C: Carry Flag    (jc/jnc)
O: Overflow Flag (jo/jno)

S: Screen Mode (0: text, 1: video)
B: Actve buffer (0: buffer 1, 1: buffer 2)

K: Keyboard Event queued
```

## Memory layout
```
0x00000000-0x0003e7ff screen buffer 1    (256000 / 0x03e800)
0x0003e800-0x0007cfff screen buffer 2    (256000 / 0x03e800)
0x0007d000-0x0007df9f text buffer 1      (  4000 / 0x000fa0)
0x0007dfa0-0x0007dfff alignment space    (    96 / 0x000060)
0x0007e000-0x0007ef9f text buffer 2      (  4000 / 0x000fa0)
0x0007efa0-0x0007efff alignment space    (    96 / 0x000060)
0x0007f000-0x00086fff stack              ( 32768 / 0x008000)
0x00087000-0x000873ff ascii font bitmask (  1024 / 0x000400)
0x00087400-0x00087fff interrupt handler  (  3072 / 0x000C00)
0x00088000 program start

=> images start at address 0x00087400
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
A BCDEFG
^ ^^^^^^
| reg id if A==1 else ignored
|
1: Stack 
0: Register
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

| `ABCDEFGH` | casm | arg0 | arg1 | arg2 |
|------------|------|------|------|------|
| `00000000` | noop | ---  | ---  | ---  |

### Integer Arithmetic

#### Operations implemented for both signed and unsigned ints
The D bit specifices whether to perform an unsigned or signed operation:<br>
`0: unsigned (example: addu)`<br>
`1: signed (example: addi)`

The carry register contains the over/underflow of the operation

| `ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation        |
|---------------|------|------|------|------|------------------|
| `000 X 0 001` | addx | lhs  | rhs  | res  | res = lhs + rhs  |
| `000 X 0 010` | subx | lhs  | rhs  | res  | res = lhs - rhs  |
| `000 X 0 011` | mulx | lhs  | rhs  | res  | res = lhs * rhs  |
| `000 X 0 100` | divx | lhs  | rhs  | res  | res = lhs / rhs  |
| `000 X 0 101` | modx | lhs  | rhs  | res  | res = lhs % rhs  |
| `000 X 0 111` | cmpx | lhs  | rhs  | res  | set S+Z of F reg |

#### Operations only for signed ints
| `ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation        |
|---------------|------|------|------|------|------------------|
| `000 1 1 000` | absi | arg  | res  | ---  | res = \|arg\|    |
| `000 1 1 001` | powi | lhs  | rhs  | res  | res = lhs ** rhs |

#### Bitwise operations
| `ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation                  |
|---------------|------|------|------|------|----------------------------|
| `000 0 1 000` | and  | lhs  | rhs  | res  | res = lhs & rhs            |
| `000 0 1 001` | or   | lhs  | rhs  | res  | res = lhs | rhs            |
| `000 0 1 010` | xor  | lhs  | rhs  | res  | res = lhs ^ rhs            |
| `000 0 1 011` | not  | arg  | res  | ---  | res = ~arg                 |
| `000 0 1 100` | shl  | lhs  | rhs  | res  | res = lhs << rhs           |
| `000 0 1 101` | shr  | lhs  | rhs  | res  | res = lhs >> rhs           |
| `000 0 1 110` | rol  | lhs  | rhs  | res  | res = lhs <<< rhs (rotate) |
| `000 0 1 111` | ror  | lhs  | rhs  | res  | res = lhs >>> rhs (rotate) |

### Conversions
| `ABC D E FGH` | casm | arg0 | arg1 | arg2 | operation       |
|---------------|------|------|------|------|-----------------|
| `000 1 1 100` | itu  | arg  | res  | ---  | int -> unsigned |
| `000 1 1 101` | uti  | arg  | res  | ---  | unsigned -> int |
| `000 1 1 110` | itf  | arg  | res  | ---  | int -> float    |
| `000 1 1 111` | fti  | arg  | res  | ---  | float -> int    |

### Floating point Arithmetic

| `ABC DEFGH` | casm  | arg0 | arg1 | arg2 | operation          |
|-------------|-------|------|------|------|--------------------|
| `001 00000` | addf  | lhs  | rhs  | res  | res = lhs + rhs    |
| `001 00001` | subf  | lhs  | rhs  | res  | res = lhs - rhs    |
| `001 00010` | mulf  | lhs  | rhs  | res  | res = lhs * rhs    |
| `001 00011` | divf  | lhs  | rhs  | res  | res = lhs / rhs    |
| `001 00100` | modf  | lhs  | rhs  | res  | res = lhs % rhs    |
| `001 00101` | absf  | arg  | res  | ---  | res = \|arg\|      |
| `001 00110` | powfi | lhs  | rhs  | res  | res = lhs ** i_rhs |
| `001 10110` | powff | lhs  | rhs  | res  | res = lhs ** rhs   |
| `001 00111` | cmpf  | lhs  | rhs  | res  | set S+Z of F reg   |
| `001 01000` | sqrt  | arg  | res  | ---  | res = sqrt(arg)    |
| `001 01001` | exp   | arg  | res  | ---  | res = arg**e       |
| `001 01010` | log   | lhs  | rhs  | res  | res = log(lhs, rhs)|
| `001 11010` | ln    | arg  | res  | ---  | res = ln(arg)      |
| `001 01011` | sin   | arg  | res  | ---  | res = sin(arg)     |
| `001 01100` | asin  | arg  | res  | ---  | res = asin(arg)    |
| `001 01101` | cos   | arg  | res  | ---  | res = cos(arg)     |
| `001 01110` | tan   | arg  | res  | ---  | res = tan(arg)     |
| `001 01111` | atan  | arg  | res  | ---  | res = atan(arg)    |
| `001 10000` | sinh  | arg  | res  | ---  | res = sinh(arg)    |
| `001 10001` | asih  | arg  | res  | ---  | res = asinh(arg)   |
| `001 10010` | cosh  | arg  | res  | ---  | res = cosh(arg)    |
| `001 10011` | acoh  | arg  | res  | ---  | res = acosh(arg)   |

### Control Flow
#### Jumps
| `ABC DEFGH` | casm | arg0 | arg1 | arg2 | predicate (F reg bit) |
|-------------|------|------|------|------|-----------------------|
| `010 00000` | jmp  | addr | ---  | ---  | true                  |
| `010 00010` | jz   | addr | ---  | ---  | Z == 1                |
| `010 00011` | jnz  | addr | ---  | ---  | Z == 0                |
| `010 00100` | jl   | addr | ---  | ---  | S == 1                |
| `010 00101` | jnl  | addr | ---  | ---  | S == 0                |
| `010 00110` | jc   | addr | ---  | ---  | C == 1                |
| `010 00111` | jnc  | addr | ---  | ---  | C == 0                |
| `010 01000` | jo   | addr | ---  | ---  | O == 1                |
| `010 01001` | jno  | addr | ---  | ---  | O == 0                |
#### Procedures
| `ABC D EFGH` | casm | arg0 | arg1 | arg2 | notes                                    |
|--------------|------|------|------|------|------------------------------------------|
| `010 1 0000` | call | addr | ---  | ---  | pushes reg I+F to stack, jumps, sets F   |
| `010 1 0001` | ret  | ---  | ---  | ---  | jumps to addr on stack, restores F       |

### Stack and Registers
| `ABC DEFGH` | casm  | arg0 | arg1 | arg2 | notes                                     |
|-------------|-------|------|------|------|-------------------------------------------|
| `100 00000` | move  | src  | dst  | ---  | move src to dest, popping if src is stack |
| `100 00001` | ld    | dst  | src  | ---  | set dst to memory @src                    |
| `100 00010` | ldl   | dst  | ---  | ---  | set dst to next value in memory after I   |
| `100 00011` | st    | src  | dst  | ---  | set memory @dst to src                    |
| `100 01000` | dup   | ---  | ---  | ---  | duplicate topmost stack elem              |
| `100 01001` | over  | ---  | ---  | ---  | dup second to topmost elem                |
| `100 01010` | srl   | ---  | ---  | ---  | rotates the top 3 elems left: ABC -> BCA  |
| `100 01011` | srr   | ---  | ---  | ---  | rotates the top 3 elems right: ABC -> CAB |
| `100 01100` | enter | ---  | ---  | ---  | saves L to stack, sets L to S             |
| `100 01101` | leave | ---  | ---  | ---  | set S to L, restores L from stack         |
| `100 01110` | pshar | ---  | ---  | ---  | push all regs in order onto stack         |
| `100 01111` | resar | ---  | ---  | ---  | restore all regs to values from stack     |

### I/O and System
#### System
| `ABC DE FGH` | casm  | arg0   | arg1 | arg2 | notes                                     |
|--------------|-------|--------|------|------|-------------------------------------------|
| `111 00 000` | sleep | millis | ---  | ---  | sleep the specified amout of milliseconds |
| `111 00 001` | wait  | cycles | ---  | ---  | wait n instr cycles, then interrupt with id 0. 'wait 0' cancels a wait |
| `111 00 002` | dinfo | id     | dst  | ---  | writes device info to *dst                |

## Interrupts
