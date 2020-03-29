mod common;
mod version_gte3;
mod version_gte4;
mod version_gte5;

use std::collections::HashMap;

use crate::game::instruction::OpCode::{Extended, OneOp, TwoOp, VarOp, ZeroOp};
use crate::game::instruction::{Instruction, OpCode};

/// Represents all the instructions available to the Z-Machine version specified in the game file.
pub struct InstructionSet {
    instructions: HashMap<OpCode, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<OpCode, Instruction> = [
            (TwoOp(0x1), Instruction::Branch(&common::je, "JE")),
            (TwoOp(0x2), Instruction::Branch(&common::jl, "JL")),
            (TwoOp(0x3), Instruction::Branch(&common::jg, "JG")),
            (TwoOp(0x4), Instruction::Branch(&common::dec_chk, "DEC_CHK")),
            (TwoOp(0x5), Instruction::Branch(&common::inc_chk, "INC_CHK")),
            (TwoOp(0x6), Instruction::Branch(&common::jin, "JIN")),
            (TwoOp(0x8), Instruction::Store(&common::or, "OR")),
            (TwoOp(0x9), Instruction::Store(&common::and, "AND")),
            (TwoOp(0xD), Instruction::Normal(&common::store, "STORE")),
            (TwoOp(0xF), Instruction::Store(&common::loadw, "LOADW")),
            (TwoOp(0x10), Instruction::Store(&common::loadb, "LOADB")),
            (
                TwoOp(0x11),
                Instruction::Store(&common::get_prop, "GET_PROP"),
            ),
            (
                TwoOp(0x12),
                Instruction::Store(&common::get_prop_addr, "GET_PROP_ADDR"),
            ),
            (TwoOp(0x14), Instruction::Store(&common::add, "ADD")),
            (TwoOp(0x15), Instruction::Store(&common::sub, "SUB")),
            (TwoOp(0x16), Instruction::Store(&common::mul, "MUL")),
            (TwoOp(0x17), Instruction::Store(&common::div, "DIV")),
            (TwoOp(0x18), Instruction::Store(&common::z_mod, "Z_MOD")),
            (OneOp(0x0), Instruction::Branch(&common::jz, "JZ")),
            (OneOp(0x5), Instruction::Normal(&common::inc, "INC")),
            (OneOp(0x6), Instruction::Normal(&common::dec, "DEC")),
            (
                OneOp(0xA),
                Instruction::Normal(&common::print_obj, "PRINT_OBJ"),
            ),
            (OneOp(0xB), Instruction::Normal(&common::ret, "RET")),
            (OneOp(0xC), Instruction::Normal(&common::jump, "JUMP")),
            (
                OneOp(0xD),
                Instruction::Normal(&common::print_paddr, "PRINT_PADDR"),
            ),
            (OneOp(0xE), Instruction::Store(&common::load, "LOAD")),
            (OneOp(0xF), Instruction::Store(&common::not, "NOT")), // Moved in V5
            (ZeroOp(0x0), Instruction::Normal(&common::rtrue, "RTRUE")),
            (ZeroOp(0x1), Instruction::Normal(&common::rfalse, "RFALSE")),
            (
                ZeroOp(0x2),
                Instruction::StringLiteral(&common::print, "PRINT"),
            ),
            (
                ZeroOp(0x3),
                Instruction::StringLiteral(&common::print_ret, "PRINT_RET"),
            ),
            (ZeroOp(0x4), Instruction::Normal(&common::nop, "NOP")),
            (
                ZeroOp(0x8),
                Instruction::Normal(&common::ret_popped, "RET_POPPED"),
            ),
            (ZeroOp(0xA), Instruction::Normal(&common::quit, "QUIT")),
            (
                ZeroOp(0xB),
                Instruction::Normal(&common::new_line, "NEW_LINE"),
            ),
            (VarOp(0x0), Instruction::Store(&common::call, "CALL")),
            (VarOp(0x1), Instruction::Normal(&common::storew, "STOREW")),
            (VarOp(0x2), Instruction::Normal(&common::storeb, "STOREB")),
            (
                VarOp(0x6),
                Instruction::Normal(&common::print_num, "PRINT_NUM"),
            ),
            (VarOp(0x7), Instruction::Store(&common::random, "RANDOM")),
            (VarOp(0x8), Instruction::Normal(&common::push, "PUSH")),
            (VarOp(0x9), Instruction::Normal(&common::pull, "PULL")),
        ]
        .iter()
        .cloned()
        .collect();

        if version >= 3 {
            instructions.extend(
                [(
                    ZeroOp(0xD),
                    Instruction::Branch(&version_gte3::verify, "VERIFY"),
                )]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        if version >= 4 {
            instructions.extend(
                [
                    (
                        TwoOp(0x19),
                        Instruction::Store(&version_gte4::call_2s, "CALL_2S"),
                    ),
                    (
                        OneOp(0x8),
                        Instruction::Store(&version_gte4::call_1s, "CALL_1S"),
                    ),
                    (
                        VarOp(0x0),
                        Instruction::Store(&version_gte4::call_vs, "CALL_VS"),
                    ),
                    (
                        VarOp(0xC),
                        Instruction::Store(&version_gte4::call_vs2, "CALL_VS2"),
                    ),
                    (
                        VarOp(0x11),
                        Instruction::Normal(&version_gte4::set_text_style, "SRT_TEXT_STYLE"),
                    ),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        if version >= 5 {
            instructions.extend(
                [
                    (
                        OneOp(0xF),
                        Instruction::Normal(&version_gte5::call_1n, "CALL_1N"),
                    ),
                    (
                        TwoOp(0x1A),
                        Instruction::Normal(&version_gte5::call_2n, "CALL_2N"),
                    ),
                    (VarOp(0x18), Instruction::Store(&common::not, "NOT")), // Moved from 1OP:143
                    (
                        VarOp(0x19),
                        Instruction::Normal(&version_gte5::call_vn, "CALL_VN"),
                    ),
                    (
                        VarOp(0x1A),
                        Instruction::Normal(&version_gte5::call_vn2, "CALL_VN2"),
                    ),
                    (
                        VarOp(0x1F),
                        Instruction::Branch(&version_gte5::check_arg_count, "CHEC_ARG_COUNT"),
                    ),
                    (
                        Extended(0x2),
                        Instruction::Store(&version_gte5::log_shift, "LOG_SHIFT"),
                    ),
                    (
                        Extended(0x3),
                        Instruction::Store(&version_gte5::art_shift, "ART_SHIFT"),
                    ),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<OpCode, Instruction>>(),
            );
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: &OpCode) -> Option<Instruction> {
        self.instructions.get(opcode).cloned()
    }
}
