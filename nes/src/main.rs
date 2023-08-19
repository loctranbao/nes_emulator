pub mod opcodes;
pub mod cpu;

#[allow(overflowing_literals)]
fn main() {
    let value = 0xf8 as i8;

    println!("{}", value as u16);
}