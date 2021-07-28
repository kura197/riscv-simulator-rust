use std::num::Wrapping;

#[derive(Debug)]
pub struct Regfile {
    pub x: [u32; 32],
    pub pc: u32
}

impl Regfile {
    pub fn new(pc: u32, sp: u32) -> Regfile {
        let mut x: [u32; 32] = [0; 32];
        x[2] = sp;
        Regfile{
            x, pc
        }
    }

    pub fn get_next_instr(&self, imem: &Vec<u8>) -> u32 {
        let pc = self.pc as usize;
        ((imem[pc+3] as u32) << 24) | ((imem[pc+2] as u32) << 16) | ((imem[pc+1] as u32) << 8) | (imem[pc] as u32)
    }

    pub fn add_pc(&mut self, val: u32) {
        self.pc = wadd(self.pc, val);
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
    if (val >> (nbit-1)) & 1 == 1 {
        (0xFFFFFFFF & !((1 << nbit) - 1)) | val
    } else {
        val
    }
}

fn wadd(a: u32, b: u32) -> u32 {
    (Wrapping(a) + Wrapping(b)).0
}

fn wsub(a: u32, b: u32) -> u32 {
    (Wrapping(a) - Wrapping(b)).0
}

pub fn decode(instr: u32) -> Operand {
    let opcode = retrieve(instr, 0, 7);
    let rd = retrieve(instr, 7, 5);
    let funct3 = retrieve(instr, 12, 3);
    let rs1 = retrieve(instr, 15, 5);
    let rs2 = retrieve(instr, 20, 5);
    let funct7 = retrieve(instr, 25, 7);
    let name: String;
    let imm: u32;
    match opcode {
        0b0110111 => {
            name = "LUI".to_string();
            imm = retrieve(instr, 12, 20) << 12;
        },
        0b0010111 => {
            name = "AUIPC".to_string();
            imm = retrieve(instr, 12, 20) << 12;
        },
        0b1101111 => {
            name = "JAL".to_string();
            imm = (retrieve(instr, 31, 1) << 20) | (retrieve(instr, 12, 8) << 12) | (retrieve(instr, 20, 1) << 11) | (retrieve(instr, 21, 10) << 1);
        },
        0b1100111 => {
            name = "JALR".to_string();
            imm = retrieve(instr, 20, 12);
        },
        0b1100011 => {
            name = "BRANCH".to_string();
            imm = (retrieve(instr, 31, 1) << 12) | (retrieve(instr, 7, 1) << 11) | (retrieve(instr, 25, 6) << 5) | (retrieve(instr, 8, 4) << 1);
        },
        0b0000011 => {
            name = "LOAD".to_string();
            imm = retrieve(instr, 20, 12);
        },
        0b0100011 => {
            name = "STORE".to_string();
            imm = (retrieve(instr, 25, 7) << 5 ) | retrieve(instr, 7, 5);
        },
        0b0010011 => {
            name = "OP-IMM".to_string();
            imm = retrieve(instr, 20, 12);
        }
        0b0110011 => {
            name = "OP".to_string();
            imm = retrieve(instr, 20, 12);
        }
        0b0001111 => {
            name = "MISC-MEM".to_string();
            imm = 0;
        }
        0b1110011 => {
            name = "SYSTEM".to_string();
            imm = retrieve(instr, 15, 5);
        }
        _ => {
            panic!("opcode {} is not supported.", opcode);
        },
    }

    Operand::new(&name, opcode, rd, funct3, rs1, rs2, funct7, imm)
}

pub fn execute(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) {
    match operand.name.as_str() {
        "LUI" => execute_lui(regfile, operand),
        "AUIPC" => execute_auipc(regfile, operand),
        "JAL" => execute_jal(regfile, operand),
        "JALR" => execute_jalr(regfile, operand),
        "BRANCH" => execute_branch(regfile, operand),
        "LOAD" => execute_load(regfile, dmem, operand),
        "STORE" => execute_store(regfile, dmem, operand),
        "OP-IMM" => execute_op_imm(regfile, operand),
        "OP" => execute_op(regfile, operand),
        "MISC-MEM" => {
            //// skip FENCE operation
        },
        "SYSTEM" => execute_system(regfile, dmem, operand),
        _ => panic!("operand name {} is not supported", operand.name),
    }
}

fn execute_lui(regfile: &mut Regfile, operand: &Operand) {
    let rd = operand.rd as usize;
    regfile.x[rd] = operand.imm;
}

fn execute_auipc(regfile: &mut Regfile, operand: &Operand) {
    let rd = operand.rd as usize;
    let pc = wsub(wadd(regfile.pc, operand.imm), 4);
    regfile.pc = pc;
    //TODO: correct?
    regfile.x[rd] = pc;
}

fn execute_jal(regfile: &mut Regfile, operand: &Operand) {
    let rd = operand.rd as usize;
    let imm = sign_extend(operand.imm, 20);

    if rd != 0 {
        regfile.x[rd] = wadd(regfile.pc, 4);
    }

    regfile.add_pc(wsub(imm, 4));
}

fn execute_jalr(regfile: &mut Regfile, operand: &Operand) {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = wadd(regfile.x[rs1], imm);
    let addr = addr & !0b1;

    if rd != 0 {
        regfile.x[rd] = regfile.pc + 4;
    }

    // 4 will be added later.
    regfile.pc = wsub(addr, 4);
}

fn execute_branch(regfile: &mut Regfile, operand: &Operand) {
    let (rs1, rs2) = (operand.rs1 as usize, operand.rs2 as usize);
    let imm = sign_extend(operand.imm, 12);
    let branch: bool;
    match operand.funct3 {
        0b000 => branch = regfile.x[rs1] == regfile.x[rs2], // BEQ
        0b001 => branch = regfile.x[rs1] != regfile.x[rs2], // BEQ
        0b100 => branch = (regfile.x[rs1] as i32) < (regfile.x[rs2] as i32), // BLT
        0b101 => branch = (regfile.x[rs1] as i32) >= (regfile.x[rs2] as i32), // BGE
        0b110 => branch = regfile.x[rs1] < regfile.x[rs2], // BLTU
        0b111 => branch = regfile.x[rs1] >= regfile.x[rs2], // BGEU
        _ => panic!("funct3 {} is not supported.", operand.funct3),
    }

    if branch {
        regfile.add_pc(wsub(imm, 4));
    }
}

fn execute_load(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = wadd(regfile.x[rs1], imm) as usize;
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
    let addr = wadd(regfile.x[rs1], imm) as usize;
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
        0b000 => regfile.x[rd] = wadd(regfile.x[rs1], imm),  // ADDI
        0b010 => regfile.x[rd] = ((regfile.x[rs1] as i32) < (imm as i32)) as u32,  // SLTI
        0b011 => regfile.x[rd] = (regfile.x[rs1] < imm) as u32,  // SLTIU
        0b100 => regfile.x[rd] = regfile.x[rs1] ^ imm,  // XORI
        0b110 => regfile.x[rd] = regfile.x[rs1] | imm,  // ORI
        0b111 => regfile.x[rd] = regfile.x[rs1] & imm,  // ANDI
        0b001 => regfile.x[rd] = regfile.x[rs1] << shamt, // SLLI
        0b101 => {
            match operand.funct7 {
                0b0000000 => regfile.x[rd] = regfile.x[rs1] >> shamt, // SRLI
                0b0100000 => regfile.x[rd] = ((regfile.x[rs1] as i32) >> shamt) as u32, // SRLI
                _ => panic!("funct7 {} is not supported.", operand.funct7),
            }
        },
        _ => panic!("funct3 {} is not supported.", operand.funct3),
    }
}

