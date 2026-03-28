#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::debug;
use log::error;
use log::info;
use minifb::Key;
use minifb::{Window, WindowOptions};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    env_logger::init();
    let mut app = App::init();
    app.start();
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

struct App {
    chip8: Chip8,
    window: Window,
    buffer: Vec<u32>,
}

impl App {
    fn init() -> App {
        let input: Vec<u8> = std::fs::read("input/IBM Logo.ch8").unwrap();

        let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT * 100];

        let mut window: Window = Window::new(
            "Test - ESC to exit",
            WIDTH * 10,
            HEIGHT * 10,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        window.set_target_fps(60);

        return App {
            chip8: Chip8::init(input),
            window: window,
            buffer: buffer,
        };
    }

    fn start(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let mut count = 0;
            while count < 10 {
                self.chip8.step();
                count += 1;
            }
            for i in 0..WIDTH {
                for j in 0..HEIGHT {
                    let bit = self.chip8.display[i + WIDTH * j];
                    if bit != 0 && bit != 1 {
                        error!("illegal bit value");
                        return;
                    }
                    for k in 0..10 {
                        for l in 0..10 {
                            self.buffer[(i * 10 + k) + (j * 10 + l) * WIDTH * 10] =
                                if bit == 1 { 0x00ffffff } else { 0x0 };
                        }
                    }
                }
            }
            self.window
                .update_with_buffer(&self.buffer, WIDTH * 10, HEIGHT * 10)
                .unwrap();
        }
    }
}

struct Chip8 {
    cpu_state: CpuState,
    display: [u8; 64 * 32],
}

impl Chip8 {
    fn init(input: Vec<u8>) -> Chip8 {
        let mut cpu = CpuState::new();
        cpu.memory[0x200..(0x200 + input.len())].copy_from_slice(&input);
        let sprites: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        cpu.memory[0..80].copy_from_slice(&sprites);
        return Chip8 {
            cpu_state: cpu,
            display: [0; 64 * 32],
        };
    }

