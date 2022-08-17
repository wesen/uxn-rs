use bitmask_enum::bitmask;

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

type PortAddress = u8;
type InstructionPointer = u16;

#[bitmask(u8)]
enum InstructionMode {
    Return = 0x40,
    Keep = 0x80,
    Short = 0x20,
}

#[repr(u8)]
enum Opcode {
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

impl std::convert::Into<u8> for Opcode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

type ExecutionResult<T> = Result<T, &'static str>;

trait Device {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8>;
    // fn dei2(&self, port: PortAddress) -> Result<u16, &str>;
    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()>;
    // fn deo2(&self, port: PortAddress, value: u16) -> Result<(), &str>;
}

struct NullDevice {}

impl Device for NullDevice {
    fn dei(&self, port: PortAddress) -> ExecutionResult<u8> {
        Err("NullDevice::dei")
    }
    fn deo(&mut self, port: PortAddress, value: u8) -> ExecutionResult<()> {
        Err("NullDevice::deo")
    }
}

type StackPointer = u8;

struct Stack {
    ptr: StackPointer,
    data: [u8; 256],
}

impl Stack {
    pub fn print(&self) {
        println!("Stack: {:?}", self.data);
    }
}

struct Uxn {
    ram: [u8; 65536],
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
    pub fn boot(&mut self) {
        let x: u8 = 0;
        let x2: StackPointer = 0;

        self.wst.ptr = x2;
        self.wst.ptr = x;

        self.wst.ptr = 0;
        self.rst.ptr = 0;
        self.ram.iter_mut().for_each(|x| *x = 0);
        self.is_halted = false;
    }
    pub fn peek(&mut self, addr: usize, is_short: bool) -> ExecutionResult<u16> {
        if is_short {
            Ok((self.ram[addr] as u16) << 8 | self.ram[addr + 1] as u16)
        } else {
            Ok(self.ram[addr] as u16)
        }
    }

    pub fn pop8(&mut self, s: &mut Stack) -> ExecutionResult<u16> {
        if s.ptr == 0 {
            return Err("Stack underflow");
        }
        let value = s.data[s.ptr as usize];
        s.ptr -= 1;
        Ok(value as u16)
    }

    pub fn pop16(&mut self, s: &mut Stack) -> ExecutionResult<u16> {
        if s.ptr <= 1 {
            return Err("Stack underflow");
        }
        s.ptr -= 2;
        Ok((s.data[s.ptr as usize] as u16) << 8 | s.data[s.ptr as usize + 1] as u16)
    }
    pub fn pop(&mut self, s: &mut Stack, is_short: bool) -> ExecutionResult<u16> {
        if is_short {
            self.pop8(s)
        } else {
            self.pop16(s)
        }
    }

    pub fn push8(&mut self, v: u16, s: &mut Stack) -> ExecutionResult<()> {
        if s.ptr >= 255 {
            return Err("Stack overflow");
        }
        s.data[s.ptr as usize] = v as u8;
        s.ptr += 1;
        Ok(())
    }
    pub fn push16(&mut self, v: u16, s: &mut Stack) -> ExecutionResult<()> {
        if s.ptr >= 254 {
            return Err("Stack overflow");
        }
        s.data[s.ptr as usize] = (v >> 8) as u8;
        s.data[s.ptr as usize] = (v & 0xff) as u8;
        s.ptr += 2;
        Ok(())
    }
    pub fn push(&mut self, v: u16, is_short: bool) -> ExecutionResult<()> {
        if is_short {
            self.push8(v, ss)
        } else {
            self.push16(v, ss)
        }
    }

    pub fn warp(&mut self, addr: u16) -> ExecutionResult<()> {
        if is_short {
            pc = addr;
        } else {
            pc += addr;
        }
        Ok(())
    }

    pub fn poke(&mut self, addr: usize, value: u16) -> ExecutionResult<()> {
        if is_short {
            self.ram[addr] = (value >> 8) as u8;
            self.ram[addr + 1] = (value & 0xff) as u8;
        } else {
            self.ram[addr] = value as u8;
        }
        Ok(())
    }


