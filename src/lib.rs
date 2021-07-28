
#[derive(Debug)]
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
        let pc = (self.pc as i32) + 4;
        self.pc = pc as u32;
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

fn sign_extend(val: u32, nbit: u32) -> u32 {
    if (val >> nbit) & 1 == 1 {
        0xFFFFFFFF & !((1 << nbit) - 1) & val
    } else {
        val
    }
}

pub fn decode(instr: u32) -> Operand {
    let opcode: u32 = instr & 0b1111111;
    match opcode {
        0b0110111 => Operand::new("LUI", opcode, 0, 0, 0, 0, 0, retrieve(instr, 12, 20)),
        //TODO: implement
        0b0010111 => Operand::new("AUIPC", opcode, 0, 0, 0, 0, 0, 0),
        //TODO
        0b1101111 => Operand::new("JAL", opcode, 0, 0, 0, 0, 0, 0),
        0b1100111 => Operand::new("JALR", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), 0, 0, retrieve(instr, 20, 12)),
        //TODO
        0b1100011 => Operand::new("BRANCH", opcode, 0, 0, 0, 0, 0, 0),
        0b0000011 => Operand::new("LOAD", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), 0, 0, retrieve(instr, 20, 12)),
        0b0100011 => Operand::new("STORE", opcode, 0, retrieve(instr, 12, 3), retrieve(instr, 15, 5), retrieve(instr, 20, 5), 0, (retrieve(instr, 25, 7) << 5 ) | retrieve(instr, 7, 5)),
        0b0010011 => Operand::new("OP-IMM", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), 0, 0, retrieve(instr, 20, 12)),
        0b0110011 => Operand::new("OP", opcode, retrieve(instr, 7, 5), retrieve(instr, 12, 3), retrieve(instr, 15, 5), retrieve(instr, 20, 5), retrieve(instr, 25, 7), retrieve(instr, 20, 12)),
        //TODO
        0b0001111 => Operand::new("MISC-MEM", opcode, 0, 0, 0, 0, 0, 0),
        //TODO
        0b1110011 => Operand::new("SYSTEM", opcode, 0, 0, 0, 0, 0, 0),
        _ => {
            panic!("opcode {} is not supported.", opcode);
        },
    }
}

pub fn execute(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) {
    match operand.name.as_str() {
        "JALR" => execute_jalr(regfile, operand),
        "LOAD" => execute_load(regfile, dmem, operand),
        "STORE" => execute_store(regfile, dmem, operand),
        "OP-IMM" => execute_op_imm(regfile, operand),
        "OP" => execute_op(regfile, operand),
        _ => panic!("operand name {} is not supported", operand.name),
    }
}

fn execute_jalr(regfile: &mut Regfile, operand: &Operand) {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = (regfile.x[rs1] + imm) as i32;
    let addr = addr & !0b1;

    if rd != 0 {
        regfile.x[rd] = regfile.get_pc() + 4;
    }

    // 4 will be added later.
    //regfile.set_pc(addr-4);
    regfile.set_pc((addr - 4) as u32);
}

fn execute_load(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = (regfile.x[rs1] + imm) as usize;
    match operand.funct3 {
        //// TODO: check endian
        0b000 => regfile.x[rd] = sign_extend(dmem[addr] as u32, 8), // LB
        0b001 => {  // LH
            if addr % 2 != 0 {
                panic!("addr {} need to be aligned by 2.", addr);
            }
            let val: u32 = ((dmem[addr+1] as u32) << 8) | dmem[addr] as u32;
            regfile.x[rd] = sign_extend(val, 16);
        },
        0b010 => {  // LW
            if addr % 4 != 0 {
                panic!("addr {} need to be aligned by 4.", addr);
            }
            regfile.x[rd] = ((dmem[addr+3] as u32) << 24) | ((dmem[addr+2] as u32) << 16) |((dmem[addr+1] as u32) << 8) | dmem[addr] as u32;
        },
        0b100 => regfile.x[rd] = dmem[addr] as u32, // LBU
        0b101 => {  // LHU
            if addr % 2 != 0 {
                panic!("addr {} need to be aligned by 2.", addr);
            }
            regfile.x[rd] = ((dmem[addr+1] as u32) << 8) | dmem[addr] as u32;
        },
        _ => panic!("funct3 {} is not supported.", operand.funct3),
    }
}

fn execute_store(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) {
    let (rs1, rs2) = (operand.rs1 as usize, operand.rs2 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = (regfile.x[rs1] + imm) as usize;
    match operand.funct3 {
        //// TODO: check endian
        0b000 => dmem[addr] = (regfile.x[rs2] & 0xFF) as u8,    // SB
        0b001 => {  // SH
            if addr % 2 != 0 {
                panic!("addr {} need to be aligned by 2.", addr);
            }
            dmem[addr] = (regfile.x[rs2] & 0xFF) as u8;
            dmem[addr+1] = ((regfile.x[rs2] >> 8) & 0xFF) as u8;
        },
        0b010 => {  // SW
            if addr % 4 != 0 {
                panic!("addr {} need to be aligned by 4.", addr);
            }
            dmem[addr] = (regfile.x[rs2] & 0xFF) as u8;
            dmem[addr+1] = ((regfile.x[rs2] >> 8) & 0xFF) as u8;
            dmem[addr+2] = ((regfile.x[rs2] >> 16) & 0xFF) as u8;
            dmem[addr+3] = ((regfile.x[rs2] >> 24) & 0xFF) as u8;
        },
        _ => panic!("funct3 {} is not supported.", operand.funct3),
    }
}

fn execute_op_imm(regfile: &mut Regfile, operand: &Operand) {
    let (rd, rs1, shamt) = (operand.rd as usize, operand.rs1 as usize, operand.rs2 as usize);
    let imm = sign_extend(operand.imm, 12);
    match operand.funct3 {
        0b000 => regfile.x[rd] = (regfile.x[rs1] as i32 + imm as i32) as u32,  // ADDI
        _ => panic!("funct3 {} is not supported.", operand.funct3),
    }
}

fn execute_op(regfile: &mut Regfile, operand: &Operand) {
    let (rd, rs1, rs2) = (operand.rd as usize, operand.rs1 as usize, operand.rs2 as usize);
    match (operand.funct7, operand.funct3) {
        (0b0000000, 0b000) => regfile.x[rd] = (regfile.x[rs1] as i32 + regfile.x[rs2] as i32) as u32,  // ADD
        (0b0100000, 0b000) => regfile.x[rd] = (regfile.x[rs1] as i32 - regfile.x[rs2] as i32) as u32,  // SUB
        (0b0000000, 0b001) => regfile.x[rd] = regfile.x[rs1] << (regfile.x[rs2] & 0b11111),  // SLL
        (0b0000000, 0b100) => regfile.x[rd] = regfile.x[rs1] ^ regfile.x[rs2],  // XOR
        (0b0000000, 0b101) => regfile.x[rd] = regfile.x[rs1] >> (regfile.x[rs2] & 0b11111),  // SLL
        (0b0000000, 0b110) => regfile.x[rd] = regfile.x[rs1] | regfile.x[rs2],  // OR
        (0b0000000, 0b111) => regfile.x[rd] = regfile.x[rs1] & regfile.x[rs2],  // AND
        _ => panic!("(funct7, funct3) ({}, {}) is not supported.", operand.funct7, operand.funct3),
    }
}