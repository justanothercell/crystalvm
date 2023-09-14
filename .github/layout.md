# Interrupt Jump Table
Starting at address 0x00000000 is the interrupt jump table.
It features addresses to jump to when specific interrupts happen
| adddress     | name     | description                                                               |
|--------------|----------|---------------------------------------------------------------------------|
| `0x00000000` | `t_main` | main program start (doesnt count as interrupt but is stored here anyways) |
| `0x00000004` | `t_slef` | sleep finished                                                            |
| `0x00000008` | `t_dcon` | device connected                                                          |
| `0x0000000C` | `t_ddis` | device disconnected                                                       |
| `0x00000018` | `t_dfre` | device read finished                                                      |
| `0x0000001C` | `t_dfwr` | device write finished                                                     |

Notes: 
- `read` means the device reads data from ram and `write` means the device writes data to ram.
- A completed `write` always zeros the whole provided buffer before allowing to write.
- The device may call `d_drre` and` t_drwr` however often it likes
- The device may take as long as it wishes to read/write data. This behavior can be used instead of `d_drre` and` t_drwr`

See [interrupts](instructions.md#interrupts-and-io) for more information and associated commands

# Registers


# Permission Registers
Permission registers may not be modified by the child itself.
Instead they can modified by the parent thread via the `tch_modpr <chid> <ror> <val>` instruction, 
or retrieved by the parent via `tch_getpr <chid> <ror>`