// Motorola 68000 CPU

mod exception_cycle_table;
mod interrupt_level;
mod status_register;

pub mod register_type;

use interrupt_level::*;
use status_register::*;

// a macro to get byte on pc as pointer
macro_rules! read_imm_8 {
    ($pc: expr) => {
        *($pc as *const u8) as u32
    };
}
macro_rules! read_imm_16 {
    ($pc: expr) => {
        read_imm_8!($pc) << 8 | read_imm_8!($pc + 1)
    };
}
macro_rules! read_imm_32 {
    ($pc: expr) => {
        read_imm_16!($pc) << 16 | read_imm_16!($pc + 2)
    };
}

pub trait M68000Callback {
    fn read_memory_8(&self, address: u32) -> u32;
    fn read_memory_16(&self, address: u32) -> u32;
    fn read_memory_32(&self, address: u32) -> u32;

    fn write_memory_8(&self, address: u32, value: u32);
    fn write_memory_16(&self, address: u32, value: u32);
    fn write_memory_32(&self, address: u32, value: u32);

    fn fetch_memory_8(&self, address: u32) -> u32;
    fn fetch_memory_16(&self, address: u32) -> u32;
    fn fetch_memory_32(&self, address: u32) -> u32;

    fn rebase_pc(&self, pc: &mut usize, base_pc: &mut usize);

    fn interrupt_acknowledge(&self) -> u32;
}

pub struct M68000<T>
where
    T: M68000Callback,
{
    d: [u32; 8],
    a: [u32; 8],

    cflag: u32,
    vflag: u32,
    zflag: u32,
    nflag: u32,

    xflag: u32,
    sflag: u32,

    interrupt_level: u32,

    usp: u32,

    base_pc: usize,
    pc: usize,

    stopped: u32,
    irq_line: u32,

    nmi_pending: bool,
    virq_state: u32,

    initial_cycles: i32,
    remaining_cycles: i32,

    end_run: bool,

    // other fields
    ir: u32,
    smar: u32,
    smdr: u32,
    dmar: u32,
    dmdr: u32,

    callback: T,
}

impl<T: M68000Callback> M68000<T> {
    pub fn new(callback: T) -> Self {
        Self {
            d: [0; 8],
            a: [0; 8],

            cflag: 0,
            vflag: 0,
            zflag: 0,
            nflag: 0,

            xflag: 0,
            sflag: 0,

            interrupt_level: INTERRUPT_LEVEL_0,

            usp: 0,

            base_pc: 0,
            pc: 0,

            stopped: 0,
            irq_line: 0,

            nmi_pending: false,
            virq_state: 0,

            initial_cycles: 0,
            remaining_cycles: 0,

            end_run: false,

            ir: 0,
            smar: 0,
            smdr: 0,
            dmar: 0,
            dmdr: 0,

            callback,
        }
    }

    pub fn reset(&mut self) {
        self.d = [0; 8];
        self.a = [0; 8];

        // sp
        self.a[7] = self.callback.fetch_memory_32(0);

        self.cflag = 0;
        self.vflag = 0;
        self.zflag = 0;
        self.nflag = 0;

        self.xflag = 0;
        self.sflag = SR_S_SET;

        self.interrupt_level = INTERRUPT_LEVEL_NMI;

        self.usp = 0;

        self.pc = self.callback.fetch_memory_32(4) as usize;
        self.callback.rebase_pc(&mut self.pc, &mut self.base_pc);

        self.stopped = 0;
        self.irq_line = 0;

        self.nmi_pending = false;
        self.virq_state = 0;

        self.initial_cycles = 0;
        self.remaining_cycles = 0;

        self.end_run = false;
    }

    pub unsafe fn execute(&mut self, cycles: i32) -> i32 {
        self.initial_cycles = cycles;
        self.remaining_cycles = cycles;

        if self.stopped != 0 {
            self.remaining_cycles = 0;
            return self.initial_cycles;
        }

        loop {
            let ir = read_imm_16!(self.pc); self.pc += 2;
            let mut smar = 0;
            let mut smdr = 0;
            let mut dmar = 0;
            let mut dmdr = 0;

            match ir {
                // ori.b #<data>, Dn   # ori_8_d
                0x0000 => m68000_proc::ori!(ori, 8, imm, dy),
                _ => self.remaining_cycles -= 4,
            }

            if self.remaining_cycles >= 0 && !self.end_run {
                break;
            }
        }

        return self.initial_cycles - self.remaining_cycles;
    }
}
