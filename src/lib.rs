use std::num::Wrapping;
use std::fmt;

#[derive(Debug)]
pub struct Regfile {
    pub x: [u32; 32],
    pub pc: u32,
}

impl Regfile {
    pub fn new(pc: u32, sp: u32) -> Regfile {
        let mut x: [u32; 32] = [0; 32];
        x[0] = 0;
        x[2] = sp;
        Regfile{
            x: x, 
            pc: pc,
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

impl fmt::Display for Regfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pc: {:08X}, x0: {:08X}, ra: {:08X}, sp: {:08X},
                 \rgp: {:08X}, tp: {:08X}, t0: {:08X}, s0: {:08X}, 
                 \ra0: {:08X}, a1: {:08X}, a2: {:08X}, a3: {:08X}, 
                 \ra4: {:08X}, a5: {:08X}, a6: {:08X}, a7: {:08X}\n", 
                    self.pc, self.x[0], self.x[1], self.x[2], 
                    self.x[3], self.x[4], self.x[5], self.x[8], 
                    self.x[10], self.x[11], self.x[12], self.x[13], 
                    self.x[14], self.x[15], self.x[16], self.x[17])
    }
}

#[derive(Debug)]
enum InstrKind {
    LUI,
    AUIPC,
    JAL,
    JALR,
    BRANCH,
    LOAD,
    STORE,
    OPIMM,
    OP,
    MISCMEM,
    SYSTEM,
}

#[derive(Debug)]
pub struct Operand {
    kind: InstrKind,
    opcode: u32,
    rd: u32,
    funct3: u32,
    rs1: u32,
    rs2: u32,
    funct7: u32,
    imm: u32,
}

impl Operand {
    fn new(kind: InstrKind, opcode: u32, rd: u32, funct3: u32, rs1: u32, rs2: u32, funct7: u32, imm: u32) -> Operand {
        return Operand{
            kind, opcode, rd, funct3, rs1, rs2, funct7, imm
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
    let kind: InstrKind;
    let imm: u32;
    match opcode {
        0b0110111 => {
            kind = InstrKind::LUI;
            imm = retrieve(instr, 12, 20) << 12;
        },
        0b0010111 => {
            kind = InstrKind::AUIPC;
            imm = retrieve(instr, 12, 20) << 12;
        },
        0b1101111 => {
            kind = InstrKind::JAL;
            imm = (retrieve(instr, 31, 1) << 20) | (retrieve(instr, 12, 8) << 12) | (retrieve(instr, 20, 1) << 11) | (retrieve(instr, 21, 10) << 1);
        },
        0b1100111 => {
            kind = InstrKind::JALR;
            imm = retrieve(instr, 20, 12);
        },
        0b1100011 => {
            kind = InstrKind::BRANCH;
            imm = (retrieve(instr, 31, 1) << 12) | (retrieve(instr, 7, 1) << 11) | (retrieve(instr, 25, 6) << 5) | (retrieve(instr, 8, 4) << 1);
        },
        0b0000011 => {
            kind = InstrKind::LOAD;
            imm = retrieve(instr, 20, 12);
        },
        0b0100011 => {
            kind = InstrKind::STORE;
            imm = (retrieve(instr, 25, 7) << 5 ) | retrieve(instr, 7, 5);
        },
        0b0010011 => {
            kind = InstrKind::OPIMM;
            imm = retrieve(instr, 20, 12);
        }
        0b0110011 => {
            kind = InstrKind::OP;
            imm = retrieve(instr, 20, 12);
        }
        0b0001111 => {
            kind = InstrKind::MISCMEM;
            imm = 0;
        }
        0b1110011 => {
            kind = InstrKind::SYSTEM;
            imm = retrieve(instr, 15, 5);
        }
        _ => {
            panic!("opcode {} is not supported.", opcode);
        },
    }

    Operand::new(kind, opcode, rd, funct3, rs1, rs2, funct7, imm)
}

pub fn execute(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) -> i64 {
    //// set x0 to 0
    regfile.x[0] = 0;
    match &operand.kind {
        InstrKind::LUI => execute_lui(regfile, operand),
        InstrKind::AUIPC => execute_auipc(regfile, operand),
        InstrKind::JAL => execute_jal(regfile, operand),
        InstrKind::JALR => execute_jalr(regfile, operand),
        InstrKind::BRANCH => execute_branch(regfile, operand),
        InstrKind::LOAD => execute_load(regfile, dmem, operand),
        InstrKind::STORE => execute_store(regfile, dmem, operand),
        InstrKind::OPIMM => execute_op_imm(regfile, operand),
        InstrKind::OP => execute_op(regfile, operand),
        InstrKind::MISCMEM => {
            //// skip FENCE operation
            regfile.add_pc(4);
            0
        },
        InstrKind::SYSTEM => execute_system(regfile, dmem, operand),
        //_ => panic!("operand name {:?} is not supported", operand.kind),
    }
}

fn execute_lui(regfile: &mut Regfile, operand: &Operand) -> i64 {
    let rd = operand.rd as usize;
    regfile.x[rd] = operand.imm;
    regfile.add_pc(4);
    return 0;
}

fn execute_auipc(regfile: &mut Regfile, operand: &Operand) -> i64 {
    let rd = operand.rd as usize;
    let pc = wadd(regfile.pc, operand.imm);
    regfile.x[rd] = pc;
    regfile.add_pc(4);
    return 0;
}

fn execute_jal(regfile: &mut Regfile, operand: &Operand) -> i64 {
    let rd = operand.rd as usize;
    let imm = sign_extend(operand.imm, 20);

    if rd != 0 {
        regfile.x[rd] = wadd(regfile.pc, 4);
    }

    regfile.add_pc(imm);
    return 0;
}

fn execute_jalr(regfile: &mut Regfile, operand: &Operand) -> i64 {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = wadd(regfile.x[rs1], imm);
    let addr = addr & !0b1;

    if rd != 0 {
        regfile.x[rd] = regfile.pc + 4;
    }

    regfile.pc = addr;
    return 0;
}

fn execute_branch(regfile: &mut Regfile, operand: &Operand) -> i64 {
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
        regfile.add_pc(imm);
    } else {
        regfile.add_pc(4);
    }

    return 0;
}

fn execute_load(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) -> i64 {
    let (rd, rs1) = (operand.rd as usize, operand.rs1 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = wadd(regfile.x[rs1], imm) as usize;
    match operand.funct3 {
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
    regfile.add_pc(4);
    return 0;
}

fn execute_store(regfile: &mut Regfile, dmem: &mut Vec<u8>, operand: &Operand) -> i64 {
    let (rs1, rs2) = (operand.rs1 as usize, operand.rs2 as usize);
    let imm = sign_extend(operand.imm, 12);
    let addr = wadd(regfile.x[rs1], imm) as usize;
    match operand.funct3 {
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
    regfile.add_pc(4);
    return 0;
}

fn execute_op_imm(regfile: &mut Regfile, operand: &Operand) -> i64 {
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
    regfile.add_pc(4);
    return 0;
}

fn execute_op(regfile: &mut Regfile, operand: &Operand) -> i64 {
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
    regfile.add_pc(4);
    return 0;
}

fn execute_system(regfile: &mut Regfile, _dmem: &mut Vec<u8>, operand: &Operand) -> i64 {
    regfile.add_pc(4);
    //TODO: implement other instrs
    match (operand.funct7, operand.funct3) {
        (0b0000000, 0b000) => { // ECALL
            println!("a0 : {:08X}", regfile.x[10]); 
            -1
        },
        _ => 0,
    }
}