fn execute_op(regfile: &mut Regfile, operand: &Operand) {
    let (rd, rs1, rs2) = (operand.rd as usize, operand.rs1 as usize, operand.rs2 as usize);
    match (operand.funct7, operand.funct3) {
        (0b0000000, 0b000) => regfile.x[rd] = wadd(regfile.x[rs1], regfile.x[rs2]),  // ADD
        (0b0100000, 0b000) => regfile.x[rd] = wsub(regfile.x[rs1], regfile.x[rs2]),  // SUB
        (0b0000000, 0b001) => regfile.x[rd] = regfile.x[rs1] << (regfile.x[rs2] & 0b11111),  // SLL
        (0b0000000, 0b010) => regfile.x[rd] = ((regfile.x[rs1] as i32) < (regfile.x[rs2] as i32)) as u32,  // SLT
        (0b0000000, 0b011) => regfile.x[rd] = (regfile.x[rs1] < regfile.x[rs2]) as u32,  // SLTU
        (0b0000000, 0b100) => regfile.x[rd] = regfile.x[rs1] ^ regfile.x[rs2],  // XOR
        (0b0000000, 0b101) => regfile.x[rd] = regfile.x[rs1] >> (regfile.x[rs2] & 0b11111),  // SRL
        (0b0100000, 0b101) => regfile.x[rd] = ((regfile.x[rs1] as i32) >> (regfile.x[rs2] & 0b11111)) as u32,  // SRA
        (0b0000000, 0b110) => regfile.x[rd] = regfile.x[rs1] | regfile.x[rs2],  // OR
        (0b0000000, 0b111) => regfile.x[rd] = regfile.x[rs1] & regfile.x[rs2],  // AND
        _ => panic!("(funct7, funct3) ({}, {}) is not supported.", operand.funct7, operand.funct3),
    }
}

fn execute_system(_regfile: &mut Regfile, _dmem: &mut Vec<u8>, _operand: &Operand) {
    ////TODO: implement
}
