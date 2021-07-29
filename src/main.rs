extern crate riscv_simulator;

use std::fs;
use std::env;

use riscv_simulator::*;

const MEMSIZE: usize = 65536;
const SPADDR: u32 = 4096;

/// load elf file and write instruction data at secton ".text" and ".data" into vector.
/// only support riscv-test binaries
/// return (pc start addr, mem)
fn load_elf(filename: &str) -> (u32, Vec<u8>) {
    let mut mem: Vec<u8> = vec![0; MEMSIZE];
    let mut start: u32 = 0;

    let buffer = fs::read(filename).expect("Unable to read file");
    match goblin::elf::Elf::parse(&buffer) {
        Ok(binary) => {
            //let e_entry = binary.header.e_entry;
            //println!("enrty : {:X}", e_entry);
            let shdr_strtab = &binary.shdr_strtab;
            for sh in binary.section_headers {
                //println!("{}, {:X}", sh_name, sh.sh_addr);
                let sh_name = &shdr_strtab[sh.sh_name];
                if sh_name == ".text.init" || sh_name == ".text" {
                    start = sh.sh_offset as u32;  
                }

                if sh_name == ".data" || sh_name == ".text.init" || sh_name == ".text" {
                    for idx in 0..sh.sh_size {
                        let offset = sh.sh_offset + idx;
                        mem[offset as usize] = buffer[offset as usize];
                        //print!("{:02X}", buffer[offset as usize]);
                        //if idx % 4 == 3 {
                        //    print!("\n");
                        //}
                    }
                }
            }
        },
        Err(_) => panic!("cannot read elf file."),
    }

    return (start, mem);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./{} filename", args[0]);
        return;
    }

    let (start, mut mem) = load_elf(&args[1]);

    let mut regfile = Regfile::new(start, SPADDR);

    //// run until ECALL or unsupported instructions are issued.
    while regfile.pc+3 < mem.len() as u32 {
        eprintln!("{}", regfile);
        let instr = regfile.get_next_instr(&mem);
        let operand = decode(instr);
        //eprintln!("{:?}", operand);
        let status = execute(&mut regfile, &mut mem, &operand);
        if status == -1 {
            //// exit if ECALL is issued.
            //// test will pass if a0 == 0
            break;
        }
    }
}
