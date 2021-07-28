

pub struct Regfile {
    x: [u32; 32],
    pc: u32
}

impl Regfile {
    pub fn new(pc: u32) -> Regfile {
        let x: [u32; 32] = [0; 32];
        Regfile{
            x, pc
        }
    }

    pub fn get_next_instr(&self, imem: &Vec<u8>) -> u32 {
        let pc = self.pc as usize;
        ((imem[pc+3] as u32) << 24) | ((imem[pc+2] as u32) << 16) | ((imem[pc+1] as u32) << 8) | (imem[pc] as u32)
    }

    pub fn inc_pc(&mut self) {
        self.pc += 4;
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn get_pc(&mut self) -> u32{
        self.pc
    }
}

#[derive(Debug)]
pub struct Operand {
    name: String,
    opcode: u32,
    rd: u32,
    funct3: u32,
    rs1: u32,
    rs2: u32,
    funct7: u32,
    imm: u32,
}

impl Operand {
    fn new(name: &str, opcode: u32, rd: u32, funct3: u32, rs1: u32, rs2: u32, funct7: u32, imm: u32) -> Operand {
        return Operand{
            name: name.to_string(),
            opcode, rd, funct3, rs1, rs2, funct7, imm
        }
    }
}

/// retrieve specific bits from x.
/// return x[shift+nbit-1:shift];
fn retrieve(x: u32, shift: u32, nbit: u32) -> u32 {
    (x >> shift) & ((1 << nbit) - 1)
}

pub fn decode(instr: u32) -> Option<Operand> {
    let opcode: u32 = instr & 0b1111111;
    match opcode {
        0b0110111 => Some(Operand::new("LUI", opcode, 0, 0, 0, 0, 0, retrieve(instr, 12, 20))),
        //TODO: implement
        0b0010111 => Some(Operand::new("AUIPC", opcode, 0, 0, 0, 0, 0, 0)),
        //TODO
        0b1101111 => Some(Operand::new("JAL", opcode, 0, 0, 0, 0, 0, 0)),
        //TODO
        0b1100111 => Some(Operand::new("JALR", opcode, 0, 0, 0, 0, 0, 0)),
        //TODO
        0b1100011 => Some(Operand::new("BRANCH", opcode, 0, 0, 0, 0, 0, 0)),
        0b0000011 => Some(Operand::new("LOAD", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), 0, 0, retrieve(instr, 20, 12))),
        //TODO
        0b0100011 => Some(Operand::new("STORE", opcode, 0, 0, 0, 0, 0, 0)),
        0b0010011 => Some(Operand::new("OP-IMM", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), 0, 0, retrieve(instr, 20, 12))),
        //TODO
        0b0001111 => Some(Operand::new("MISC-MEM", opcode, 0, 0, 0, 0, 0, 0)),
        //TODO
        0b1110011 => Some(Operand::new("SYSTEM", opcode, 0, 0, 0, 0, 0, 0)),
        _ => {
            println!("not supported.");
            None
        },
    }
}
