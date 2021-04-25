use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use anyhow::{anyhow, Result};
use crate::cart;
use crate::ppu;


pub struct CPU {

    //registers
    r_pc: u16,
    r_a: u8,
    r_x: u8,
    r_y: u8,
    r_sp: u8,
    r_st: u8,

    cycles: usize,
    skip_cycles: usize,

    frame_ready: bool,

    ram: Vec<u8>,
    cart: Rc<RefCell<cart::Cartridge>>,
    ppu: Rc<RefCell<ppu::PPU>>

}


impl CPU {
    pub fn new(cart: Rc<RefCell<cart::Cartridge>>, ppu: Rc<RefCell<ppu::PPU>>) -> Self {
        CPU {
            r_pc: 0,
            r_a: 0,
            r_x: 0,
            r_y: 0,
            r_sp: 0,
            r_st: 0,
            cycles: 0,
            skip_cycles: 0,
            frame_ready: false,
            ram: Vec::with_capacity(2048),
            cart: cart,
            ppu: ppu
        }
    }

    pub fn interrupt(tp: InteruptType) {

    }
    pub fn is_frame_ready(&self) -> bool {
        self.frame_ready
    }
    pub fn init(&mut self) {
        self.reset();
    }
    pub fn reset(&mut self) -> Result<()> {

        self.r_pc = self.read_address(RESET_VECTOR)?;
        self.r_a = 0;
        self.r_x = 0;
        self.r_y = 0;
        self.r_st = STATUS_START;
        //to be confirmed
        self.r_sp = STACK_START;

        self.cycles = 0;
        self.skip_cycles = 0;
        Ok(())
    }
    
    fn read_address(&self, addr: u16) -> Result<u16> {
        let addrl: u16 = self.read(addr)? as u16;
        let addrh: u16 = self.read(addr+ 1)? as u16;
        Ok((addrh << 8) | addrl)
    }

    pub fn step(&mut self) -> Result<()>{
        self.cycles += 1;
        if self.skip_cycles > 0 {
            self.skip_cycles -= 1;
            return Ok(());
        }

        let opcode: u8 = self.read(self.r_pc)?;
        self.r_pc += 1;

        if !OP_MAP.contains_key(&opcode) {
            return Err(anyhow!("invalid opcode: {}", opcode));
        }

        let op = OP_MAP.get(&opcode).unwrap();
        debug!("executin {:?}", op.0);
        match op.0 {
            Instructions::ORA => self.exe_ora(op),
            _ => self.just_for_test(op)
        }
    }
    fn just_for_test(&mut self, op: &Op) -> Result<()> {
        Ok(())
    }

    fn write(&mut self, addr: u16, data: u8) -> Result<()> {
        match addr {
            addr if addr < 0x2000 => self.write_ram(addr & 0x07FF, data),
            addr if addr < 0x4000 => self.write_ppu_register(addr & 0x2007, data),
            addr if (addr < 0x4014 || addr == 0x4015) => self.write_apu_register(addr, data),
            0x4014                => self.write_oamdma_addr(data),
            0x4016                => self.write_joy1(data),
            0x4017                => self.write_joy2(data),
            addr if addr < 0x6000 => self.write_unused_addr(addr, data),
            addr if addr < 0x8000 => self.write_cart(addr, data),
            addr => self.write_unused_addr(addr, data)
        }
    }

    fn write_ram(&mut self, addr: u16, data: u8) -> Result<()> {
        debug!("write_cart called, address: {:#06x}, data: {:#04x}", addr, data);
        self.ram[addr as usize] = data;
        Ok(())
    }
    
