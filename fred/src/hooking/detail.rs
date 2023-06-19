use std::{ffi::c_void, mem::MaybeUninit};

use zydis::{
    ffi::{
        DecodedOperandKind, Request, ZydisEncoderDecodedInstructionToEncoderRequest,
        ZydisEncoderEncodeInstructionAbsolute,
    },
    *,
};

#[derive(Debug)]
pub enum CodeRelocError {
    /// Failed to calculate an absolute address.
    FailedAbsoluteAddressCalc,
    /// Failed to convert a ZydisDecodedInstruction to a ZydisEncoderRequest.
    FailedDecodedInstrToEncoderRequest,
    /// Failed to encode an instruction.
    FailedEncodeInstr,
}

pub fn relocate_code(
    source: usize,
    source_size: usize,
    dest: usize,
) -> std::result::Result<Vec<u8>, CodeRelocError> {
    let decoder = Decoder::new(MachineMode::LONG_COMPAT_32, StackWidth::_32).unwrap();
    let source_slice = unsafe { std::slice::from_raw_parts(source as *const u8, source_size) };

    let mut source_offset = 0;
    let mut dest_offset = 0;
    let mut output = Vec::with_capacity(source_size);

    for instr_info in decoder.decode_all::<AllOperands>(source_slice, source as u64) {
        let (_ip, _raw_bytes, instr) = instr_info.unwrap();
        let mut operands = instr.operands().to_owned();
        // Iterate over all operands, checking if each one is EIP-relative and patching the address accordingly.
        // (The rust port of Zydis doesn't allow us to modify Instruction instances, so we have to clone the array.)
        // (Since we don't use any other high-level bindings and resort to the raw FFI functions later on, this is fine... ish)
        for operand in operands.iter_mut() {
            let original_operand = operand.clone();
            if let DecodedOperandKind::Imm(ref mut info) = operand.kind {
                if info.is_relative {
                    if let Ok(absolute_address) = instr
                        .calc_absolute_address((source + source_offset) as u64, &original_operand)
                    {
                        info.is_relative = false;
                        info.value = absolute_address;
                    } else {
                        return Err(CodeRelocError::FailedAbsoluteAddressCalc);
                    }
                }
            } else if let DecodedOperandKind::Mem(ref mut info) = operand.kind {
                if info.base == Register::EIP {
                    if let Ok(absolute_address) = instr
                        .calc_absolute_address((source + source_offset) as u64, &original_operand)
                    {
                        info.base = Register::NONE;
                        info.disp.displacement = absolute_address as i64;
                    } else {
                        return Err(CodeRelocError::FailedAbsoluteAddressCalc);
                    }
                }
            }
        }

        let old_size = output.len();

        // Reserve space in the vector for the encoded instruction.
        output.resize(old_size + MAX_INSTRUCTION_LENGTH, 0x90);

        unsafe {
            // Convert the decoded instruction to an encoder request.
            let mut encoder_request = MaybeUninit::<Request>::zeroed().assume_init();

            let status = ZydisEncoderDecodedInstructionToEncoderRequest(
                &*instr,
                operands.as_ptr(),
                instr.operand_count_visible as u8,
                &mut encoder_request,
            );
            if status.is_error() {
                return Err(CodeRelocError::FailedDecodedInstrToEncoderRequest);
            }

            // Encode the instruction into the output vector.
            let mut encoded_size = MAX_INSTRUCTION_LENGTH;
            let status = ZydisEncoderEncodeInstructionAbsolute(
                &encoder_request,
                output.as_mut_ptr().offset(dest_offset as isize) as *mut c_void,
                &mut encoded_size,
                (dest + dest_offset) as u64,
            );
            if status.is_error() {
                return Err(CodeRelocError::FailedEncodeInstr);
            }

            // Shrink the vector to fit.
            output.truncate(old_size + encoded_size);

            // Increment the destination offset by the size of the encoded instruction.
            dest_offset += encoded_size;
        }

        // Increment the source offset by the size of the original instruction.
        source_offset += _raw_bytes.len();
    }
    Ok(output)
}

pub fn get_instruction_len(ptr: *const u8) -> usize {
    let decoder = Decoder::new(MachineMode::LONG_COMPAT_32, StackWidth::_32).unwrap();
    let source_slice = unsafe { std::slice::from_raw_parts(ptr, MAX_INSTRUCTION_LENGTH) };

    if let Ok(Some(instr)) = decoder.decode_first::<AllOperands>(source_slice) {
        return instr.length as usize;
    } else {
        return 0;
    }
}

pub fn find_suitable_backup_size(base: *const u8) -> (usize, usize) {
    let decoder = Decoder::new(MachineMode::LONG_COMPAT_32, StackWidth::_32).unwrap();
    let source_slice = unsafe { std::slice::from_raw_parts(base, MAX_INSTRUCTION_LENGTH) };

    let mut offset = 0;
    let mut padded = 0;

    for instr_info in decoder.decode_all::<AllOperands>(source_slice, base as u64) {
        let (_ip, _raw_bytes, instr) = instr_info.unwrap();
        if offset >= 5 {
            break;
        }
        offset += instr.length as usize;
        padded += MAX_INSTRUCTION_LENGTH;
    }

    (offset, padded)
}
