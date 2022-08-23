extern crate alloc;

use alloc::boxed::Box;
use bitmask_enum::bitmask;
use core::convert::From;
use core::result::Result;
use core::result::Result::{Err, Ok};

// description of the varvara virtual computer: https://wiki.xxiivv.com/site/varvara.html
// high level page of the VM: https://wiki.xxiivv.com/site/uxn.html

// Memory   RAM             Data    64kb
// Stacks   Working Stack   Data    254 bytes
//                          Error   1 byte
//                          Pointer 1 byte
//          Return Stack    Data    254 bytes
//                          Error   1 byte
//                          Pointer 1 byte
// IO       Devices         Data    256 bytes

// we have a DEI and DEO trait to use for devices

// 0xFF //

pub type PortAddress = u8;
pub type InstructionPointer = u16;
pub type ExecutionResult<T> = Result<T, &'static str>;

#[bitmask(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InstructionMode {
    None = 0x00,
    Return = 0x40,
    Keep = 0x80,
    Short = 0x20,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Opcode {
    LIT = 0x00,
    INC = 0x01,
    POP = 0x02,
    NIP = 0x03,
    SWP = 0x04,
    ROT = 0x05,
    DUP = 0x06,
    OVR = 0x07,
    EQU = 0x08,
    NEQ = 0x09,
    GTH = 0x0a,
    LTH = 0x0b,
    JMP = 0x0c,
    JCN = 0x0d,
    JSR = 0x0e,
    STH = 0x0f,
    LDZ = 0x10,
    STZ = 0x11,
    LDR = 0x12,
    STR = 0x13,
    LDA = 0x14,
    STA = 0x15,
    DEI = 0x16,
    DEO = 0x17,
    ADD = 0x18,
    SUB = 0x19,
    MUL = 0x1a,
    DIV = 0x1b,
    AND = 0x1c,
    ORA = 0x1d,
    EOR = 0x1e,
    SFT = 0x1f,
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

trait Device {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8>;
    // fn dei2(&self, port: PortAddress) -> Result<u16, &str>;
    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()>;
    // fn deo2(&self, port: PortAddress, value: u16) -> Result<(), &str>;
}

struct VectorDevice {
    x: u8,
    y: u8,
    width: u8,
    height: u8,
    color: u8
}

impl VectorDevice {
    fn new() -> Self {
        VectorDevice {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            color: 0
        }
    }

    // memory layout of vector device
    // 0x00: draw (write 0x01 for RECT, 0x02 for CIRCLE)
    // 0x01: x
    // 0x02: y
    // 0x03: width
    // 0x04: height
    // 0x05: color
    fn draw_circle(&self) {
        println!("draw circle at ({}, {}) with radius {} and color {}", self.x, self.y, self.width, self.color);
    }

    fn draw_rectangle(&self) {
        println!("drawing rectangle at ({}, {}) with width {} and height {}", self.x, self.y, self.width, self.height);
    }
}
impl Device for VectorDevice {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8> {
        return Err("device not implemented");
    }

    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()> {
        match port {
            0x00 => match value {
                0x01 => self.draw_rectangle(),
                0x02 => self.draw_circle(),
                _ => return Err("invalid draw command")
            },
            0x01 => self.x = value,
            0x02 => self.y = value,
            0x03 => self.width = value,
            0x04 => self.height = value,
            0x05 => self.color = value,
            _ => return Err("device not implemented"),
        }
        return Err("device not implemented");
    }
}


struct BitmapDevice {
    buffer: [u8; 256],
    x: u8,
    y: u8,
    color: u8,
}

impl BitmapDevice {
    fn new() -> Self {
        BitmapDevice {
            buffer: [0; 256],
            x: 0,
            y: 0,
            color: 0
        }
    }

    fn blit(&self) {
        println!("blitting bitmap to screen");
    }

    fn draw_pixel(&mut self) {
        self.buffer[self.y as usize * 16 + self.x as usize] = self.color;
    }
}

impl Device for BitmapDevice {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8> {
        return Err("device not implemented");
    }
    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()> {
        match port {
            0x00 => match value {
                0x00 => self.blit(),
                0x01 => self.draw_pixel(),
                _ => return Err("invalid draw command")
            },
            0x01 => self.x = value % 16,
            0x02 => self.y = value % 16,
            0x03 => self.color = value,
            _ => return Err("device not implemented"),
        }
        return Err("device not implemented");
    }
}

struct NullDevice {}

impl Device for NullDevice {
    fn dei(&self, _port: PortAddress) -> ExecutionResult<u8> {
        Err("NullDevice::dei")
    }
    fn deo(&mut self, _port: PortAddress, _value: u8) -> ExecutionResult<()> {
        Err("NullDevice::deo")
    }
}

type StackPointer = u8;

struct Stack {
    ptr: StackPointer,
    kptr: StackPointer,
    data: [u8; 256],
}

impl Stack {
    pub fn print(&self) {
        println!("Stack: {:?}", self.data);
    }
}

pub struct Uxn {
    ram: [u8; 65536],
    pc: u16,
    wst: Stack,
    rst: Stack,
    devices: [Box<dyn Device>; 16],
    is_halted: bool,
}

impl Device for Uxn {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8> {
        match port {
            0x02 => return Ok(self.wst.ptr),
            0x03 => return Ok(self.rst.ptr),
            _ => return Err("Uxn::dei"),
        }
    }

    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()> {
        match port {
            0x02 => self.wst.ptr = value,
            0x03 => self.rst.ptr = value,
            0x0e => self.print(),
            0x0f => self.is_halted = value != 0x00,
            port if port > 0x07 && port < 0x0e => return Ok(()), // TODO screen palette
            _ => return Err("Uxn::deo"),
        }
        Ok(())
    }
}

impl Uxn {
    pub fn new() -> Self {
        Uxn {
            ram: [0; 65536],
            pc: 0,
            wst: Stack {
                ptr: 0,
                kptr: 0,
                data: [0; 256],
            },
            rst: Stack {
                ptr: 0,
                kptr: 0,
                data: [0; 256],
            },
            devices: [
                Box::new(NullDevice {}), // reserved for the system device
                Box::new(BitmapDevice::new()),
                Box::new(VectorDevice::new()),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {}),
                Box::new(NullDevice {})
            ],
            is_halted: false,
        }
    }

    pub fn boot(&mut self) {
        let x: u8 = 0;
        let x2: StackPointer = 0;

        self.wst.ptr = x2;
        self.wst.ptr = x;

        self.wst.ptr = 0;
        self.wst.kptr = self.wst.ptr;
        self.rst.ptr = 0;
        self.rst.kptr = self.rst.ptr;

        self.ram.iter_mut().for_each(|x| *x = 0);
        self.pc = 0;
        self.is_halted = false;
    }

    pub fn load_program(&mut self, program: &[u8], addr: usize) {
        self.ram[addr..(addr + program.len())].copy_from_slice(program);
    }

    #[inline(always)]
    pub fn peek(&mut self, addr: usize, mode: InstructionMode) -> ExecutionResult<u16> {
        if mode.contains(InstructionMode::Short) {
            Ok((self.ram[addr] as u16) << 8 | self.ram[addr + 1] as u16)
        } else {
            Ok(self.ram[addr] as u16)
        }
    }

    #[inline(always)]
    fn get_stack(&mut self, mode: InstructionMode) -> &mut Stack {
        if mode.contains(InstructionMode::Return) {
            &mut self.rst
        } else {
            &mut self.wst
        }
    }

    #[inline(always)]
    pub fn kpop8(&mut self, mode: InstructionMode) -> ExecutionResult<u16> {
        let mut s = self.get_stack(mode);
        if s.kptr == 0 {
            return Err("Stack underflow");
        }
        let value = s.data[s.kptr as usize];
        s.kptr -= 1;
        Ok(value as u16)
    }

    #[inline(always)]
    pub fn kpop16(&mut self, mode: InstructionMode) -> ExecutionResult<u16> {
        let mut s = self.get_stack(mode);
        if s.kptr <= 1 {
            return Err("Stack underflow");
        }
        s.kptr -= 2;
        Ok((s.data[s.kptr as usize] as u16) << 8 | s.data[s.kptr as usize + 1] as u16)
    }

    #[inline(always)]
    pub fn pop8(&mut self, mode: InstructionMode) -> ExecutionResult<u16> {
        let mut s = self.get_stack(mode);
        if s.ptr == 0 {
            return Err("Stack underflow");
        }
        s.ptr -= 1;
        let value = s.data[s.ptr as usize];
        Ok(value as u16)
    }

    #[inline(always)]
    pub fn pop16(&mut self, mode: InstructionMode) -> ExecutionResult<u16> {
        let mut s = self.get_stack(mode);
        if s.ptr <= 1 {
            return Err("Stack underflow");
        }
        s.ptr -= 2;
        Ok((s.data[s.ptr as usize] as u16) << 8 | s.data[s.ptr as usize + 1] as u16)
    }

    #[inline(always)]
    pub fn pop(&mut self, mode: InstructionMode) -> ExecutionResult<u16> {
        if mode.contains(InstructionMode::Keep) {
            if mode.contains(InstructionMode::Short) {
                self.kpop16(mode)
            } else {
                self.kpop8(mode)
            }
        } else {
            if mode.contains(InstructionMode::Short) {
                self.pop16(mode)
            } else {
                self.pop8(mode)
            }
        }
    }

    #[inline(always)]
    pub fn push8(&mut self, v: u16, mode: InstructionMode) -> ExecutionResult<()> {
        let mut s = self.get_stack(mode);
        if s.ptr >= 255 {
            return Err("Stack overflow");
        }
        s.data[s.ptr as usize] = v as u8;
        s.ptr += 1;
        Ok(())
    }
    #[inline(always)]
    pub fn push16(&mut self, v: u16, mode: InstructionMode) -> ExecutionResult<()> {
        let mut s = self.get_stack(mode);
        if s.ptr >= 254 {
            return Err("Stack overflow");
        }
        s.data[s.ptr as usize] = (v >> 8) as u8;
        s.data[s.ptr as usize] = (v & 0xff) as u8;
        s.ptr += 2;
        Ok(())
    }
    #[inline(always)]
    pub fn push(&mut self, v: u16, mode: InstructionMode) -> ExecutionResult<()> {
        if mode.contains(InstructionMode::Short) {
            self.push16(v, mode)
        } else {
            self.push8(v, mode)
        }
    }

    #[inline(always)]
    pub fn warp(&mut self, addr: u16, mode: InstructionMode) -> ExecutionResult<()> {
        if mode.contains(InstructionMode::Short) {
            self.pc = addr;
        } else {
            self.pc += addr;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn poke(&mut self, addr: usize, value: u16, mode: InstructionMode) -> ExecutionResult<()> {
        if mode.contains(InstructionMode::Short) {
            self.ram[addr] = (value >> 8) as u8;
            self.ram[addr + 1] = (value & 0xff) as u8;
        } else {
            self.ram[addr] = value as u8;
        }
        Ok(())
    }

    pub fn eval(&mut self, start_addr: InstructionPointer) -> Result<(), &str> {
        self.pc = start_addr;

        if self.pc == 0x0 || self.is_halted {
            return Ok(());
        }

        loop {
            let instr = self.ram[self.pc as usize];
            let opcode = (instr & 0x1f).into();

            self.pc += 1;
            if instr == 0x00 {
                break;
            }

            let mode: InstructionMode = instr.into();
            let is_keep = mode.contains(InstructionMode::Keep);

            if is_keep {
                self.wst.kptr = self.wst.ptr;
                self.rst.kptr = self.rst.ptr;
            }

            let res: Result<(), &str> = match opcode {
                Opcode::LIT => {
                    self.peek(self.pc as usize, mode)
                        .and_then(|a|
                            self.push(a, mode).and_then(|_| {
                                self.pc += 1;
                                if mode.contains(InstructionMode::Short) {
                                    self.pc += 1;
                                }
                                Ok(())
                            })).into()
                }
                Opcode::INC => {
                    self.pop(mode).and_then(|a|
                        self.push(a + 1, mode)).into()
                }
                Opcode::POP => {
                    self.pop(mode).and_then(|_| Ok(()))
                }
                Opcode::NIP => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|_|
                            self.push(a, mode))).into()
                }
                Opcode::SWP => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a, mode).and_then(|_|
                                self.push(b, mode)))).into()
                }
                Opcode::ROT => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.pop(mode).and_then(|c|
                                self.push(b, mode).and_then(|_|
                                    self.push(a, mode)).and_then(|_|
                                    self.push(c, mode))))).into()
                }
                Opcode::DUP => {
                    self.pop(mode).and_then(|a|
                        self.push(a, mode).and_then(|_|
                            self.push(a, mode))).into()
                }
                Opcode::OVR => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(b, mode).and_then(|_|
                                self.push(a, mode).and_then(|_|
                                    self.push(b, mode))))).into()
                }
                Opcode::EQU => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push8(if a == b { 1 } else { 0 }, mode))).into()
                }
                Opcode::NEQ => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push8(if a != b { 1 } else { 0 }, mode))).into()
                }
                Opcode::GTH => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push8(if b > a { 1 } else { 0 }, mode))).into()
                }
                Opcode::LTH => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push8(if b < a { 1 } else { 0 }, mode))).into()
                }
                Opcode::JMP => {
                    self.pop(mode).and_then(|a|
                        self.warp(a, mode)).into()
                }
                Opcode::JCN => {
                    self.pop(mode).and_then(|a|
                        self.pop8(mode).and_then(|b|
                            if b != 0 { self.warp(a, mode) } else { Ok(()) })).into()
                }
                Opcode::JSR => {
                    self.pop(mode).and_then(|a|
                        self.push8(self.pc,
                                   if mode.contains(InstructionMode::Return) {
                                       InstructionMode::None
                                   } else {
                                       InstructionMode::Return
                                   }).and_then(|_|
                            self.warp(a, mode))).into()
                }
                Opcode::STH => {
                    self.pop(mode).and_then(|a|
                        self.push16(a,
                                    if mode.contains(InstructionMode::Return) {
                                        InstructionMode::None
                                    } else {
                                        InstructionMode::Return
                                    })).into()
                }
                Opcode::LDZ => {
                    self.pop8(mode).and_then(|a|
                        self.peek(a as usize, mode).and_then(|b|
                            self.push(b, mode))).into()
                }
                Opcode::STZ => {
                    self.pop8(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.poke(a as usize, b, mode))).into()
                }
                Opcode::LDR => {
                    self.pop8(mode).and_then(|a|
                        self.peek((a + self.pc) as usize, mode).and_then(|b|
                            self.push(b, mode))).into()
                }
                Opcode::STR => {
                    self.pop8(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.poke((a + self.pc) as usize, b, mode))).into()
                }
                Opcode::LDA => {
                    self.pop16(mode).and_then(|a|
                        self.peek(a as usize, mode).and_then(|b|
                            self.push(b, mode))).into()
                }
                Opcode::STA => {
                    self.pop16(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.poke(a as usize, b, mode))).into()
                }
                Opcode::DEI => {
                    self.pop8(mode).and_then(|a| {
                        {
                            let device = ((a >> 4) & 0x0f) as usize;
                            let port = (a & 0x0F) as u8;
                            if device == 0 {
                                // system device
                                self.dei(port)
                            } else {
                                self.devices[device].dei(port)
                            }
                        }.and_then(|b|
                            self.push(b as u16, mode)
                        )
                    }).into()
                }
                Opcode::DEO => {
                    self.pop8(mode).and_then(|a| {
                        self.pop(mode).and_then(|value| {
                            let device = ((a >> 4) & 0x0f) as usize;
                            let port = (a & 0x0F) as u8;
                            if device == 0 {
                                // system device
                                self.deo(port, a as u8)
                            } else {
                                self.devices[device].deo(port, value as u8)
                            }
                        })
                    }).into()
                }
                Opcode::ADD => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a + b, mode))).into()
                }
                Opcode::SUB => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(b - a, mode))).into()
                }
                Opcode::MUL => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a * b, mode))).into()
                }
                Opcode::DIV => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b| {
                            if a == 0 {
                                Err("Division by zero")
                            } else {
                                self.push(b / a, mode)
                            }
                        }))
                }
                Opcode::AND => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a & b, mode))).into()
                }
                Opcode::ORA => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a | b, mode))).into()
                }
                Opcode::EOR => {
                    self.pop(mode).and_then(|a|
                        self.pop(mode).and_then(|b|
                            self.push(a ^ b, mode))).into()
                }
                Opcode::SFT => {
                    self.pop(mode).and_then(|a|
                        self.pop8(mode).and_then(|b|
                            self.push(a << ((b & 0xF0) >> 4) >> (b & 0x0F), mode))).into()
                }
            };
            if res.is_err() {
                return res;
            }
        }


        Ok(())
    }

    pub fn halt(&mut self) {
        self.wst.print();
        self.rst.print();
    }

    pub fn print(&self) {}
}
