use std::{collections::HashMap, path::Path, fs::File, io::{BufReader, Read, BufRead, Write}, rc::Rc, cell::RefCell, sync::{atomic::AtomicBool, Mutex}, borrow::Borrow, process::exit};
use crate::machine::{Machine, REG_I};

pub struct Debugger<'a> {
    line_mappings: HashMap<u32, u32>,
    machine: &'a mut Machine,
    source: Option<Vec<String>>,
}


static SYSTEMS_INITIALIZED: Mutex<bool> = Mutex::new(false);
static CTRLC_DO_QUIT: Mutex<bool> = Mutex::new(true);
static CTRLC_ABORT: Mutex<bool> = Mutex::new(false);

impl<'a> Debugger<'a> {
    pub fn new(machine: &'a mut Machine) -> Self{
        Self::raw_new::<&str, &str>(machine, None, None)
    }

    pub fn with_debug_info_file<P: AsRef<Path>>(machine: &'a mut Machine, dfile: P) -> Self{
        Self::raw_new::<_, &str>(machine, Some(dfile), None)
    }

    pub fn with_debug_info_and_source<P1: AsRef<Path>, P2: AsRef<Path>>(machine: &'a mut Machine, dfile: P1, sfile: P2) -> Self{
        Self::raw_new(machine, Some(dfile), Some(sfile))
    }

    fn raw_new<P1: AsRef<Path>, P2: AsRef<Path>>(machine: &'a mut Machine, dfile: Option<P1>, sfile: Option<P2>) -> Self{
        if !*SYSTEMS_INITIALIZED.lock().unwrap() {
            *SYSTEMS_INITIALIZED.lock().unwrap() = true;

            ctrlc::set_handler(||{
                if *CTRLC_DO_QUIT.lock().unwrap() {
                    println!("Exited via CTRL+C");
                    exit(0);
                }
                *CTRLC_ABORT.lock().unwrap() = true;
            }).unwrap();
        }

        let mut line_mappings: HashMap<u32, u32> = Default::default();

        if let Some(df) = dfile {
            let debug_info = File::open(df).unwrap();
            let mut reader = BufReader::new(debug_info);
            let mut buf = vec![0u8;8];
            while reader.read_exact(&mut buf).is_ok() {
                let addr = u32::from_be_bytes(buf[0..4].try_into().unwrap());
                let line = u32::from_be_bytes(buf[4..8].try_into().unwrap());
                line_mappings.insert(addr, line);
            }
        }

        Self { 
            line_mappings,
            machine,
            source: sfile.map(|sf|{
                let mut lines = vec![];
                let source = File::open(sf).unwrap();
                let mut reader = BufReader::new(source);
                while reader.has_data_left().unwrap() {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    lines.push(line);
                }
                lines
            })
        }
    }

    pub fn run(&mut self) {
        loop {
            self.handle_breaked();
        }
    }

    fn handle_breaked(&mut self) {
        let addr = self.machine.registers[REG_I];
        let raw = self.machine.read_word(addr);
        let instr = raw >> 21;

        let instr_formatted = if instr == 0b111_11111111 {
            format!("[breakpoint \"{}{}\"]", char::from((raw >> 8) as u8), char::from(raw as u8))
        } else {
            let arg0 = (raw >> 14 & 0b01111111) as u8;
            let arg1 = (raw >> 7 & 0b01111111) as u8;
            let arg2 = (raw & 0b01111111) as u8;
            let (s0, v0) = self.resolve_arg(arg0);
            let (s1, v1) = self.resolve_arg(arg1);
            let (s2, v2) = self.resolve_arg(arg2);
            format!("[{}=0x{:08X} {}=0x{:08X} {}=0x{:08X}]", s0, v0, s1, v1, s2, v2)
        };

        if let Some(line) = self.line_mappings.get(&addr) {
            println!("address 0x{:08X} line {} {}", addr, line + 1, instr_formatted);
            if let Some(lines) = &self.source {
                println!("{:3} | {}", line + 1, lines[*line as usize]);
            }
        } else {
            println!("address 0x{:08X} <no line mapping> {}", addr, instr_formatted)
        }

        self.input_command();
    }

