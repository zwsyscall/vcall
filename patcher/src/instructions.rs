use anyhow::Result;
use iced_x86::code_asm::*;
use rand::{Rng, rng, seq::IndexedRandom as _};

pub struct Ins {
    pub len: usize,
    pub bytes: Vec<u8>,
    #[allow(unused)]
    pub desc: String,
}

// This returns a Vector that consists of vectors where the index number is the length of the instructions inside :-)
pub fn create_instruction_palette() -> Result<Vec<Vec<Ins>>> {
    let mut ins_ec = Vec::new();
    let one_byte = {
        let mut v = Vec::new();
        for reg_1 in vec![rax, rbx, rcx] {
            v.push(make_ins(&format!("push {:#?}", reg_1), |a| a.push(reg_1))?);
            v.push(make_ins(&format!("pop {:#?}", reg_1), |a| a.pop(reg_1))?);
        }
        v.push(make_ins("ret", |a| a.ret())?);

        for i in v.iter() {
            assert_eq!(i.len, 1)
        }
        v
    };

    let two_byte = {
        let mut v = Vec::new();
        for reg_1 in vec![eax, ebx, ecx] {
            for reg_2 in vec![eax, ebx, ecx] {
                v.push(make_ins(&format!("xor {:#?} {:#?}", reg_1, reg_2), |a| {
                    a.xor(reg_1, reg_2)
                })?);
            }
            v.push(make_ins("rdtsc", |a| a.rdtsc())?);
        }
        for i in v.iter() {
            assert_eq!(i.len, 2)
        }
        v
    };

    let three_byte = {
        let mut v = Vec::new();
        for reg_1 in vec![rax, rbx, rcx] {
            for reg_2 in vec![rax, rbx, rcx] {
                v.push(make_ins(&format!("mov {:#?} {:#?}", reg_1, reg_2), |a| {
                    a.mov(reg_1, reg_2)
                })?);
                v.push(make_ins(&format!("add {:#?} {:#?}", reg_1, reg_2), |a| {
                    a.add(reg_1, reg_2)
                })?);
                v.push(make_ins(&format!("sub {:#?} {:#?}", reg_1, reg_2), |a| {
                    a.sub(reg_1, reg_2)
                })?);
            }
            v.push(make_ins(&format!("inc {:#?}", reg_1), |a| a.inc(reg_1))?);
            v.push(make_ins(&format!("shl {:#?} ", reg_1), |a| {
                a.shl(reg_1, 1)
            })?);
            v.push(make_ins(&format!("dec {:#?} ", reg_1), |a| a.dec(reg_1))?);
        }
        for i in v.iter() {
            assert_eq!(i.len, 3)
        }
        v
    };

    let four_byte = {
        let mut v = Vec::new();
        for reg_1 in vec![ax, bx, cx] {
            v.push(make_ins(&format!("sub {:#?} 0x8", reg_1), |a| {
                a.sub(reg_1, 8)
            })?);
        }
        for reg_1 in vec![ax, bx, cx] {
            v.push(make_ins(&format!("cmp {:#?} 0", reg_1), |a| {
                a.cmp(reg_1, 0)
            })?);
        }

        for i in v.iter() {
            assert_eq!(i.len, 4)
        }
        v
    };

    let broken = {
        let mut v = Vec::new();
        v.push(Ins {
            len: 1,
            bytes: vec![0xe9],
            desc: "jmp {broken}".to_string(),
        });
        v.push(Ins {
            len: 1,
            bytes: vec![0xe8],
            desc: "call {broken}".to_string(),
        });
        v
    };

    ins_ec.push(one_byte);
    ins_ec.push(two_byte);
    ins_ec.push(three_byte);
    ins_ec.push(four_byte);
    ins_ec.push(broken);

    Ok(ins_ec)
}

fn make_ins<F>(desc: &str, mut generator: F) -> Result<Ins, IcedError>
where
    F: FnMut(&mut CodeAssembler) -> Result<(), IcedError>,
{
    let mut a = CodeAssembler::new(64)?;
    generator(&mut a)?;
    let bytes = a.assemble(0x0)?;
    Ok(Ins {
        len: bytes.len(),
        bytes,
        desc: desc.to_string(),
    })
}

pub fn fill_space(needed_len: usize, palette: &Vec<Vec<Ins>>) -> Vec<u8> {
    let mut result = Vec::with_capacity(needed_len);
    let mut rng = rng();

    // Add space for the last breaking instruction
    let target = needed_len - 1;
    let mut current_sum = 0;

    while current_sum < target {
        let limit = (target - current_sum - 1).min(3);
        let num = rng.random_range(0..=limit);
        if let Some(ins) = palette[num].choose(&mut rng) {
            assert_eq!(ins.len, ins.bytes.len());
            result.extend_from_slice(&ins.bytes);

            current_sum += num + 1;
        }
    }

    result.extend_from_slice(&palette[4].choose(&mut rng).unwrap().bytes);
    result
}
