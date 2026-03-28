#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::env;

use log::{debug, warn};
use minifb::Key;
use minifb::{Window, WindowOptions};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    env_logger::init();
    let path = env::args().nth(1).expect("Usage: chip8 <rom path>");
    let mut app = App::init(path);
    app.start();
}

struct CpuState {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u16,
    program_counter: u16,
    stack: Vec<u16>,
    rng: SmallRng,
}

impl CpuState {
    fn new() -> CpuState {
        return CpuState {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,
            stack: Vec::new(),
            rng: SmallRng::seed_from_u64(12345),
        };
    }
}

struct App {
    chip8: Chip8,
    window: Window,
    buffer: Vec<u32>,
    key_map: HashMap<Key, usize>,
}

impl App {
    fn init(input_path: String) -> App {
        let input: Vec<u8> = std::fs::read(input_path).expect("Failed to read file");

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
        let key_map: HashMap<Key, usize> = HashMap::from([
            (Key::Key1, 0x1),
            (Key::Key2, 0x2),
            (Key::Key3, 0x3),
            (Key::Key4, 0xC),
            (Key::Q, 0x4),
            (Key::W, 0x5),
            (Key::E, 0x6),
            (Key::R, 0xd),
            (Key::A, 0x7),
            (Key::S, 0x8),
            (Key::D, 0x9),
            (Key::F, 0xe),
            (Key::Z, 0xa),
            (Key::X, 0x0),
            (Key::C, 0xb),
            (Key::V, 0xf),
        ]);
        return App {
            chip8: Chip8::init(input),
            window: window,
            buffer: buffer,
            key_map: key_map,
        };
    }

