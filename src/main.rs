mod uxn;
mod assembler;

use crate::uxn::{Uxn, Opcode, InstructionMode};

fn print_int(value: &Box<i32>) {
    println!("{}", value);
}

/*
func change_int(a: *i32) {
  *a = *a + 1;
}
 */

fn change_int_box(value: &mut Box<i32>) {
    let a: i32 = value.as_ref().clone();
    *(value.as_mut()) = a + 1;
}

fn print_int_ref(value: &i32) {
    println!("{}", value);
}

fn change_int_ref(value: &mut i32) {
    *value = *value + 1;
}

fn main() {
    // let mut uxn = Uxn::new();
    // let lit = (Opcode::LIT as u8) | u8::from(InstructionMode::Keep);
    // uxn.boot();
    // uxn.load_program(&[
    //     lit, 0x10, Opcode::DUP as u8,
    //     lit, 0x20, Opcode::ADD as u8,
    //     lit, 0xff, lit, 0x0f, Opcode::DEO as u8, 0x00
    // ], 0x100);
    // let ret = uxn.eval(0x100);
    //
    // println!("{:?}", ret);

    let mut a_unboxed = 5;
    print_int_ref(&a_unboxed);
    change_int_ref(&mut a_unboxed);
    print_int_ref(&a_unboxed);
    change_int_ref(&mut a_unboxed);
    print_int_ref(&a_unboxed);

    println!("----");

    let mut a = Box::new(5);
    print_int(&a);
    change_int_box(&mut a);
    print_int(&a);
    change_int_box(&mut a);
    print_int(&a);
}
