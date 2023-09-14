from sys import argv

import asm_funcs


def parse(code: str, file: str):
    lines = []
    byte_pos = 0
    vars = { **asm_funcs.__dict__ }
    for (l, line) in enumerate(code.split('\n')):
        line = line.strip()
        if len(line) == 0:
            continue
        if line[0] == '$':
            pos = eval(line[1:].replace('@', 'label_'), vars, vars)
            if byte_pos > pos:
                error(file, l, f'trying to set position to byte {pos:X} but is already at {byte_pos:X}')
            byte_pos = pos
        elif line[0] == '.':
            line, ty = parse_var_name(line[1:])
            if len(ty) == 0:
                error(file, l, 'a type has to follow `.`')
            tys = ['str', 'bytes', 'u32', 'i32', 'u8', 'i8']
            if ty not in tys:
                error(file, l, 'invalid type .{ty}: {tys}')
            byte_pos_start = byte_pos
            if ty == 'str':
                data = eval(line.replace('@', 'label_'), vars, vars)
                raw_data = data.encode('utf-8')
                byte_pos += len(raw_data)
            if ty == 'bytes':
                data = eval(line.replace('@', 'label_'), vars, vars)
                raw_data = data
                byte_pos += len(raw_data)
            if ty == 'u32':
                raw_data = lambda: eval(line.replace('@', 'label_'), vars, vars).to_bytes(4, 'little')
                byte_pos += 4
            if ty == 'i32':
                raw_data = lambda: eval(line.replace('@', 'label_'), vars, vars).to_bytes(4, 'little', signed=True)
                byte_pos += 4
            if ty == 'u8':
                raw_data = lambda: eval(line.replace('@', 'label_'), vars, vars).to_bytes(1, 'little')
                byte_pos += 1
            if ty == 'i8':
                raw_data = lambda: eval(line.replace('@', 'label_'), vars, vars).to_bytes(1, 'little', signed=True)
                byte_pos += 1
            lines.append(('raw_data', byte_pos_start, raw_data))
        else:
            line, ident = parse_var_name(line)
            if len(ident) == 0:
                error(file, l, 'line has to either start with `.`, `$` or <ident>')
            line = line.strip()
            
            if len(line) > 0 and line[0] == ':':
                raw_ident = 'label_' + ident
                if raw_ident in vars:
                    error(file, l, 'label {ident} is already in use')
                vars[raw_ident] = byte_pos
            else:
                lines.append(('instruction', byte_pos, lambda: parse_instr(ident, line, vars)))
    for x in lines:
        print(x)

def parse_var_name(line: str) -> (str, str):
    line = line.strip()
    var = ''
    while len(line) > 0 and (line[0].isalnum() or line[0] == '_'):
        var += line[0]
        line = line[1:]
    return line, var

def parse_instr(instr, args, vars):
    pass

def error(file, line, reason):
    print(f'Error in {file}:{line}\n    {reason}')
    exit(1)

if __name__ == '__main__':
    file = argv[1]
    out_file = argv[2]
    with open(file, 'r') as f:
        parse(f.read(), file)