    fn start(&mut self) {
        let mut keys: [bool; 16] = [false; 16];
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let mut count = 0;
            for (&key, &value) in self.key_map.iter() {
                keys[value] = self.window.is_key_down(key);
            }
            while count < 10 {
                self.chip8.step(keys);
                count += 1;
            }
            for i in 0..WIDTH {
                for j in 0..HEIGHT {
                    let bit = self.chip8.display[i + WIDTH * j];
                    for k in 0..10 {
                        for l in 0..10 {
                            self.buffer[(i * 10 + k) + (j * 10 + l) * WIDTH * 10] =
                                if bit == 1 { 0x00ffffff } else { 0x0 };
                        }
                    }
                }
            }
            if self.chip8.cpu_state.delay_timer != 0 {
                self.chip8.cpu_state.delay_timer -= 1;
            }
            if self.chip8.cpu_state.sound_timer != 0 {
                self.chip8.cpu_state.sound_timer -= 1;
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

    fn step(&mut self, keys: [bool; 16]) {
        let state: &mut CpuState = &mut self.cpu_state;
        let diplay: &mut [u8; _] = &mut self.display;

        let pc: usize = state.program_counter as usize;
        let opcode: u16 = ((state.memory[pc] as u16) << 8) | (state.memory[pc + 1] as u16);
        match opcode {
            0x00E0 => {
                debug!("{:#06x} Clear the display", opcode);
                for i in 0..diplay.len() {
                    diplay[i] = 0;
                }
            }
            0x00EE => {
                debug!("{:#06x} Return from sub routine", opcode);
                let return_address = state.stack.pop().expect("Can't return to empty stack");
                state.program_counter = return_address;
            }
            _ if (opcode & 0xF000) == 0x1000 => {
                let nnn = opcode & 0x0FFF;
                debug!("{:#06x} Sets the program counter to {nnn}", opcode);
                state.program_counter = opcode & 0x0FFF;
                return;
            }
            _ if (opcode & 0xF000) == 0x2000 => {
                let nnn = opcode & 0x0FFF;
                debug!("Call subroutine at {nnn}");
                state.stack.push(state.program_counter);
                state.program_counter = nnn;
                return;
            }
            _ if (opcode & 0xF000) == 0x3000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Skip next instruction if V[{x}] = {kk}", opcode);
                if state.v[x] == kk {
                    state.program_counter += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x4000 => {
                let (x, kk) = opcode.get_xkk();
                debug!("{:#06x} Skip next instruction if V[{x}] != {kk}", opcode);
                if state.v[x] != kk {
                    state.program_counter += 2;
                }
            }
            _ if (opcode & 0xF000) == 0x5000 => {
                let (x, y) = opcode.get_xy();
                debug!("{:#06x} Skip next instruction if V[{x}] = V[{y}]", opcode);
                if state.v[x] == state.v[y] {
                    state.program_counter += 2;
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
                state.v[x as usize] = state.v[x as usize].wrapping_add(kk);
            }
            _ if (opcode & 0xF00F) == 0x8000 => {
                debug!("{:#06x} Set Vx = Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8001 => {
                debug!("{:#06x} Set Vx = Vx OR Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] | state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8002 => {
                debug!("{:#06x} Set Vx = Vx AND Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] & state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8003 => {
                debug!("{:#06x} Set Vx = Vx XOR Vy", opcode);
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] ^ state.v[y as usize];
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
                    state.program_counter += 2;
                }
            }
            _ if (opcode & 0xF000) == 0xA000 => {
                let nnn = opcode.get_nnn();
                debug!("{:#06x} Set I = {:#06x}", opcode, nnn);
                state.i = nnn;
            }
            _ if (opcode & 0xF000) == 0xB000 => {
                debug!("{:#06x} Jump to location nnn + V0", opcode);
                state.program_counter = opcode.get_nnn() + (state.v[0] as u16);
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
                let (x, _) = opcode.get_xy();
                debug!(
                    "{:#06x} Skip next instruction if key with the value of V[{x}] is pressed",
                    opcode
                );
                if keys[state.v[x] as usize] {
                    state.program_counter += 2;
                }
            }
            _ if (opcode & 0xF0FF) == 0xE0A1 => {
                let (x, _) = opcode.get_xy();
                debug!(
                    "{:#06x} Skip next instruction if key with the value of V[{x}] is not pressed",
                    opcode
                );
                if !keys[state.v[x] as usize] {
                    state.program_counter += 2;
                }
            }
            _ if (opcode & 0xF0FF) == 0xF007 => {
                debug!("{:#06x} Set Vx = delay timer value", opcode);
                let (x, _) = opcode.get_xy();
                state.v[x] = state.delay_timer as u8;
            }
            _ if (opcode & 0xF0FF) == 0xF00A => {
                let (x, _) = opcode.get_xy();
                debug!(
                    "{:#06x} Wait for a key press, store the value of the key in V[{x}]",
                    opcode
                );
                for i in 0u8..16 {
                    if keys[i as usize] {
                        state.v[x] = i;
                        state.program_counter += 2;
                        return;
                    }
                }
                return;
            }
            _ if (opcode & 0xF0FF) == 0xF015 => {
                debug!("{:#06x} Set delay timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.delay_timer = state.v[x]
            }
            _ if (opcode & 0xF0FF) == 0xF018 => {
                debug!("{:#06x} Set sound timer = Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.sound_timer = state.v[x] as u16;
            }
            _ if (opcode & 0xF0FF) == 0xF01E => {
                debug!("{:#06x} Set I = I + Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.i = state.i + (state.v[x] as u16);
            }
            _ if (opcode & 0xF0FF) == 0xF029 => {
                debug!("{:#06x} Set I = location of sprite for digit Vx", opcode);
                let (x, _) = opcode.get_xy();
                state.i = (state.v[x] as u16) * 5;
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
                let offset = state.i as usize;
                let (x, _) = opcode.get_xy();
                debug!(
                    "{:#06x} Store registers V0 through V[{x}] in memory starting at location I({:#06x})",
                    opcode, state.i
                );
                state.memory[offset..offset + 1 + x].copy_from_slice(&state.v[0..x + 1]);
            }
            _ if (opcode & 0xF0FF) == 0xF065 => {
                let (x, _) = opcode.get_xy();
                debug!(
                    "{:#06x} Read registers V0 through V[{x}] from memory starting at location I",
                    opcode
                );
                let offset = state.i as usize;
                state.v[0..x + 1].copy_from_slice(&state.memory[offset..offset + x + 1]);
            }
            _ => {
                warn!("{:#06x} Unknown instruction", opcode);
            }
        };
        state.program_counter += 2;
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
