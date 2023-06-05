import sys
import re

start_addr = 0x00022000

if len(sys.argv) != 3:
    print('usage: assepbler.py <in.casm> <out.cstl>')
    exit(1)
file = sys.argv[1]
outfile = sys.argv[2]

with open(file, 'r') as source_file:
    lines = source_file.readlines()

def error(err, line):
    print(f'Error in line {line+1}: {err}')
    exit(1)

class Here:
    def __init__(self):
        self.offset = 0
        self.resolved = None
        if current_section is None:
            raise Exception('Cannot use $ outside of a section')
        current_section.instructions.append(Location(self))

    def __add__(self, other):
        self.offset += other
        return self

    def __sub__(self, other):
        self.offset -= other
        return self
    
    def __repr__(self) -> str:
        if self.resolved is not None:
            return f'{self.resolved + self.offset}'
        if self.offset == 0:
            return f'$0x{id(self):X}'
        elif self.offset > 0:
            return f'$0x{id(self):X} + {self.offset}'
        else:
            return f'$0x{id(self):X} - {abs(self.offset)}'

class uint:
    def __init__(self, value):
        self.value = value

    def __add__(self, other):
        self.value += other
        if self.value < 0:
            raise Exception(f'Uint cannot be negative: {self}!')
        return self

    def __sub__(self, other):
        self.value -= other
        if self.value < 0:
            raise Exception(f'Uint cannot be negative: {self}!')
        return self
    
    def __mul__(self, other):
        self.value *= other
        if self.value < 0:
            raise Exception(f'Uint cannot be negative: {self}!')
        return self
    
    def __floordiv__(self, other):
        self.value //= other
        if self.value < 0:
            raise Exception(f'Uint cannot be negative: {self}!')
        return self
    
    def __repr__(self) -> str:
        return f'0x{self.value:X}u'
    
    def __str__(self) -> str:
        return f'uint(0x{self.value:X})'

class Section:
    def __init__(self, addr) -> None:
        self.addr = addr
        self.instructions = []


class Location:
    def __init__(self, here) -> None:
        self.here = here

    def __repr__(self) -> str:
        return f'Location({self.here})'

class Stack:
    def __repr__(self) -> str:
        return 'Stack'

def eval_var(var: str, line=-1):
    var = var.strip()
    if var == '!':
        return Stack()
    if var[0] == '%':
        return var
    if var[0] == '@':
        if var[1:] in vars:
            return vars[var[1:]]
        error(f'No such macro variable: "{var[1:]}"', line)
    if var[-1] == 'u':
        return uint(int(var[:-1], base=0))
    if len(var) > 2 and not var[1].isnumeric():
        return uint(int(var, base=0))
    return int(var, base=0)

vars = { 'Here': Here, 'uint': uint }

sections = []
current_section = None


def resolve(section):
    d = 0
    for c, instr in enumerate(section.instructions):
        if type(instr) == Location:
            instr.here.resolved = section.addr + c - d
            d += 1

for i, line in enumerate(lines):
    line = line.strip()
    if len(line) == 0:
        continue
    if line[0] == '#':
        name, expr = line[1:].split(' ', 1)
        expr = expr.replace('$', 'Here()')
        for match in re.findall(r'0[xX][0-9A-Fa-f]+|0[oO][0-7]+|0[bB][01]+', expr):
            expr = expr.replace(match, str(eval_var(match)))
        try:
            vars[name] = eval(expr, vars)
        except Exception as e:
            error(e, i)
    elif line[0] == '~':
        addr = eval_var(line[1:], line=i)
        if type(addr) != uint:
            error(f'section has to be resolved to an uint: {addr}', i)
        if addr.value < start_addr:
            error(f'no sections before {start_addr} allowed: {addr}', i)
        section = Section(addr)
        sections.append(section)
        if current_section is not None:
            resolve(current_section)
        current_section = section
    else:
        args = line.split()
        cmd = [args[0]]
        current_section.instructions.append(cmd)
        for a in args[1:]:
            r = eval_var(a, line=i)
            if type(r) == str:
                cmd.append(r)
            else:
                current_section.instructions.append(r)

if current_section is not None:
    resolve(current_section)

sections = sorted(sections, key=lambda s: s.addr.value)



with open(outfile, 'wb') as outbin:
    last_addr = start_addr
    for section in sections:
        for instr in section.instructions:
            if type(instr) == Location:
                # is only marker
                pass
            elif type(instr) == list:
                b = '11111111_000'
                cmd = instr[0]
                for arg in instr[1:]:
                    if arg == '!':
                        b += '1000000'
                    elif arg[0] == '%' and arg[1:].isnumeric():
                        n = int(arg[1:])
                        if n > 48:
                            raise Exception(f'invalid trg num "{arg}"')
                        b += '0' + f'{n:06b}'
                    elif len(arg) == 2 and arg[0] == '%' and arg[1] in ['S', 'I', 'L', 'C', 'F']:
                        n = ['S', 'I', 'L', 'C', 'F'].index(arg[1])
                        b += '0' + f'{n:06b}'
                    else:
                        raise Exception(f'invalid arg "{arg}"')
                b += '0' * (33-len(b))
                outbin.write(bytes(int(b, base=2).to_bytes(4, 'big')))
            elif type(instr) == int:
                outbin.write(instr.to_bytes(4, 'little'))
            elif type(instr) == uint:
                outbin.write(instr.value.to_bytes(4, 'little'))
            elif type(instr) == Here:
                outbin.write((instr.resolved + instr.offset).value.to_bytes(4, 'little'))
            else:
                raise Exception