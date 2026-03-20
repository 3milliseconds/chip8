const std = @import("std");
const Io = std.Io;

const chip8 = @import("chip8");

pub fn main(init: std.process.Init) !void {
    const inputPath = "input/IBM Logo.ch8";
    const io = init.io;
    const file = try std.Io.Dir.cwd().openFile(io, inputPath, .{ .mode = .read_only });
    defer file.close(io);
    var state: State = .init();
    const program_slice = state.memory[0x200..];
    file.reader(io, program_slice);
}

const State = struct {
    memory: [1024 * 4]u8,
    V: [16]u8,
    I: u8,
    PC: u16,
    SP: u16,

    pub fn init() State {
        return .{ .memory = [_]u8{0} ** (1024 * 4), .V = [_]u8{0} ** 16, .I = 0, .PC = 0, .SP = 0 };
    }
};
