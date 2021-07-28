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

    let mut regfile = Regfile::new(52);

    while regfile.get_pc()+3 < imem.len() as u32 {
        println!("PC: {}", regfile.get_pc());
        let instr = regfile.get_next_instr(&imem);
        //println!("{:08x}", instr);
        let operand = decode(instr);
        println!("{:?}", operand);
        execute(&mut regfile, &mut dmem, &operand);
        println!("{:?}", regfile);
        regfile.inc_pc();
    }
}