    fn step(&mut self) {
        let state: &mut CpuState = &mut self.cpu_state;
        let diplay: &mut [u8; _] = &mut self.display;

        let pc: usize = state.pc as usize;
        let opcode: u16 = ((state.memory[pc] as u16) << 8) | (state.memory[pc + 1] as u16);
        match opcode {
            0x00E0 => {
                debug!("{:#06x} Clear the display", opcode);
            }
            0x00EE => {
                debug!("{:#06x} Return from sub routine", opcode);
                todo!("not implemented");
            }
            _ if (opcode & 0xF000) == 0x1000 => {
                let nnn = opcode & 0x0FFF;
                debug!("{:#06x} Sets the program counter to {nnn}", opcode);
                state.pc = opcode & 0x0FFF;
                return;
            }
            _ if (opcode & 0xF000) == 0x2000 => {
                debug!("Call subroutine at nnn");
                todo!("not implemented");
            }
            _ if (opcode & 0xF000) == 0x3000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Skip next instruction if V[{x}] = {kk}", opcode);
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x4000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Skip next instruction if V[{x}] != {kk}", opcode);
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x5000 => {
                let (x, y) = opcode.get_xy();
                debug!("{:#06x} Skip next instruction if V[{x}] = V[{y}]", opcode);
                if state.v[x] == state.v[y] {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x6000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Set V[{x}] = {kk}", opcode);
                state.v[x] = kk;
            }
            _ if (opcode & 0xF000) == 0x7000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Set V[{x}] = V{x} + {kk}", opcode);
                state.v[x as usize] += kk;
            }
            _ if (opcode & 0xF00F) == 0x8000 => {
                debug!("{:#06x} Set Vx = Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8001 => {
                debug!("{:#06x} Set Vx = Vx OR Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] ^ state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8003 => {
                debug!("{:#06x} Set Vx = Vx AND Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] & state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8004 => {
                debug!("{:#06x} Set Vx = Vx + Vy, set VF = carry", opcode);
                let (x, y) = opcode.get_xy();
                let carry;
                (state.v[x], carry) = state.v[x].overflowing_add(state.v[y]);
                state.v[0xf] = if carry { 1 } else { 0 }
            }
            _ if (opcode & 0xF00F) == 0x8005 => {
                debug!("{:#06x} Set Vx = Vx - Vy, set VF = NOT borrow", opcode);
                let (x, y) = opcode.get_xy();
                let overflow: bool;
                (state.v[x], overflow) = state.v[x].overflowing_sub(state.v[y]);
                state.v[0xf] = if !overflow { 1 } else { 0 };
            }
            _ if (opcode & 0xF00F) == 0x8006 => {
                debug!("{:#06x} Set Vx = Vx SHR 1", opcode);
                let (x, _) = opcode.get_xy();
                state.v[0xf] = if state.v[x] & 0x1 == 1 { 1 } else { 0 };
                state.v[x] = state.v[x] >> 1;
            }
            _ if (opcode & 0xF00F) == 0x8007 => {
                debug!("{:#06x} Set Vx = Vy - Vx, set VF = NOT borrow", opcode);
                let (x, y) = opcode.get_xy();
                let overflow: bool;
                (state.v[x], overflow) = state.v[y].overflowing_sub(state.v[x]);
                state.v[0xf] = if !overflow { 1 } else { 0 };
            }
            _ if (opcode & 0xF00F) == 0x800E => {
                debug!("{:#06x} Set Vx = Vx SHL 1", opcode);
                let (x, _) = opcode.get_xy();
                state.v[0xf] = if (state.v[x] & 0x80) == 0x80 { 1 } else { 0 };
                state.v[x] = state.v[x] << 1;
            }
            _ if (opcode & 0xF000) == 0x9000 => {
                let (x, y) = opcode.get_xy();
                debug!("{:#06x} Skip next instruction if V[{x}] != V[{y}]", opcode);
                if state.v[x] != state.v[y] {
                    state.pc += 2;
                }
            }
            _ if (opcode & 0xF000) == 0xA000 => {
                let nnn = opcode.get_nnn();
                debug!("{:#06x} Set I = {:#06x}", opcode, nnn);
                state.i = nnn;
            }
            _ if (opcode & 0xF000) == 0xB000 => {
                debug!("{:#06x} Jump to location nnn + V0", opcode);
                state.pc = opcode.get_nnn() + (state.v[0] as u16);
            }
            _ if (opcode & 0xF000) == 0xC000 => {
                debug!("{:#06x} Set Vx = random byte AND kk", opcode);
                let (x, kk) = opcode.get_xkk();
                state.v[x] = state.rng.random::<u8>() & kk;
            }
            _ if (opcode & 0xF000) == 0xD000 => {
                let (x, y) = opcode.get_xy();
                let (vx, vy) = (state.v[x] as usize, state.v[y] as usize);
                let n: u16 = opcode & 0xF;
                debug!(
                    "{:#06x} Display {n}-byte sprite starting at memory location I({:06x}) at (Vx, Vy), set VF = collision",
                    opcode, state.i
                );
                let i: usize = state.i as usize;
                let bytes = &state.memory[i..i + n as usize];
                state.v[0xf] = 0;
                for (index, byte) in bytes.iter().enumerate() {
                    let byte = *byte;
                    for bit_pos in 0..=7 {
                        let bit = (byte >> (7 - bit_pos)) & 1;
                        let array_pos = ((vx + bit_pos) % WIDTH) + ((vy + index) % HEIGHT) * WIDTH;
                        diplay[array_pos] ^= bit;
                        if diplay[array_pos] == 0 && bit == 1 {
                            state.v[0xf] = 1;
                        }
                    }
                }
            }
            _ if (opcode & 0xF0FF) == 0xE09E => {
                debug!(
                    "{:#06x} Skip next instruction if key with the value of Vx is pressed",
                    opcode
                );
            }
            _ if (opcode & 0xF0FF) == 0xE0A1 => {
                debug!(
                    "{:#06x} Skip next instruction if key with the value of Vx is not pressed",
                    opcode
                )
            }
            _ if (opcode & 0xF0FF) == 0xF007 => {
                debug!("{:#06x} Set Vx = delay timer value", opcode);
                let (x, _) = opcode.get_xy();
                state.v[x] = state.dt as u8;
            }
            _ if (opcode & 0xF0FF) == 0xF00A => {
                debug!(
                    "{:#06x} Wait for a key press, store the value of the key in Vx",
                    opcode
                )
            }
            _ if (opcode & 0xF0FF) == 0xF015 => {
                debug!("{:#06x} Set delay timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.dt = state.v[x]
            }
            _ if (opcode & 0xF0FF) == 0xF018 => {
                debug!("{:#06x} Set sound timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.dt = state.v[x];
            }
            _ if (opcode & 0xF0FF) == 0xF01E => {
                debug!("{:#06x} Set I = I + Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.i = state.i + (state.v[x] as u16);
            }
            _ if (opcode & 0xF0FF) == 0xF029 => {
                debug!("{:#06x} Set I = location of sprite for digit Vx", opcode)
            }
            _ if (opcode & 0xF0FF) == 0xF033 => {
                debug!(
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
                debug!(
                    "{:#06x} Store registers V0 through Vx in memory starting at location I",
                    opcode
                );
                let offset = state.i as usize;
                state.memory[offset..offset + 0x10].copy_from_slice(&state.v);
            }
            _ if (opcode & 0xF0FF) == 0xF065 => {
                debug!(
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
        state.pc += 2;
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
        let x: usize = ((self & 0x0F00) >> 8) as usize;
        let y: usize = ((self & 0x00F0) >> 4) as usize;
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
