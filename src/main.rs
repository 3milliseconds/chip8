use log::error;
use log::info;
use rand::{RngExt, SeedableRng, rngs::SmallRng};
fn main() {
    env_logger::init();
    let input = std::fs::read("input/IBM Logo.ch8").unwrap();
    let mut cpu = CpuState::new();
    cpu.memory[0x200..(0x200 + input.len())].copy_from_slice(&input);
    start_cpu(&mut cpu);
}

fn start_cpu(state: &mut CpuState) {
    let mut count = 0;
    while count < 50 {
        let pc = state.pc as usize;
        let opcode: u16 = ((state.memory[pc] as u16) << 8) | (state.memory[pc + 1] as u16);
        match opcode {
            0x00E0 => {
                info!("{:#06x} Clear the display", opcode);
            }
            0x00EE => {
                info!("{:#06x} Return from sub routine", opcode);
                todo!("not implemented");
            }
            _ if (opcode & 0xF000) == 0x1000 => {
                let nnn = opcode & 0x0FFF;
                info!("{:#06x} Sets the program counter to {nnn}", opcode);
                state.pc = opcode & 0x0FFF;
                continue;
            }
            _ if (opcode & 0xF000) == 0x2000 => {
                info!("Call subroutine at nnn");
                todo!("not implemented");
            }
            _ if (opcode & 0xF000) == 0x3000 => {
                let (x, kk) = opcode.get_xkk();
                info!("{:#06x} Skip next instruction if V[{x}] = {kk}", opcode);
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x4000 => {
                let (x, kk) = opcode.get_xkk();
                info!("{:#06x} Skip next instruction if V[{x}] != {kk}", opcode);
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x5000 => {
                let (x, y) = opcode.get_xy();
                info!("{:#06x} Skip next instruction if V[{x}] = V[{y}]", opcode);
                if state.v[x] == state.v[y] {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x6000 => {
                let (x, kk) = opcode.get_xkk();
                info!("{:#06x} Set V[{x}] = {kk}", opcode);
                state.v[x] = kk;
            }
            _ if (opcode & 0xF000) == 0x7000 => {
                let (x, kk) = opcode.get_xkk();
                info!("{:#06x} Set V[{x}] = V{x} + {kk}", opcode);
                state.v[x as usize] += kk;
            }
            _ if (opcode & 0xF00F) == 0x8000 => {
                info!("{:#06x} Set Vx = Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8001 => {
                info!("{:#06x} Set Vx = Vx OR Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] ^ state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8003 => {
                info!("{:#06x} Set Vx = Vx AND Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] & state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8004 => {
                info!("{:#06x} Set Vx = Vx + Vy, set VF = carry", opcode);
                let (x, y) = opcode.get_xy();
                let carry;
                (state.v[x], carry) = state.v[x].overflowing_add(state.v[y]);
                state.v[0xf] = if carry { 1 } else { 0 }
            }
            _ if (opcode & 0xF00F) == 0x8005 => {
                info!("{:#06x} Set Vx = Vx - Vy, set VF = NOT borrow", opcode);
                let (x, y) = opcode.get_xy();
                let overflow: bool;
                (state.v[x], overflow) = state.v[x].overflowing_sub(state.v[y]);
                state.v[0xf] = if !overflow { 1 } else { 0 };
            }
            _ if (opcode & 0xF00F) == 0x8006 => {
                info!("{:#06x} Set Vx = Vx SHR 1", opcode);
                let (x, _) = opcode.get_xy();
                state.v[0xf] = if state.v[x] & 0x1 == 1 { 1 } else { 0 };
                state.v[x] = state.v[x] >> 1;
            }
            _ if (opcode & 0xF00F) == 0x8007 => {
                info!("{:#06x} Set Vx = Vy - Vx, set VF = NOT borrow", opcode);
                let (x, y) = opcode.get_xy();
                let overflow: bool;
                (state.v[x], overflow) = state.v[y].overflowing_sub(state.v[x]);
                state.v[0xf] = if !overflow { 1 } else { 0 };
            }
            _ if (opcode & 0xF00F) == 0x800E => {
                info!("{:#06x} Set Vx = Vx SHL 1", opcode);
                let (x, _) = opcode.get_xy();
                state.v[0xf] = if (state.v[x] & 0x80) == 0x80 { 1 } else { 0 };
                state.v[x] = state.v[x] << 1;
            }
            _ if (opcode & 0xF000) == 0x9000 => {
                let (x, y) = opcode.get_xy();
                info!("{:#06x} Skip next instruction if V[{x}] != V[{y}]", opcode);
                if state.v[x] != state.v[y] {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0xA000 => {
                let nnn = opcode.get_nnn();
                info!("{:#06x} Set I = {nnn}", opcode);
                state.i = nnn;
            }
            _ if (opcode & 0xF000) == 0xB000 => {
                info!("{:#06x} Jump to location nnn + V0", opcode);
                state.pc = opcode.get_nnn() + (state.v[0] as u16);
            }
            _ if (opcode & 0xF000) == 0xC000 => {
                info!("{:#06x} Set Vx = random byte AND kk", opcode);
                let (x, kk) = opcode.get_xkk();
                state.v[x] = state.rng.random::<u8>() & kk;
            }
            _ if (opcode & 0xF000) == 0xD000 => info!(
                "{:#06x} Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision",
                opcode
            ),
            _ if (opcode & 0xF0FF) == 0xE09E => {
                info!(
                    "{:#06x} Skip next instruction if key with the value of Vx is pressed",
                    opcode
                );
            }
            _ if (opcode & 0xF0FF) == 0xE0A1 => {
                info!(
                    "{:#06x} Skip next instruction if key with the value of Vx is not pressed",
                    opcode
                )
            }
            _ if (opcode & 0xF0FF) == 0xF007 => {
                info!("{:#06x} Set Vx = delay timer value", opcode);
                let (x, _) = opcode.get_xy();
                state.v[x] = state.dt as u8;
            }
            _ if (opcode & 0xF0FF) == 0xF00A => {
                info!(
                    "{:#06x} Wait for a key press, store the value of the key in Vx",
                    opcode
                )
            }
            _ if (opcode & 0xF0FF) == 0xF015 => {
                info!("{:#06x} Set delay timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.dt = state.v[x]
            }
            _ if (opcode & 0xF0FF) == 0xF018 => {
                info!("{:#06x} Set sound timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.dt = state.v[x];
            }
            _ if (opcode & 0xF0FF) == 0xF01E => {
                info!("{:#06x} Set I = I + Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.i = state.i + (state.v[x] as u16);
            }
            _ if (opcode & 0xF0FF) == 0xF029 => {
                info!("{:#06x} Set I = location of sprite for digit Vx", opcode)
            }
            _ if (opcode & 0xF0FF) == 0xF033 => {
                info!(
                    "{:#06x} Store BCD representation of Vx in memory locations I, I+1, and I+2",
                    opcode
                );
                let (x, _) = opcode.get_xy();
                let mut r = state.v[x];
                let h = r / 100;
                r = r % 100;
                let t = r / 10;
                r = r % 10;
                let o = r;
                let pos = state.i as usize;
                state.memory[pos] = h;
                state.memory[pos + 1] = t;
                state.memory[pos + 2] = o;
            }
            _ if (opcode & 0xF0FF) == 0xF055 => {
                info!(
                    "{:#06x} Store registers V0 through Vx in memory starting at location I",
                    opcode
                );
                let offset = state.i as usize;
                state.memory[offset..offset + 0x10].copy_from_slice(&state.v);
            }
            _ if (opcode & 0xF0FF) == 0xF065 => {
                info!(
                    "{:#06x} Read registers V0 through Vx from memory starting at location I",
                    opcode
                );
                let offset = state.i as usize;
                state
                    .v
                    .copy_from_slice(&state.memory[offset..offset + 0x10]);
            }

            _ => {
                error!("{:#06x} Unknown instruction", opcode);
                return;
            }
        };
        count += 1;
        state.pc += 2;
    }
}

struct CpuState {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    dt: u8,
    st: u16,
    pc: u16,
    sp: u8,
    rng: SmallRng,
}

impl CpuState {
    fn new() -> CpuState {
        return CpuState {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
            rng: SmallRng::seed_from_u64(12345),
        };
    }
}

trait OpCode {
    fn get_nnn(&self) -> u16;
    fn get_xy(&self) -> (usize, usize);
    fn get_xkk(&self) -> (usize, u8);
}
impl OpCode for u16 {
    #[inline]
    fn get_xy(&self) -> (usize, usize) {
        let x = ((self & 0x0F00) >> 8) as usize;
        let y = ((self & 0x00F0) >> 4) as usize;
        (x, y)
    }

    #[inline]
    fn get_xkk(&self) -> (usize, u8) {
        let x = ((self & 0x0F00) >> 8) as usize;
        let kk = (self & 0x00FF) as u8;
        (x, kk)
    }

    #[inline]
    fn get_nnn(&self) -> u16 {
        (self & 0x0FFF) as u16
    }
}