    fn write_cart(&mut self, addr: u16, data: u8) -> Result<()>{
        debug!("write_cart called, address: {:#06x}, data: {:#04x}", addr, data);
        self.cart.borrow_mut().write_sram(addr, data)
    }
    fn write_joy1(&mut self, data: u8) -> Result<()> {
        debug!("write_joy1 called, data: {:#04x}", data);
        Ok(())
    }
    fn write_joy2(&mut self, data: u8) -> Result<()> {
        debug!("write_joy2 called, data: {:#04x}", data);
        Ok(())
    }
    fn write_unused_addr(&mut self, addr: u16, data: u8) -> Result<()> {
        debug!("write_unused_addr called, address: {:#06x}, data: {:#04x}", addr, data);
        Ok(())
    }
    fn write_ppu_register(&mut self, addr: u16, data: u8) -> Result<()> {
        debug!("write_ppu_register called, address: {:#06x}, data: {:#04x}", addr, data);
        Ok(())
    }
    fn write_apu_register(&mut self, addr: u16, data: u8) -> Result<()> {
        debug!("write_apu_register called, address: {:#06x}, data: {:#04x}", addr, data);
        Ok(())
    }
    fn write_oamdma_addr(&mut self, data: u8) -> Result<()> {
        debug!("write_oamdma_addr called, data: {:#04x}", data);
        Ok(())
    }

    fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            addr if addr < 0x2000 => self.read_ram(addr & 0x07FF),
            addr if addr < 0x4000 => self.read_ppu_register(addr & 0x2007),
            addr if (addr < 0x4014 || addr == 0x4015) => self.read_apu_register(addr),
            0x4014                => self.read_oamdma_addr(),
            0x4016                => self.read_joy1(),
            0x4017                => self.read_joy2(),
            addr if addr < 0x6000 => self.read_unused_addr(addr),
            addr if addr < 0x8000 => self.read_cart(addr),
            addr => self.read_unused_addr(addr)
        }
    }

    
    fn read_ram(&self, addr: u16) -> Result<u8> {
        debug!("read_ram called, address: {:#06x}", addr);
        Ok(self.ram[addr as usize])
    }
    fn read_cart(&self, addr: u16) -> Result<u8> {
        self.cart.borrow().read_prg(addr)
    }
    fn read_joy1(&self) -> Result<u8> {
        debug!("read_joy1 called");
        Ok(0)
    }
    fn read_joy2(&self) -> Result<u8> {
        debug!("read_joy2 called");
        Ok(0)
    }
    fn read_unused_addr(&self, addr: u16) -> Result<u8> {
        debug!("read_unused_addr called, addr: {:#06x}", addr);
        Ok(0)
    }
    fn read_ppu_register(&self, addr: u16) -> Result<u8> {
        debug!("read_ppu_register called, addr: {:#06x}", addr);
        Ok(0)
    }
    fn read_apu_register(&self, addr: u16) -> Result<u8> {
        debug!("read_apu_register called, addr: {:#06x}", addr);
        Ok(0)
    }
    fn read_oamdma_addr(&self) -> Result<u8> {
        debug!("read_oamdma_addr called");
        Ok(0)
    }

    fn exe_ora(&mut self, op: &Op) -> Result<()> {
        Ok(())
    }
}


