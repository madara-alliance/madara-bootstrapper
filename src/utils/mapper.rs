use cairo_vm::felt::Felt252;
use cairo_vm::serde::deserialize_program::{Attribute, BuiltinName, HintParams, Identifier, ReferenceManager};
use cairo_vm::types::program::Program;

use crate::contract_clients::subxt_funcs::appchain::runtime_types::cairo_felt::lib_bigint_felt::Felt252 as Felt252Subxt;
use crate::contract_clients::subxt_funcs::appchain::runtime_types::cairo_vm::serde::deserialize_program::BuiltinName as BuiltinNameSubxt;

pub fn map_builtins(p: &Program) -> Vec<BuiltinNameSubxt> {
    let mut builtins: Vec<BuiltinNameSubxt> = vec![];

    for x in &p.builtins {
        builtins.push(match x {
            BuiltinName::range_check => BuiltinNameSubxt::range_check,
            BuiltinName::pedersen => BuiltinNameSubxt::pedersen,
            BuiltinName::poseidon => BuiltinNameSubxt::poseidon,
            BuiltinName::ec_op => BuiltinNameSubxt::ec_op,
            BuiltinName::bitwise => BuiltinNameSubxt::bitwise,
            BuiltinName::ecdsa => BuiltinNameSubxt::ecdsa,
            BuiltinName::keccak => BuiltinNameSubxt::keccak,
            BuiltinName::output => BuiltinNameSubxt::output,
            BuiltinName::segment_arena => BuiltinNameSubxt::segment_arena,
        });
    }

    builtins
}

pub fn map_constants(p: &Program) -> Vec<(String, Felt252Subxt)> {
    let mut constants: Vec<(String, Felt252Subxt)> = vec![];

    for (k, v) in &p.constants {
        let new_felt = v.to_be_bytes();
        // ! TO_FIX
        // constants.push((k.to_string(), Felt252Subxt::from).unwrap()));
    }

    constants
}
