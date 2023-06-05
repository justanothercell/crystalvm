## Specifications
- 32 bit memory
- double buffered screen 320 x 200 with RGB(A) 32 bit colors (64000 * 4 bytes * 2)
- 2 display modes: text and color
- in-memory font for text mode to load custom font
- builtin font which can be overridden by setting memory in asm
- mouse input: 2 memory locations
- keyboard input
- system clock
- stack and regs

## Registers/Stack
- normal regs (0x00-0x30 / 0-48)
- program stack (S, reg 0x31 / 49)
- instruction pointer (I, reg 0x32 / 50)
- frame pointer (F, reg 0x33 / 51)<br>
(points to base of current stack frame) 
- carry register (C, reg 0x34 / 52)
- flag register (C, reg 0x35 / 53)

### Flag Register
```
32 bit
 00000000111111110000000011111111
 ^       ^^                  ^^^^
 K       SB                  OCSZ
Z: Zero Flag     (jz/jnz)
S: Sign Flag     (jl/lnl)
C: Carry Flag    (jc/jnc)
O: Overflow Flag (jo/jno)

S: Screen Mode (0: text, 1: video)
B: Actve buffer (0: buffer 1, 1: buffer 2)

K: Keyboard Event queued
```

## Memory layout
```
0x000000-0x00f9ff screen buffer 1    (64000 / 0xfa00, page 0-15.625)
0x00fa00-0x01f3ff screen buffer 2    (64000 / 0xfa00, page 15.625-31.25)
0x01f400-0x0213ff stack              ( 8192 / 0x2000, page 31.25-33.25)
0x021400-0x0214ff ascii font bitmask ( 256  / 0x00ff, page 33.25-33.3125)
0x021500-0x021fff space              ( 2816 / 0x0b00, page 33.3125-34)
0x022000-0x022fff user page "0"      ( 4096 / 0x1000, page 34-35)
0x023000-0x023fff user page "1"      ( 4096 / 0x1000, page 35-36)
...
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

| ABC   |                       |
|-------|-----------------------|                 
| `000` | integer arithmetic    |
| `001` | float arithmetic      |
| `010` | control flow          |
| `100` | stack/registers       |
| `101` | memory                |
| `110` | I/O                   |

### Integer Arithmetic

#### Operations implemented for both signed and unsigned ints
The D bit specifices whether to perform an unsigned or signed operation:<br>
`0: unsigned (example: addu)`<br>
`1: signed (example: addi)`<br>
The carry register contains the over/underflow of the operation

| `ABC D E FGH` | asm  | arg0 | arg1 | arg2 | operation        |
|---------------|------|------|------|------|------------------|
| `000 X 0 000` | addx | lhs  | rhs  | res  | res = lhs + rhs  |
| `000 X 0 001` | subx | lhs  | rhs  | res  | res = lhs - rhs  |
| `000 X 0 010` | mulx | lhs  | rhs  | res  | res = lhs * rhs  |
| `000 X 0 011` | divx | lhs  | rhs  | res  | res = lhs / rhs  |
| `000 X 0 100` | modx | lhs  | rhs  | res  | res = lhs % rhs  |
| `000 X 0 111` | cmpx | lhs  | rhs  | res  | set S+Z of F reg |

#### Operations only for signed ints
| `ABC D E FGH` | asm  | arg0 | arg1 | arg2 | operation        |
|---------------|------|------|------|------|------------------|
| `000 1 1 000` | absi | arg  | ---  | res  | res = \|arg\|    |
| `000 1 1 001` | powi | lhs  | rhs  | res  | res = lhs ** rhs |

#### Bitwise operations
| `ABC D E FGH` | asm | arg0 | arg1 | arg2 | operation        |
|---------------|-----|------|------|------|------------------|
| `000 0 1 000` | and | lhs  | rhs  | res  | res = lhs & rhs  |
| `000 0 1 001` | or  | lhs  | rhs  | res  | res = lhs | rhs  |
| `000 0 1 010` | xor | lhs  | rhs  | res  | res = lhs ^ rhs  |
| `000 0 1 011` | not | arg  | ---  | res  | res = ~arg       |
| `000 0 1 100` | shl | lhs  | rhs  | res  | res = lhs << rhs |
| `000 0 1 101` | shr | lhs  | rhs  | res  | res = lhs >> rhs |

### Conversions
| `ABC D E FGH` | asm | arg0 | arg1 | arg2 | operation       |
|---------------|-----|------|------|------|-----------------|
| `000 1 1 100` | itu | arg  | ---  | res  | int -> unsigned |
| `000 1 1 101` | uti | arg  | ---  | res  | unsigned -> int |
| `000 1 1 110` | itf | arg  | ---  | res  | int -> float    |
| `000 1 1 111` | fti | arg  | ---  | res  | float -> int    |

### Floating point Arithmetic

| `ABC DEFGH` | asm  | arg0 | arg1 | arg2 | operation         |
|-------------|------|------|------|------|-------------------|
| `001 00000` | addf | lhs  | rhs  | res  | res = lhs + rhs   |
| `001 00001` | subf | lhs  | rhs  | res  | res = lhs - rhs   |
| `001 00010` | mulf | lhs  | rhs  | res  | res = lhs * rhs   |
| `001 00011` | divf | lhs  | rhs  | res  | res = lhs / rhs   |
| `001 00100` | modf | lhs  | rhs  | res  | res = lhs % rhs   |
| `001 00101` | absf | arg  | ---  | res  | res = \|arg\|     |
| `001 00110` | powf | lhs  | rhs  | res  | res = lhs % rhs   |
| `001 00111` | cmpf | lhs  | rhs  | res  | set S+Z of F reg  |
| `001 01000` | sqrt | arg  | ---  | res  | res = sqrt(arg)   |
| `001 01001` | exp  | arg  | ---  | res  | res = arg**e      |
| `001 01010` | ln   | arg  | ---  | res  | res = ln(arg)     |
| `001 01011` | sin  | arg  | ---  | res  | res = sin(arg)    |
| `001 01100` | asin | arg  | ---  | res  | res = asin(arg)   |
| `001 01101` | cos  | arg  | ---  | res  | res = cos(arg)    |
| `001 01110` | tan  | arg  | ---  | res  | res = tan(arg)    |
| `001 01111` | atan | arg  | ---  | res  | res = atan(arg)   |
| `001 10000` | sinh | arg  | ---  | res  | res = sinh(arg)   |
| `001 10001` | asih | arg  | ---  | res  | res = asinh(arg)  |
| `001 10010` | cosh | arg  | ---  | res  | res = cosh(arg)   |
| `001 10011` | acoh | arg  | ---  | res  | res = acosh(arg)  |

### Control Flow
#### Jumps
| `ABC DEFGH` | asm  | arg0 | arg1 | arg2 | predicate (F reg bit) |
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
| `ABC D EFGH` | asm  | arg0 | arg1 | arg2 | notes                                    |
|--------------|------|------|------|------|------------------------------------------|
| `010 1 0000` | call | addr | ---  | ---  | pushes reg I+F to stack, jumps           |
| `010 1 0001` | ret  | ---- | ---  | ---  | jumps to last instr on stack, restores F |

### Stack and Registers
| `ABC D EFGH` | asm  | arg0 | arg1 | arg2 | notes                                     |
|--------------|------|------|------|------|-------------------------------------------|
| `100 0 0000` | copy | src  | dst  | ---  | copy src to dst                           |
| `100 0 0001` | move | ---- | ---  | ---  | move src to dest, popping if src is stack |
| `100 0 0001` | move | ---- | ---  | ---  | move src to dest, popping if src is stack |