use crate::args::CallTarget;
use crate::instructions::{create_instruction_palette, fill_space};
use anyhow::{Context, Result, anyhow};
use goblin::pe::PE;
use iced_x86::{Code, Decoder, DecoderOptions, Instruction};

// "RNG"
fn rng(seed: i32) -> i32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

pub fn patch_instructions(pe: &PE, mut buffer: Vec<u8>, mode: CallTarget) -> Result<Vec<u8>> {
    let text_section = pe
        .sections
        .iter()
        .find(|s| s.name.starts_with(b".text"))
        .ok_or(anyhow!("[!] No .text section found"))?;

    // Calculate offsets
    let file_start = text_section.pointer_to_raw_data as usize;
    let raw_size = text_section.size_of_raw_data as usize;

    // The Base RVA of the section (e.g., 0x1000)
    let section_rva = text_section.virtual_address as u64;

    // Slice out .text for the decoder
    let text_bytes = &buffer[file_start..file_start + raw_size];

    // Initialize Decoder
    // We treat the IP as the RVA so instr.ip() returns the RVA directly
    let mut decoder = Decoder::with_ip(64, text_bytes, section_rva, DecoderOptions::NONE);
    let mut instructions = vec![];
    let mut instruction = Instruction::default();

    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        instructions.push(instruction);
    }

    let mut call_count: usize = 0;
    let mut cc_chain_count: usize = 0;
    let mut ret_count: usize = 0;
    let mut idx = 0;

    // We could probably do this at compile time but I don't care
    let random_ins = create_instruction_palette()?;

    while idx < instructions.len() {
        let instr = &instructions[idx];
        let offset_in_file = file_start + (instr.ip() - section_rva) as usize;

        match instr.code() {
            Code::Int3 => {
                let mut cc_len = 1;

                while let Some(next_instr) = instructions.get(idx + cc_len) {
                    if next_instr.code() == Code::Int3 {
                        cc_len += 1;
                    } else {
                        break;
                    }
                }

                // (try to) avoid false positives
                if cc_len > 2 {
                    let rand_ins = fill_space(cc_len, &random_ins);
                    assert_eq!(cc_len, rand_ins.len());
                    buffer[offset_in_file..offset_in_file + cc_len].copy_from_slice(&rand_ins);
                    cc_chain_count += 1;
                }

                idx += cc_len;
                continue;
            }

            // Call
            Code::Call_rel32_64 => {
                call_count += 1;
                // INT 3
                buffer[offset_in_file] = 0xCC;
                if mode.encrypted() {
                    // Fetch the offset of the bytes indicating the call's target
                    let disp_file_offset = offset_in_file + 1;
                    // Set up the RVA to be int 3 + 1 (i.e the start of the target bytes)
                    let disp_rva = (instr.ip() + 1) as i32;

                    // Fetch the original call target offset
                    let raw_target_bytes = &buffer[disp_file_offset..disp_file_offset + 4];
                    let original_target =
                        i32::from_le_bytes(raw_target_bytes.try_into().context(format!(
                            "Call at file offset {} had a failing target",
                            offset_in_file
                        ))?);

                    // Generate the random mask and xor it
                    let mask = rng(disp_rva);
                    let encrypted_target = original_target ^ mask;

                    // Save the new call target
                    buffer[disp_file_offset..disp_file_offset + 4]
                        .copy_from_slice(&encrypted_target.to_le_bytes());
                }
            }

            // Ret
            Code::Retnq | Code::Retnw => {
                ret_count += 1;
                // 0xF1 -> ICEBP | int 1 -> EXCEPTION_SINGLE_STEP
                buffer[offset_in_file] = 0xF1;
            }
            _ => {}
        }
        idx += 1;
    }

    println!(
        "[+] Modified:\n \\__{} virtualized calls\n  \\__{} virtualized rets\n   \\__{} 0xcc chains replaced",
        call_count, ret_count, cc_chain_count
    );
    return Ok(buffer);
}