#[derive(PartialEq, Clone, Copy, Debug)]
enum AddressMode {
    IMM,ZP,ZPX,ZPY,IZX,IZY,ABS,ABX,ABY,IND,REL,IMP
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InteruptType {
    NMI
}

type OpLen = usize;
type OpCycles = usize;
type Op = (Instructions, AddressMode, OpLen, OpCycles, bool);

#[derive(Debug)]
enum Instructions {
    ORA,AND,EOR,ADC,SBC,LDA,STA,LDX,STX,LDY,STY,
    CMP,CPX,CPY,DEC,INC,
    DEX,DEY,INX,INY,ASL,ROL,LSR,ROR,TAX,TXA,
    TAY,TYA,TSX,TXS,PLA,PHA,PLP,PHP,
    BPL,BMI,BVC,BVS,BCC,BCS,BNE,BEQ,
    CLC,SEC,CLD,SED,CLI,SEI,CLV,
    JSR,JMP,RTS,RTI,BIT,
    BRK
    // SLO,RLA,RRA,SAX,LAX,DCP,ISC,ANC,ALR,ARR,XAA,LAX
}


static STATUS_N: u8 = 0x80;
static STATUS_V: u8 = 0x40;
static STATUS_B: u8 = 0x10;
static STATUS_D: u8 = 0x08;
static STATUS_I: u8 = 0x04;
static STATUS_Z: u8 = 0x02;
static STATUS_C: u8 = 0x01;

static RESET_VECTOR: u16 = 0xFFFC;
static BRK_VECTOR: u16 = 0xFFFE;
static IRQ_VECTOR: u16 = 0xFFFE;
static NMI_VECTOR: u16 = 0xFFFE;
static STATUS_START: u8 = 0x04;
static STACK_START: u8 = 0xFD;

lazy_static! {
    static ref OP_MAP: HashMap<u8, Op> = {
        let mut m = HashMap::new();
        m.insert(0x69, (Instructions::ADC, AddressMode::IMM,  2,  2, false));
        m.insert(0x65, (Instructions::ADC, AddressMode::ZP,  2,  3, false));
        m.insert(0x75, (Instructions::ADC, AddressMode::ZPX,  2,  4, false));
        m.insert(0x6d, (Instructions::ADC, AddressMode::ABS,  3,  4, false));
        m.insert(0x7d, (Instructions::ADC, AddressMode::ABX,  3,  4, true));
        m.insert(0x79, (Instructions::ADC, AddressMode::ABY,  3,  4, true));
        m.insert(0x61, (Instructions::ADC, AddressMode::IZX,  2,  6, false));
        m.insert(0x71, (Instructions::ADC, AddressMode::IZY,  2,  5, true));
        m.insert(0x29, (Instructions::AND, AddressMode::IMM,  2,  2, false));
        m.insert(0x25, (Instructions::AND, AddressMode::ZP,  2,  3, false));
        m.insert(0x35, (Instructions::AND, AddressMode::ZPX,  2,  4, false));
        m.insert(0x2d, (Instructions::AND, AddressMode::ABS,  3,  4, false));
        m.insert(0x3d, (Instructions::AND, AddressMode::ABX,  3,  4, true));
        m.insert(0x39, (Instructions::AND, AddressMode::ABY,  3,  4, true));
        m.insert(0x21, (Instructions::AND, AddressMode::IZX,  2,  6, false));
        m.insert(0x31, (Instructions::AND, AddressMode::IZY,  2,  5, true));
        m.insert(0x0a, (Instructions::ASL, AddressMode::IMP,  1,  2, false));
        m.insert(0x06, (Instructions::ASL, AddressMode::ZP,  2,  5, false));
        m.insert(0x16, (Instructions::ASL, AddressMode::ZPX,  2,  6, false));
        m.insert(0x0e, (Instructions::ASL, AddressMode::ABS,  3,  6, false));
        m.insert(0x1e, (Instructions::ASL, AddressMode::ABX,  3,  7, false));
        m.insert(0x90, (Instructions::BCC, AddressMode::REL,  2,  2, false));
        m.insert(0xb0, (Instructions::BCS, AddressMode::REL,  2,  2, true));
        m.insert(0xf0, (Instructions::BEQ, AddressMode::REL,  2,  2, true));
        m.insert(0x24, (Instructions::BIT, AddressMode::ZP,  2,  3, false));
        m.insert(0x2c, (Instructions::BIT, AddressMode::ABS,  3,  4, false));
        m.insert(0x30, (Instructions::BMI, AddressMode::REL,  2,  2, false));
        m.insert(0xd0, (Instructions::BNE, AddressMode::REL,  2,  2, true));
        m.insert(0x10, (Instructions::BPL, AddressMode::REL,  2,  2, true));
        m.insert(0x00, (Instructions::BRK, AddressMode::IMP,  1,  7, false));
        m.insert(0x50, (Instructions::BVC, AddressMode::REL,  2,  2, true));
        m.insert(0x70, (Instructions::BVS, AddressMode::REL,  2,  2, true));
        m.insert(0x18, (Instructions::CLC, AddressMode::IMP,  1,  2, false));
        m.insert(0xd8, (Instructions::CLD, AddressMode::IMP,  1,  2, false));
        m.insert(0x58, (Instructions::CLI, AddressMode::IMP,  1,  2, false));
        m.insert(0xb8, (Instructions::CLV, AddressMode::IMP,  1,  2, false));
        m.insert(0xc9, (Instructions::CMP, AddressMode::IMM,  2,  2, false));
        m.insert(0xc5, (Instructions::CMP, AddressMode::ZP,  2,  3, false));
        m.insert(0xd5, (Instructions::CMP, AddressMode::ZPX,  2,  4, false));
        m.insert(0xcd, (Instructions::CMP, AddressMode::ABS,  3,  4, false));
        m.insert(0xdd, (Instructions::CMP, AddressMode::ABX,  3,  4, true));
        m.insert(0xd9, (Instructions::CMP, AddressMode::ABY,  3,  4, true));
        m.insert(0xc1, (Instructions::CMP, AddressMode::IZX,  2,  6, false));
        m.insert(0xd1, (Instructions::CMP, AddressMode::IZY,  2,  5, true));
        m.insert(0xe0, (Instructions::CPX, AddressMode::IMM,  2,  2, false));
        m.insert(0xe4, (Instructions::CPX, AddressMode::ZP,  2,  3, false));
        m.insert(0xec, (Instructions::CPX, AddressMode::ABS,  3,  4, false));
        m.insert(0xc0, (Instructions::CPY, AddressMode::IMM,  2,  2, false));
        m.insert(0xc4, (Instructions::CPY, AddressMode::ZP,  2,  3, false));
        m.insert(0xcc, (Instructions::CPY, AddressMode::ABS,  3,  4, false));
        m.insert(0xc6, (Instructions::DEC, AddressMode::ZP,  2,  5, false));
        m.insert(0xd6, (Instructions::DEC, AddressMode::ZPX,  2,  6, false));
        m.insert(0xce, (Instructions::DEC, AddressMode::ABS,  3,  6, false));
        m.insert(0xde, (Instructions::DEC, AddressMode::ABX,  3,  7, false));
        m.insert(0xca, (Instructions::DEX, AddressMode::IMP,  1,  2, false));
        m.insert(0x88, (Instructions::DEY, AddressMode::IMP,  1,  2, false));
        m.insert(0x49, (Instructions::EOR, AddressMode::IMM,  2,  2, false));
        m.insert(0x45, (Instructions::EOR, AddressMode::ZP,  2,  3, false));
        m.insert(0x55, (Instructions::EOR, AddressMode::ZPX,  2,  4, false));
        m.insert(0x4d, (Instructions::EOR, AddressMode::ABS,  3,  4, false));
        m.insert(0x5d, (Instructions::EOR, AddressMode::ABX,  3,  4, true));
        m.insert(0x59, (Instructions::EOR, AddressMode::ABY,  3,  4, true));
        m.insert(0x41, (Instructions::EOR, AddressMode::IZX,  2,  6, false));
        m.insert(0x51, (Instructions::EOR, AddressMode::IZY,  2,  5, true));
        m.insert(0xe6, (Instructions::INC, AddressMode::ZP,  2,  5, false));
        m.insert(0xf6, (Instructions::INC, AddressMode::ZPX,  2,  6, false));
        m.insert(0xee, (Instructions::INC, AddressMode::ABS,  3,  6, false));
        m.insert(0xfe, (Instructions::INC, AddressMode::ABX,  3,  7, false));
        m.insert(0xe8, (Instructions::INX, AddressMode::IMP,  1,  2, false));
        m.insert(0xc8, (Instructions::INY, AddressMode::IMP,  1,  2, false));
        m.insert(0x4c, (Instructions::JMP, AddressMode::ABS,  3,  3, false));
        m.insert(0x6c, (Instructions::JMP, AddressMode::IND,  3,  5, false));
        m.insert(0x20, (Instructions::JSR, AddressMode::ABS,  3,  6, false));
        m.insert(0xa9, (Instructions::LDA, AddressMode::IMM,  2,  2, false));
        m.insert(0xa5, (Instructions::LDA, AddressMode::ZP,  2,  3, false));
        m.insert(0xb5, (Instructions::LDA, AddressMode::ZPX,  2,  4, false));
        m.insert(0xad, (Instructions::LDA, AddressMode::ABS,  3,  4, false));
        m.insert(0xbd, (Instructions::LDA, AddressMode::ABX,  3,  4, true));
        m.insert(0xb9, (Instructions::LDA, AddressMode::ABY,  3,  4, true));
        m.insert(0xa1, (Instructions::LDA, AddressMode::IZX,  2,  6, false));
        m.insert(0xb1, (Instructions::LDA, AddressMode::IZY,  2,  5, true));
        m.insert(0xa2, (Instructions::LDX, AddressMode::IMM,  2,  2, false));
        m.insert(0xa6, (Instructions::LDX, AddressMode::ZP,  2,  3, false));
        m.insert(0xb6, (Instructions::LDX, AddressMode::ZPY,  2,  4, false));
        m.insert(0xae, (Instructions::LDX, AddressMode::ABS,  3,  4, false));
        m.insert(0xbe, (Instructions::LDX, AddressMode::ABY,  3,  4, true));
        m.insert(0xa0, (Instructions::LDY, AddressMode::IMM,  2,  2, false));
        m.insert(0xa4, (Instructions::LDY, AddressMode::ZP,  2,  3, false));
        m.insert(0xb4, (Instructions::LDY, AddressMode::ZPX,  2,  4, false));
        m.insert(0xac, (Instructions::LDY, AddressMode::ABS,  3,  4, false));
        m.insert(0xbc, (Instructions::LDY, AddressMode::ABX,  3,  4, true));
        m.insert(0x4a, (Instructions::LSR, AddressMode::IMP,  1,  2, false));
        m.insert(0x46, (Instructions::LSR, AddressMode::ZP,  2,  5, false));
        m.insert(0x56, (Instructions::LSR, AddressMode::ZPX,  2,  6, false));
        m.insert(0x4e, (Instructions::LSR, AddressMode::ABS,  3,  6, false));
        m.insert(0x5e, (Instructions::LSR, AddressMode::ABX,  3,  7, false));
        // m.insert(0x1a, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0x3a, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0x5a, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0x7a, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0xda, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0xea, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        // m.insert(0xfa, (Instructions::NOP, AddressMode::IMP,  1,  2, false));
        m.insert(0x09, (Instructions::ORA, AddressMode::IMM,  2,  2, false));
        m.insert(0x05, (Instructions::ORA, AddressMode::ZP,  2,  3, false));
        m.insert(0x15, (Instructions::ORA, AddressMode::ZPX,  2,  4, false));
        m.insert(0x0d, (Instructions::ORA, AddressMode::ABS,  3,  4, false));
        m.insert(0x1d, (Instructions::ORA, AddressMode::ABX,  3,  4, true));
        m.insert(0x19, (Instructions::ORA, AddressMode::ABY,  3,  4, true));
        m.insert(0x01, (Instructions::ORA, AddressMode::IZX,  2,  6, false));
        m.insert(0x11, (Instructions::ORA, AddressMode::IZY,  2,  5, true));
        m.insert(0x48, (Instructions::PHA, AddressMode::IMP,  1,  3, false));
        m.insert(0x08, (Instructions::PHP, AddressMode::IMP,  1,  3, false));
        m.insert(0x68, (Instructions::PLA, AddressMode::IMP,  1,  4, false));
        m.insert(0x28, (Instructions::PLP, AddressMode::IMP,  1,  4, false));
        m.insert(0x2a, (Instructions::ROL, AddressMode::IMP,  1,  2, false));
        m.insert(0x26, (Instructions::ROL, AddressMode::ZP,  2,  5, false));
        m.insert(0x36, (Instructions::ROL, AddressMode::ZPX,  2,  6, false));
        m.insert(0x2e, (Instructions::ROL, AddressMode::ABS,  3,  6, false));
        m.insert(0x3e, (Instructions::ROL, AddressMode::ABX,  3,  7, false));
        m.insert(0x6a, (Instructions::ROR, AddressMode::IMP,  1,  2, false));
        m.insert(0x66, (Instructions::ROR, AddressMode::ZP,  2,  5, false));
        m.insert(0x76, (Instructions::ROR, AddressMode::ZPX,  2,  6, false));
        m.insert(0x6e, (Instructions::ROR, AddressMode::ABS,  3,  6, false));
        m.insert(0x7e, (Instructions::ROR, AddressMode::ABX,  3,  7, false));
        m.insert(0x40, (Instructions::RTI, AddressMode::IMP,  1,  6, false));
        m.insert(0x60, (Instructions::RTS, AddressMode::IMP,  1,  6, false));
        m.insert(0xe9, (Instructions::SBC, AddressMode::IMM,  2,  2, false));
        m.insert(0xe5, (Instructions::SBC, AddressMode::ZP,  2,  3, false));
        m.insert(0xf5, (Instructions::SBC, AddressMode::ZPX,  2,  4, false));
        m.insert(0xed, (Instructions::SBC, AddressMode::ABS,  3,  4, false));
        m.insert(0xfd, (Instructions::SBC, AddressMode::ABX,  3,  4, true));
        m.insert(0xf9, (Instructions::SBC, AddressMode::ABY,  3,  4, true));
        m.insert(0xe1, (Instructions::SBC, AddressMode::IZX,  2,  6, false));
        m.insert(0xf1, (Instructions::SBC, AddressMode::IZY,  2,  5, true));
        m.insert(0x38, (Instructions::SEC, AddressMode::IMP,  1,  2, false));
        m.insert(0xf8, (Instructions::SED, AddressMode::IMP,  1,  2, false));
        m.insert(0x78, (Instructions::SEI, AddressMode::IMP,  1,  2, false));
        m.insert(0x85, (Instructions::STA, AddressMode::ZP,  2,  3, false));
        m.insert(0x95, (Instructions::STA, AddressMode::ZPX,  2,  4, false));
        m.insert(0x8d, (Instructions::STA, AddressMode::ABS,  3,  4, false));
        m.insert(0x9d, (Instructions::STA, AddressMode::ABX,  3,  5, false));
        m.insert(0x99, (Instructions::STA, AddressMode::ABY,  3,  5, false));
        m.insert(0x81, (Instructions::STA, AddressMode::IZX,  2,  6, false));
        m.insert(0x91, (Instructions::STA, AddressMode::IZY,  2,  6, true));
        m.insert(0x86, (Instructions::STX, AddressMode::ZP,  2,  3, false));
        m.insert(0x96, (Instructions::STX, AddressMode::ZPY,  2,  4, false));
        m.insert(0x8e, (Instructions::STX, AddressMode::ABS,  3,  4, false));
        m.insert(0x84, (Instructions::STY, AddressMode::ZP,  2,  3, false));
        m.insert(0x94, (Instructions::STY, AddressMode::ZPX,  2,  4, false));
        m.insert(0x8c, (Instructions::STY, AddressMode::ABS,  3,  4, false));
        m.insert(0xaa, (Instructions::TAX, AddressMode::IMP,  1,  2, false));
        m.insert(0xa8, (Instructions::TAY, AddressMode::IMP,  1,  2, false));
        m.insert(0xba, (Instructions::TSX, AddressMode::IMP,  1,  2, false));
        m.insert(0x8a, (Instructions::TXA, AddressMode::IMP,  1,  2, false));
        m.insert(0x9a, (Instructions::TXS, AddressMode::IMP,  1,  2, false));
        m.insert(0x98, (Instructions::TYA, AddressMode::IMP,  1,  2, false));
        //Illegal opcodes
        // m.insert(0x4b, (Instructions::ALR, AddressMode::IMM,  2,  2));
        // m.insert(0x0b, (Instructions::ANC, AddressMode::IMM,  2,  2));
        // m.insert(0x2b, (Instructions::ANC, AddressMode::IMM,  2,  2));
        // m.insert(0x6b, (Instructions::ARR, AddressMode::IMM,  2,  2));
        // m.insert(0xcb, (Instructions::AXS, AddressMode::IMM,  2,  2));
        // m.insert(0xa3, (Instructions::LAX, AddressMode::IZX,  2,  6));
        // m.insert(0xa7, (Instructions::LAX, AddressMode::ZP,  2,  3));
        // m.insert(0xaf, (Instructions::LAX, AddressMode::ABS,  3,  4));
        // m.insert(0xb3, (Instructions::LAX, AddressMode::IZY,  2,  5));
        // m.insert(0xb7, (Instructions::LAX, AddressMode::ZPY,  2,  4));
        // m.insert(0xbf, (Instructions::LAX, AddressMode::ABY,  3,  4));
        // m.insert(0x83, (Instructions::SAX, AddressMode::IZX,  2,  6));
        // m.insert(0x87, (Instructions::SAX, AddressMode::ZP,  2,  3));
        // m.insert(0x8f, (Instructions::SAX, AddressMode::ABS,  3,  4));
        // m.insert(0x97, (Instructions::SAX, AddressMode::ZPY,  2,  4));
        // m.insert(0xc3, (Instructions::DCP, AddressMode::IZX,  2,  8));
        // m.insert(0xc7, (Instructions::DCP, AddressMode::ZP,  2,  5));
        // m.insert(0xcf, (Instructions::DCP, AddressMode::ABS,  3,  6));
        // m.insert(0xd3, (Instructions::DCP, AddressMode::IZY,  2,  8));
        // m.insert(0xd7, (Instructions::DCP, AddressMode::ZPX,  2,  6));
        // m.insert(0xdb, (Instructions::DCP, AddressMode::ABY,  3,  7));
        // m.insert(0xdf, (Instructions::DCP, AddressMode::ABX,  3,  7));
        // m.insert(0xe3, (Instructions::ISC, AddressMode::IZX,  2,  8));
        // m.insert(0xe7, (Instructions::ISC, AddressMode::ZP,  2,  5));
        // m.insert(0xef, (Instructions::ISC, AddressMode::ABS,  3,  6));
        // m.insert(0xf3, (Instructions::ISC, AddressMode::IZY,  2,  8));
        // m.insert(0xf7, (Instructions::ISC, AddressMode::ZPX,  2,  6));
        // m.insert(0xfb, (Instructions::ISC, AddressMode::ABY,  3,  7));
        // m.insert(0xff, (Instructions::ISC, AddressMode::ABX,  3,  7));
        // m.insert(0x23, (Instructions::RLA, AddressMode::IZX,  2,  8));
        // m.insert(0x27, (Instructions::RLA, AddressMode::ZP,  2,  5));
        // m.insert(0x2f, (Instructions::RLA, AddressMode::ABS,  3,  6));
        // m.insert(0x33, (Instructions::RLA, AddressMode::IZY,  2,  8));
        // m.insert(0x37, (Instructions::RLA, AddressMode::ZPX,  2,  6));
        // m.insert(0x3b, (Instructions::RLA, AddressMode::ABY,  3,  7));
        // m.insert(0x3f, (Instructions::RLA, AddressMode::ABX,  3,  7));
        // m.insert(0x63, (Instructions::RRA, AddressMode::IZX,  2,  8));
        // m.insert(0x67, (Instructions::RRA, AddressMode::ZP,  2,  5));
        // m.insert(0x6f, (Instructions::RRA, AddressMode::ABS,  3,  6));
        // m.insert(0x73, (Instructions::RRA, AddressMode::IZY,  2,  8));
        // m.insert(0x77, (Instructions::RRA, AddressMode::ZPX,  2,  6));
        // m.insert(0x7b, (Instructions::RRA, AddressMode::ABY,  3,  7));
        // m.insert(0x7f, (Instructions::RRA, AddressMode::ABX,  3,  7));
        // m.insert(0x03, (Instructions::SLO, AddressMode::IZX,  2,  8));
        // m.insert(0x07, (Instructions::SLO, AddressMode::ZP,  2,  5));
        // m.insert(0x0f, (Instructions::SLO, AddressMode::ABS,  3,  6));
        // m.insert(0x13, (Instructions::SLO, AddressMode::IZY,  2,  8));
        // m.insert(0x17, (Instructions::SLO, AddressMode::ZPX,  2,  6));
        // m.insert(0x1b, (Instructions::SLO, AddressMode::ABY,  3,  7));
        // m.insert(0x1f, (Instructions::SLO, AddressMode::ABX,  3,  7));
        // m.insert(0x43, (Instructions::SRE, AddressMode::IZX,  2,  8));
        // m.insert(0x47, (Instructions::SRE, AddressMode::ZP,  2,  5));
        // m.insert(0x4f, (Instructions::SRE, AddressMode::ABS,  3,  6));
        // m.insert(0x53, (Instructions::SRE, AddressMode::IZY,  2,  8));
        // m.insert(0x57, (Instructions::SRE, AddressMode::ZPX,  2,  6));
        // m.insert(0x5b, (Instructions::SRE, AddressMode::ABY,  3,  7));
        // m.insert(0x5f, (Instructions::SRE, AddressMode::ABX,  3,  7));
        m
    };
}