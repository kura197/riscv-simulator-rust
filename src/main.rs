extern crate riscv_simulator;

use std::fs;
use std::env;

use riscv_simulator::*;

const MEMSIZE: usize = 65536;
const INITADDR: u32 = 0x1000;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./{} filename", args[0]);
        return;
    }

    let filename = &args[1];
    let imem = fs::read(filename).expect("Unable to read file");
    let mut dmem: Vec<u8> = vec![0; MEMSIZE];
    //// TODO: read elf format
    //// for lw test
    dmem[0x2000] = 0xFF;
    dmem[0x2002] = 0xFF;
    dmem[0x2005] = 0xFF;
    dmem[0x2007] = 0xFF;
    dmem[0x2008] = 0xF0;
    dmem[0x2009] = 0x0F;
    dmem[0x200a] = 0xF0;
    dmem[0x200b] = 0x0F;
    dmem[0x200c] = 0x0F;
    dmem[0x200d] = 0xF0;
    dmem[0x200e] = 0x0F;
    dmem[0x200f] = 0xF0;

    //let mut regfile = Regfile::new(52, 4096);
    let mut regfile = Regfile::new(INITADDR, 4096);

    while regfile.pc+3 < imem.len() as u32 {
        eprintln!("PC: {:x}", regfile.pc);
        let instr = regfile.get_next_instr(&imem);
        //println!("{:08X}", instr);
        let operand = decode(instr);
        eprintln!("{:?}", operand);
        let status = execute(&mut regfile, &mut dmem, &operand);
        if status == -1 {
            break;
        }
        eprintln!("{:?}", regfile);
        regfile.add_pc(4);
    }
}
