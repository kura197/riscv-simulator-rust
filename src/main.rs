extern crate riscv_simulator;

use std::fs;
use std::env;

use riscv_simulator::*;

const MEMSIZE: usize = 65536;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./{} filename", args[0]);
        return;
    }

    let filename = &args[1];
    let imem = fs::read(filename).expect("Unable to read file");
    let mut dmem: Vec<u8> = vec![0; MEMSIZE];

    //let mut regfile = Regfile::new(52, 4096);
    let mut regfile = Regfile::new(0x1000, 4096);

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
        //println!("{:?}", regfile);
        regfile.add_pc(4);
    }
}