    fn input_command(&mut self) {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        
        let mut help = false;
        let mut print_regs = false;
        let mut print_mem = None;
        let mut n = 1u32;
        let mut breakpoint = false;
        let mut breakpoint_tag = None;

        let r: Result<(), String> = try {
            for arg in buffer.split(',') {
                let arg = arg.trim();
                if arg.len() == 0 { continue; }
                if let Some((a, b)) = arg.split_once('=') {
                    let a = a.trim();
                    let b = b.trim();
                    if a.len() == 0 || b.len() == 0 {
                        Err(format!("Arg formated invalidly `{arg}`"))?;
                    }
                    match a {
                        "help" => help = b.parse().map_err(|_| format!("could not parse bool for `help={b}`"))?,
                        "p" | "print" => {
                            print_regs = b == "regs";
                            if b.starts_with("mem:") {
                                if let Some((start_s, len_s)) = b.split_once(':').unwrap().1.split_once('+') {
                                    let start = string_to_u32(b.trim().to_string()).map_err(|_| format!("could not parse start in `{arg}` for `print=mem:start+len`"))?;
                                    let len = string_to_u32(b.trim().to_string()).map_err(|_| format!("could not parse len in `{arg}` for `print=mem:start+len`"))?;
                                    print_mem = Some((start, len));
                                } else {
                                    Err(format!("Arg formated invalidly `{arg}` for `print=mem:start+len`"))?;
                                }
                            }
                        },
                        "n" => n = string_to_u32(b.to_string()).map_err(|_| format!("could not parse u32 for `n={b}`"))?,
                        "bp" | "breakpoint" => breakpoint = b.parse().map_err(|_| format!("could not parse bool for `breakpoint={b}`"))?,
                        "bpt" | "breakpoint_tag" => breakpoint_tag = Some(b),
                        _ => Err(format!("Invalid arg `{arg}`"))?
                    }
                } else {
                    Err(format!("expected `arg=<val>`, got `{arg}`. try `help=true` for more info"))?;
                }
            }
        };
        match r {
            Ok(()) => {
                if help {
                    println!("help=<bool>          - show this menu");
                    println!("print=regs           - print registers");
                    println!("print=mem:start+len  - prints a slice of memory[start..start+len]");
                    println!("n=<u32>              - repeat n times before interrupting");
                    println!("breakpoint=<bool>    - whether to interrupt at next breakpoint");
                    println!("breakpoint_tag=<str> - restricts breaking to a certain tag");
                }

                if print_regs {
                    println!("{:016X?}", self.machine.registers);
                }

                if let Some((start, len)) = print_mem {
                    println!("{:016X?}", &self.machine.memory[start as usize..start as usize+len as usize]);
                }

                *CTRLC_ABORT.lock().unwrap() = false;
                *CTRLC_DO_QUIT.lock().unwrap() = false;
                'abort: for i in 0..n {
                    loop {
                        if *CTRLC_ABORT.lock().unwrap() {
                            println!("CTRL+C: interrupting early after {i} out of {n} loops!");
                            println!();
                            break 'abort;
                        }

                        self.machine.execute_next();

                        let addr = self.machine.registers[REG_I];
                        let raw = self.machine.read_word(addr);
                        let instr = raw >> 21;

                        if !breakpoint {
                            break;
                        }

                        if breakpoint && instr == 0b111_11111111 {
                            if let Some(bpt) = breakpoint_tag {
                                let tag = String::from_utf8(vec![(raw >> 8) as u8, raw as u8]).unwrap();
                                if tag == bpt {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                *CTRLC_DO_QUIT.lock().unwrap() = true;
            },
            Err(e) => println!("invalid arg: {e}")
        }

    }

    fn resolve_arg(&mut self, arg: u8) -> (String, u32) {
        let regs = self.machine.registers.clone();
        let data = self.machine.fetch_data(arg);
        self.machine.registers.clone_from(&regs);
        (match arg {
            0b01111111 => "literal".to_owned(),
            0b01000000 => "!".to_owned(),
            _ => format!("%{}", if arg < 48 { format!("{arg:02X}") } else { ["S", "I", "L", "C", "F", "Q"][arg as usize-48].to_owned() })
        }, data)
    }
}

pub(crate) fn string_to_u32(mut num: String) -> Result<u32, String>{
    num = num.replace('_', "");
    let radix = if num.len() > 2 {
        if num.chars().nth(0).unwrap() == '0' {
            let r = match num.chars().nth(1).unwrap() {
                'b' => Some(0b10), // binary
                'q' => Some(4),    // quaternal
                'o' => Some(0o10), // octal
                'z' => Some(12),   // dozenal
                'x' => Some(0x10), // hexadecimal
                _ => None         // decimal (or invalid)
            };
            if let Some(r) = r {
                num.remove(0);
                num.remove(0);
                r
            } else { 10 }
        } else { 10 }
    } else { 10 };

    u32::from_str_radix(&num, radix).map_err(|_|format!("Invalid u32 '{num}'"))
}