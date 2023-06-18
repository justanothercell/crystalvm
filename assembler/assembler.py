import sys
import re
import struct

from instructions_map import instr_to_bits

start_addr = 0x0008DE00

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
        h = Here()
        h.offset = self.offset + other
        return h

    def __sub__(self, other):
        h = Here()
        h.offset = self.offset - other
        return h
    
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
        value = self.value + other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)

    def __sub__(self, other):
        value = self.value - other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
    def __mul__(self, other):
        value = self.value * other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
    def __floordiv__(self, other):
        value = self.value // other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
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
    
class Variable:
    def __init__(self, name):
        self.name = name

    def __repr__(self) -> str:
        return f'Variable({self.name})'

def eval_var(var: str, line=-1):
    var = var.strip()
    if var == '!':
        return Stack()
    if var[0] == '%':
        return var
    if var[0] == '@':
        if var[1:] in vars:
            return vars[var[1:]]
        return Variable(var[1:])
        # error(f'No such macro variable: "{var[1:]}"', line)
    if var[-1] == 'u':
        return uint(int(var[:-1], base=0))
    if len(var) > 2 and not var[1].isnumeric():
        return uint(int(var, base=0))
    return int(var, base=0)

vars = { 'Here': Here, 'uint': uint }

sections = []
current_section = Section(uint(start_addr))


def resolve(section):
    d = 0
    for c, instr in enumerate(section.instructions):
        if type(instr) == Location:
            instr.here.resolved = section.addr + (c - d) * 4
            d += 1
        elif type(instr) == list:
            d -= sum([type(x) != str for x in instr[1:]])


for i, line in enumerate(lines):
    line = line.strip()
    if len(line) == 0 or line.startswith(';'):
        continue
    if line[0] == '#':
        name, expr = line[1:].split(' ', 1)
        expr = expr.replace('$', 'Here()')
        for match in re.findall(r'0[xX][0-9A-Fa-f]+|0[oO][0-7]+|0[bB][01]+', expr):
            expr = expr.replace(match, str(eval_var(match)))
        try:
            if name in vars and name != '_':
                error(f'variable {name} already exists, can\'t be redefined', i)
            vars[name] = eval(expr, vars)
        except Exception as e:
            error(e, i)
    elif line[0] == '~':
        addr = eval_var(line[1:], line=i)
        if type(addr) != uint:
            error(f'section has to be resolved to an uint: {addr}', i)
        if addr.value < start_addr:
            error(f'no sections before {start_addr:X} allowed: {addr}', i)
        section = Section(addr)
        sections.append(section)
        resolve(current_section)
        current_section = section
    else:
        args = line.split()
        cmd = [args[0]]
        current_section.instructions.append(cmd)
        for a in args[1:]:
            r = eval_var(a, line=i)
            cmd.append(r)

resolve(current_section)

sections = sorted(sections, key=lambda s: s.addr.value)


def write_num(file, num) -> bool:
    if type(num) == int:
        file.write(num.to_bytes(4, 'big'))
        return True
    elif type(num) == uint:
        file.write(num.value.to_bytes(4, 'big'))
        return True
    elif type(num) == Here:
        file.write((num.resolved + num.offset).value.to_bytes(4, 'big'))
        return True
    elif type(num) == float:
        file.write(struct.pack('f', num))
        return True
    return False


with open(outfile, 'wb') as outbin:
    last_addr = start_addr-4
    for section in sections:
        if section.addr.value <= last_addr:
            raise Exception(f'Invalid location {section.addr.value:08X}, already are at {last_addr:08X}')
        outbin.write(bytes(section.addr.value-last_addr-4))
        last_addr = section.addr.value
        for instr in section.instructions:
            if type(instr) == Variable:
                instr = vars[instr.name]
            if type(instr) == Location:
                # marker
                pass
            elif type(instr) == list:
                cmd = instr[0]
                b = instr_to_bits(cmd)
                nums = []
                for arg in instr[1:]:
                    if type(arg) == str:
                        if arg == '!':
                            b += '1000000'
                        elif arg[0] == '%' and arg[1:].isnumeric():
                            n = int(arg[1:], base=16)
                            if n > 48:
                                raise Exception(f'invalid trg num "{arg}"')
                            b += '0' + f'{n:06b}'
                        elif len(arg) == 2 and arg[0] == '%' and arg[1] in ['S', 'I', 'L', 'C', 'F', 'Q']:
                            n = ['S', 'I', 'L', 'C', 'F', 'Q'].index(arg[1]) + 48
                            b += '0' + f'{n:06b}'
                        else:
                            raise Exception(f'invalid arg "{arg}"')
                    else:
                        b += '1111111'
                        nums.append(arg)
                b += '0' * (32-len(b))
                outbin.write(bytes(int(b, base=2).to_bytes(4, 'big')))
                for num in nums:
                    if not write_num(outbin, num):
                        raise Exception(f'invalid argument "{num}" of type {type(instr)} for {" ".join(instr)}')
            elif not write_num(outbin, instr):
                raise Exception(f'invalid instruction "{instr}" of type {type(instr)}')