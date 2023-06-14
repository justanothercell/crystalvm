import sys

from instructions_map import bits_to_instr

if len(sys.argv) < 3:
    print('usage: assepbler.py <in.casm> <out.cstl> <offset,len: optional>')
    exit(1)

file = sys.argv[1]
outfile = sys.argv[2]

if len(sys.argv) > 3:
    offset, length = [int(i, base=0) for i in sys.argv[3].split(',')]
else:
    offset, length = 0, None

with open(file, 'rb') as source_file:
    with open(outfile, 'w') as dest_file:
        source_file.read(offset)
        while raw := source_file.read(4):
            bits = ''.join([bin(b)[2:].rjust(8, '0') for b in raw])
            instr, *args = bits[0:11], bits[11:18], bits[18:25], bits[25:32]
            try:
                instr = bits_to_instr(instr)
            except:
                pass
            params = []
            for arg in args:
                if arg == '1000000':
                    params.append('!')
                else:
                    r = int(arg, base=2)
                    if r >= 48:
                        params.append(f'%{["S", "I", "L", "C", "F"][r-48]}')
                    else:
                        params.append(f'%{r:02X}')

            if instr == 'ldl':
                dest_file.write(f'{instr:5} {params[0]:3} {params[1]:3} {params[2]:3} 0x{int.from_bytes(source_file.read(4), "little"):08X}\n')
                if length is not None:
                    length -= 4
                    if length <= 0:
                        break
            else:
                dest_file.write(f'; {instr:5} {params[0]:3} {params[1]:3} {params[2]:3}\n')
            if length is not None:
                length -= 4
                if length <= 0:
                    break