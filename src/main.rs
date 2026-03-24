use log::info;
fn main() {
    env_logger::init();
    let input = std::fs::read("input/IBM Logo.ch8").unwrap();
    let mut cpu = CpuState::new();
    cpu.memory[0x200..(0x200 + input.len())].copy_from_slice(&input);
    start_cpu(&mut cpu);
}

fn start_cpu(state: &mut CpuState) {
    let mut count = 0;
    while count < 10 {
        let pc = state.pc as usize;
        let opcode: u16 = ((state.memory[pc] as u16) << 8) | (state.memory[pc + 1] as u16);
        info!("Instruction: {:#x}", opcode);
        match opcode {
            0x00E0 => {
                info!("Clear the display.")
            }
            0x00EE => {
                info!("return from sub routine")
            }
            0x1000..=0x1FFF => {
                info!("sets the program counter to nnn.");
                state.pc = opcode & 0x0FFF;
            }
            0x2000..=0x2FFF => {
                info!("Call subroutine at nnn.")
            }
            0x3000..=0x3FFF => {
                info!("Skip next instruction if Vx = kk.");
                let (x, kk) = opcode.get_xkk();
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            0x4000..=0x4FFF => {
                info!("Skip next instruction if Vx != kk.");
                let (x, kk) = opcode.get_xkk();
                if state.v[x] == kk {
                    state.pc += 2;
                }
            }
            0x5000..=0x5FFF => {
                info!("Skip next instruction if Vx = Vy.");
                let (x, y) = opcode.get_xy();
                if state.v[x] == state.v[y] {
                    state.pc += 2;
                }
            }
            0x6000..=0x6FFF => {
                info!("Set Vx = kk.");
                let (x, kk) = opcode.get_xkk();
                state.v[x] = kk;
            }
            0x7000..=0x7FFF => {
                info!("Set Vx = Vx + kk.");
                let (x, kk) = opcode.get_xkk();
                state.v[x as usize] += kk;
            }
            _ if (opcode & 0xF00F) == 0x8000 => {
                info!("Set Vx = Vy.");
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8001 => {
                info!("Set Vx = Vx OR Vy.");
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] ^ state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8003 => {
                info!("Set Vx = Vx AND Vy.");
                let (x, y) = opcode.get_xy();
                state.v[x as usize] = state.v[x as usize] & state.v[y as usize];
            }
            _ if (opcode & 0xF00F) == 0x8004 => {
                info!("Set Vx = Vx + Vy, set VF = carry.");
                let (x, y) = opcode.get_xy();
                let carry;
                (state.v[x], carry) = state.v[x].overflowing_add(state.v[y]);
                state.v[0xf] = if carry { 1 } else { 0 }
            }
            _ if (opcode & 0xF00F) == 0x8005 => {
                info!("Set Vx = Vx - Vy, set VF = NOT borrow.");
                let (x, y) = opcode.get_xy();
                let overflow: bool;
                (state.v[x], overflow) = state.v[x].overflowing_sub(state.v[y]);
                state.v[0xf] = if !overflow { 1 } else { 0 };
            }
            _ if (opcode & 0xF00F) == 0x8006 => {
                info!("Set Vx = Vx SHR 1.");
                let (x, _) = opcode.get_xy();
                state.v[0xf] = if state.v[x] & 0x1 == 1 { 1 } else { 0 };
                state.v[0xf] = state.v[0xf] >> 1;
            }
            _ if (opcode & 0xF00F) == 0x8007 => info!("Set Vx = Vy - Vx, set VF = NOT borrow."),
            0x8000..=0x8FFF => info!("Set Vx = Vx SHL 1."),
            0x9000..=0x9FFF => info!("Skip next instruction if Vx != Vy."),
            0xA000..=0xAFFF => info!("Set I = nnn."),
            0xB000..=0xBFFF => info!("Jump to location nnn + V0."),
            0xC000..=0xCFFF => info!("Set Vx = random byte AND kk."),
            0xD000..=0xDFFF => info!(
                "Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision."
            ),
            _ if (opcode & 0xF0FF) == 0xE09E => {
                info!("Skip next instruction if key with the value of Vx is pressed.")
            }
            _ if (opcode & 0xF0FF) == 0xE0A1 => {
                info!("Skip next instruction if key with the value of Vx is not pressed.")
            }
            _ if (opcode & 0xF0FF) == 0xF007 => {
                info!("Set Vx = delay timer value.")
            }
            _ if (opcode & 0xF0FF) == 0xF00A => {
                info!("Wait for a key press, store the value of the key in Vx.")
            }
            _ if (opcode * 0xF0FF) == 0xF015 => info!("Set delay timer = Vx."),
            _ if (opcode * 0xF0FF) == 0xF018 => info!("Set sound timer = Vx."),
            _ if (opcode * 0xF0FF) == 0xF01E => info!("Set I = I + Vx."),
            _ if (opcode * 0xF0FF) == 0xF029 => {
                info!("Set I = location of sprite for digit Vx.")
            }
            _ if (opcode * 0xF0FF) == 0xF033 => {
                info!("Store BCD representation of Vx in memory locations I, I+1, and I+2.")
            }
            _ if (opcode * 0xF0FF) == 0xF055 => {
                info!("Store registers V0 through Vx in memory starting at location I.")
            }
            _ if (opcode * 0xF0FF) == 0xF065 => {
                info!("Read registers V0 through Vx from memory starting at location I.")
            }

            _ => info!("got something else"),
        };
        count += 1;
        state.pc += 2;
    }
}

struct CpuState {
    memory: [u8; 4096],
    v: [u8; 16],
    ix: u16,
    dt: u16,
    st: u16,
    pc: u16,
    sp: u8,
}

impl CpuState {
    fn new() -> CpuState {
        return CpuState {
            memory: [0; 4096],
            v: [0; 16],
            ix: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            sp: 0,
        };
    }
}

trait OpCode {
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
}