    pub fn eval(&mut self, startAddr: InstructionPointer) -> Result<(), &str> {
        let mut pc = startAddr;

        if pc == 0x0 || self.is_halted {
            return Ok(());
        }

        // TODO make sure we are not running too long

        let opcode = self.ram[pc as usize];
        let mode: InstructionMode = opcode.into();
        let is_return = mode.contains(InstructionMode::Return);
        let is_keep = mode.contains(InstructionMode::Keep);
        let is_short = mode.contains(InstructionMode::Short);

        let (ss, ds) = if is_return {
            (&mut self.rst, &mut self.wst)
        } else {
            (&mut self.wst, &mut self.rst)
        };

        let res: Result<(), &str> = match (opcode & 0x1f).into() {
            Opcode::LIT => {
                self.peek(pc as usize, is_short).and_then(|a|
                    self.push(a, is_short)).into()
            }
            Opcode::INC => {
                self.pop(ss, is_short).and_then(|a|
                    self.push(a + 1, is_short)).into()
            }
            Opcode::POP => {
                self.pop(ss, is_short).and_then(|_| Ok(()))
            }
            Opcode::NIP => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|_|
                        self.push(a, is_short))).into()
            }
            Opcode::SWP => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push(a, is_short).and_then(|_|
                            self.push(b, is_short)))).into()
            }
            Opcode::ROT => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.pop(ss, is_short).and_then(|c|
                            self.push(b, is_short).and_then(|_|
                                self.push(a, is_short)).and_then(|_|
                                self.push(c, is_short))))).into()
            }
            Opcode::DUP => {
                self.pop(ss, is_short).and_then(|a|
                    self.push(a, is_short).and_then(|_|
                        self.push(a, is_short))).into()
            }
            Opcode::OVR => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push(b, is_short).and_then(|_|
                            self.push(a, is_short).and_then(|_|
                                self.push(b, is_short))))).into()
            }
            Opcode::EQU => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push8(if a == b { 1 } else { 0 }, ss))).into()
            }
            Opcode::NEQ => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push8(if a != b { 1 } else { 0 }, ss))).into()
            }
            Opcode::GTH => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push8(if b > a { 1 } else { 0 }, ss))).into()
            }
            Opcode::LTH => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.push8(if b < a { 1 } else { 0 }, ss))).into()
            }
            Opcode::JMP => {
                self.pop(ss, is_short).and_then(|a|
                    self.warp(a)).into()
            }
            Opcode::JCN => {
                self.pop(ss, is_short).and_then(|a|
                    self.pop8(ss).and_then(|b|
                        if b != 0 { self.warp(a) } else { Ok(()) })).into()
            }
            Opcode::JSR => {
                self.pop(ss, is_short).and_then(|a|
                    self.push8(pc, ds).and_then(|_|
                        self.warp(a))).into()
            }
            Opcode::STH => {
                self.pop(ss, is_short).and_then(|a|
                    self.push16(a, ds)).into()
            }
            Opcode::LDZ => {
                self.pop8(ss).and_then(|a|
                    self.peek(a as usize, is_short).and_then(|b|
                        self.push(b, is_short))).into()
            }
            Opcode::STZ => {
                self.pop8(ss).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.poke(a as usize, b))).into()
            }
            Opcode::LDR => {
                self.pop8(ss).and_then(|a|
                    self.peek((a + pc) as usize, is_short).and_then(|b|
                        self.push(b, is_short))).into()
            }
            Opcode::STR => {
                self.pop8(ss).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.poke((a + pc) as usize, b))).into()
            }
            Opcode::LDA => {
                self.pop16(ss).and_then(|a|
                    self.peek(a as usize, is_short).and_then(|b|
                        self.push(b, is_short))).into()
            }
            Opcode::STA => {
                self.pop16(ss).and_then(|a|
                    self.pop(ss, is_short).and_then(|b|
                        self.poke(a as usize, b))).into()
            }
            Opcode::DEI => {
                self.pop8(ss).and_then(|a| {
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
                        self.push(b as u16, is_short)
                    )
                }).into()
            }
            // DEO => {}
            // ADD => {}
            // SUB => {}
            // MUL => {}
            // DIV => {}
            // AND => {}
            // ORA => {}
            // EOR => {}
            // SFT => {}
            _ => return Err("Uxn::eval"),
        };


        Ok(())
    }

    pub fn halt(&mut self) {
        self.wst.print();
        self.rst.print();
    }

    pub fn print(&self) {}
}

fn main() {
    let mut uxn = Uxn {
        ram: [0; 65536],
        wst: Stack {
            ptr: 0,
            data: [0; 256],
        },
        rst: Stack {
            ptr: 0,
            data: [0; 256],
        },
        devices: [
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
            Box::new(NullDevice {}),
            Box::new(NullDevice {}),
            Box::new(NullDevice {}),
            Box::new(NullDevice {}),
        ],
        is_halted: false,
    };

    println!("Hello, world!");
}
