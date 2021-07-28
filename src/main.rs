use std::fs;
use std::env;

fn get_next_instr(imem: &Vec<u8>, pc: u32) -> u32 {
    let pc = pc as usize;
    ((imem[pc+3] as u32) << 24) | ((imem[pc+2] as u32) << 16) | ((imem[pc+1] as u32) << 8) | (imem[pc] as u32)
}

struct Regfile {
    x: [u32; 32],
    pc: u32
}

impl Regfile {
    fn new() -> Regfile {
        let x: [u32; 32] = [0; 32];
        let pc: u32 = 0;
        Regfile{
            x, pc
        }
    }

    fn inc_pc(&mut self) {
        self.pc += 4;
    }
}

fn decode(regfile: &mut Regfile, instr: u32) {
    regfile.x[0] = instr;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ./{} filename", args[0]);
        return;
    }

    let filename = &args[1];
    let imem = fs::read(filename).expect("Unable to read file");

    let mut regfile = Regfile::new();
    while regfile.pc+3 < imem.len() as u32 {
        let instr = get_next_instr(&imem, regfile.pc);
        println!("{}", instr);
        decode(&mut regfile, instr);
        regfile.inc_pc();
    }
}
