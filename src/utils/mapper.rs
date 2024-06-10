use indexmap::IndexMap;
// use starknet_api::state::EntryPoint;
use starknet_api::core::EntryPointSelector;
use starknet_api::deprecated_contract_class::{EntryPoint, EntryPointOffset, EntryPointType};
use starknet_api::hash::StarkFelt;
use starknet_core::types::contract::legacy::{
    LegacyEntrypointOffset, RawLegacyAbiEntry, RawLegacyEntryPoint, RawLegacyEntryPoints, RawLegacyEvent,
    RawLegacyFunction, RawLegacyMember, RawLegacyStruct,
};
use starknet_core::types::{LegacyContractAbiEntry, LegacyEntryPointsByType, LegacyStructMember};

// pub fn map_builtins(p: &Program) -> Vec<BuiltinNameSubxt> {
//     let mut builtins: Vec<BuiltinNameSubxt> = vec![];
//
//     for x in &p.builtins {
//         builtins.push(match x {
//             BuiltinName::range_check => BuiltinNameSubxt::range_check,
//             BuiltinName::pedersen => BuiltinNameSubxt::pedersen,
//             BuiltinName::poseidon => BuiltinNameSubxt::poseidon,
//             BuiltinName::ec_op => BuiltinNameSubxt::ec_op,
//             BuiltinName::bitwise => BuiltinNameSubxt::bitwise,
//             BuiltinName::ecdsa => BuiltinNameSubxt::ecdsa,
//             BuiltinName::keccak => BuiltinNameSubxt::keccak,
//             BuiltinName::output => BuiltinNameSubxt::output,
//             BuiltinName::segment_arena => BuiltinNameSubxt::segment_arena,
//         });
//     }
//
//     builtins
// }
//
// pub fn map_constants(p: &Program) -> Vec<(String, Felt252Subxt)> {
//     let mut constants: Vec<(String, Felt252Subxt)> = vec![];
//
//     for (k, v) in &p.constants {
//         let new_felt = v.to_be_bytes();
//         let obj = Felt252Subxt { value: FeltBigIntSubxt { value: new_felt.to_vec() } };
//         constants.push((k.to_string(), obj));
//     }
//
//     constants
// }
//
// pub fn map_data(p: &Program) -> Vec<MaybeRelocatableSubxt> {
//     let mut data: Vec<MaybeRelocatableSubxt> = vec![];
//
//     // log::debug!("{:?}", p.data());
//
//     for x in p.data() {
//         match x {
//             MaybeRelocatable::RelocatableValue(val) =>
// data.push(MaybeRelocatableSubxt::RelocatableValue {                 0: RelocatableSubxt {
// segment_index: val.segment_index as i64, offset: val.offset as u64 },             }),
//             MaybeRelocatable::Int(val) => data.push(MaybeRelocatableSubxt::Int {
//                 0: Felt252Subxt { value: FeltBigIntSubxt { value: val.to_bytes_be() } },
//             }),
//         };
//     }
//     // log::debug!("{:?}", data);
//
//     data
// }
//
// pub fn map_hints(p: &Program) -> Vec<HintParamsSubxt> {
//     let mut hints: Vec<HintParamsSubxt> = vec![];
//
//     for x in p.hints() {
//         let ref_ids = x.flow_tracking_data.reference_ids.clone();
//         let mut ref_ids_vec: Vec<(String, u64)> = vec![];
//         for (x, y) in ref_ids {
//             ref_ids_vec.push((x, y as u64));
//         }
//
//         hints.push(HintParamsSubxt {
//             code: x.code.clone(),
//             accessible_scopes: x.accessible_scopes.clone(),
//             flow_tracking_data: FlowTrackingDataSubxt {
//                 ap_tracking: ApTrackingSubxt {
//                     group: x.flow_tracking_data.ap_tracking.group.clone() as u64,
//                     offset: x.flow_tracking_data.ap_tracking.offset.clone() as u64,
//                 },
//                 reference_ids: ref_ids_vec,
//             },
//         })
//     }
//
//     hints
// }
//
// pub fn map_hints_ranges(p: &Program) -> Vec<(u64, u64)> {
//     let mut hints_ranges: Vec<(u64, u64)> = vec![];
//
//     for x in p.hints_ranges() {
//         match x {
//             Some((x, y)) => {
//                 hints_ranges.push((x.clone() as u64, y.get() as u64));
//             },
//             None => {}
//         }
//     }
//
//     hints_ranges
// }
//
// pub fn map_main(p: &Program) -> Option<u64> {
//     match p.main() {
//         Some(val) => {
//             Some(val.clone() as u64)
//         },
//         None => {
//             None
//         }
//     }
// }
// pub fn map_program_start(p: &Program) -> Option<u64> {
//     match p.start() {
//         Some(val) => {
//             Some(val.clone() as u64)
//         },
//         None => {
//             None
//         }
//     }
// }
// pub fn map_program_end(p: &Program) -> Option<u64> {
//     match p.end() {
//         Some(val) => {
//             Some(val.clone() as u64)
//         },
//         None => {
//             None
//         }
//     }
// }
// pub fn map_error_message_attributes(p: &Program) -> Vec<AttributeSubxt> {
//     let mut error_message_attribute: Vec<AttributeSubxt> = vec![];
//
//     for x in p.error_message_attributes() {
//         let ref_ids = x.flow_tracking_data.clone().unwrap().reference_ids;
//         let mut ref_ids_vec: Vec<(String, u64)> = vec![];
//         for (x, y) in ref_ids {
//             ref_ids_vec.push((x, y as u64));
//         }
//
//         error_message_attribute.push(AttributeSubxt {
//             name: x.name.clone(),
//             start_pc: x.start_pc.clone() as u64,
//             end_pc: x.end_pc.clone() as u64,
//             value: x.value.clone(),
//             flow_tracking_data: Some(FlowTrackingDataSubxt {
//                 ap_tracking: ApTrackingSubxt {
//                     group: x.flow_tracking_data.clone().unwrap().ap_tracking.group as u64,
//                     offset: x.flow_tracking_data.clone().unwrap().ap_tracking.offset as u64,
//                 },
//                 reference_ids: ref_ids_vec,
//             }),
//         })
//     }
//
//     error_message_attribute
// }
//
// pub fn map_instruction_locations(p: &Program) -> Option<Vec<(u64, InstructionLocationSubxt)>> {
//     let mut instruction_locations: Vec<(u64, InstructionLocationSubxt)> = vec![];
//
//     for x in p.instruction_locations().clone().unwrap() {
//         instruction_locations.push((
//             x.0.clone() as u64,
//             InstructionLocationSubxt { inst: map_location_internal(x.1.inst.clone()), hints:
// vec![] },         ))
//     }
//
//     Some(instruction_locations)
// }
//
// #[allow(unconditional_recursion)]
// fn map_location_internal(l: Location) -> LocationSubxt {
//     LocationSubxt {
//         end_line: l.end_line.clone(),
//         end_col: l.end_col.clone(),
//         input_file: InputFileSubxt { filename: l.input_file.filename },
//         parent_location: match l.parent_location {
//             Some(val) => {
//                 Box::new(Some((
//                     map_location_internal(val.0.deref().clone()),
//                     val.1.clone(),
//                 )))
//             },
//             None => {
//                 Box::new(None)
//             }
//         },
//         start_line: l.start_line,
//         start_col: l.start_col,
//     }
// }
//
// pub fn map_identifiers(p: &Program) -> Vec<(String, IdentifierSubxt)> {
//     let mut vec_identifiers: Vec<(String, IdentifierSubxt)> = vec![];
//
//     for x in p.identifiers() {
//         let mut members_vec: Vec<(String, MemberSubxt)> = vec![];
//         match x.1.clone().members {
//             Some(val) => {
//                 for (x, y) in val {
//                     members_vec.push((x, MemberSubxt { cairo_type: y.cairo_type.clone(), offset:
// y.offset.clone() as u64 }));                 }
//             },
//             None => {}
//         }
//         vec_identifiers.push((
//             x.0.clone(),
//             IdentifierSubxt {
//                 pc: match x.1.clone().pc {
//                     Some(val) => Some(val as u64),
//                     None => None
//                 },
//                 type_: match x.1.clone().type_ {
//                     Some(val) => Some(val),
//                     None => None
//                 },
//                 value: match x.1.clone().value {
//                     Some(val) => {
//                         Some(Felt252Subxt {
//                             value: FeltBigIntSubxt {
//                                 value: val.to_bytes_be()
//                             }
//                         })
//                     },
//                     None => None
//                 },
//                 full_name: match x.1.clone().full_name {
//                     Some(val) => {
//                         Some(val)
//                     },
//                     None => None
//                 },
//                 members: Some(members_vec),
//                 cairo_type: match x.1.clone().cairo_type {
//                     Some(val) => {
//                         Some(val)
//                     },
//                     None => None
//                 }
//             },
//         ));
//     }
//
//     vec_identifiers
// }
//
// pub fn map_reference_manager(p: &Program) -> Vec<HintReferenceSubxt> {
//     let mut vec_reference_manager: Vec<HintReferenceSubxt> = vec![];
//     let ref_mngr = p.reference_manager().references;
//     for x in ref_mngr {
//         vec_reference_manager.push(HintReferenceSubxt {
//             offset1: match x.value_address.offset1 {
//                 OffsetValue::Immediate(val) => {
//                     OffsetValueSubxt::Immediate(Felt252Subxt { value: FeltBigIntSubxt { value:
// val.to_bytes_be() } })                 }
//                 OffsetValue::Value(val) => OffsetValueSubxt::Value(val),
//                 OffsetValue::Reference(reg, val, bool_val) => OffsetValueSubxt::Reference(
//                     match reg {
//                         Register::AP => RegisterSubxt::AP,
//                         Register::FP => RegisterSubxt::FP,
//                     },
//                     val,
//                     bool_val,
//                 ),
//             },
//             offset2: match x.value_address.offset2 {
//                 OffsetValue::Immediate(val) => {
//                     OffsetValueSubxt::Immediate(Felt252Subxt { value: FeltBigIntSubxt { value:
// val.to_bytes_be() } })                 }
//                 OffsetValue::Value(val) => OffsetValueSubxt::Value(val),
//                 OffsetValue::Reference(reg, val, bool_val) => OffsetValueSubxt::Reference(
//                     match reg {
//                         Register::AP => RegisterSubxt::AP,
//                         Register::FP => RegisterSubxt::FP,
//                     },
//                     val,
//                     bool_val,
//                 ),
//             },
//             dereference: x.value_address.dereference,
//             ap_tracking_data: Some(ApTrackingSubxt {
//                 group: x.ap_tracking_data.group as u64,
//                 offset: x.ap_tracking_data.offset as u64,
//             }),
//             cairo_type: Some(x.value_address.value_type),
//         })
//     }
//
//     vec_reference_manager
// }

pub fn to_raw_legacy_entrypoint(entrypoints: LegacyEntryPointsByType) -> RawLegacyEntryPoints {
    let mut vec_constructor: Vec<RawLegacyEntryPoint> = vec![];
    let mut vec_external: Vec<RawLegacyEntryPoint> = vec![];
    let mut vec_l1_handler: Vec<RawLegacyEntryPoint> = vec![];

    for x in entrypoints.constructor {
        vec_constructor
            .push(RawLegacyEntryPoint { offset: LegacyEntrypointOffset::U64AsInt(x.offset), selector: x.selector })
    }
    for x in entrypoints.external {
        vec_external
            .push(RawLegacyEntryPoint { offset: LegacyEntrypointOffset::U64AsInt(x.offset), selector: x.selector })
    }
    for x in entrypoints.l1_handler {
        vec_l1_handler
            .push(RawLegacyEntryPoint { offset: LegacyEntrypointOffset::U64AsInt(x.offset), selector: x.selector })
    }

    RawLegacyEntryPoints { constructor: vec_constructor, external: vec_external, l1_handler: vec_l1_handler }
}

pub fn map_entrypoint_selector(entrypoints: RawLegacyEntryPoints) -> IndexMap<EntryPointType, Vec<EntryPoint>> {
    let mut vec_entrypoints: IndexMap<EntryPointType, Vec<EntryPoint>> = IndexMap::new();

    let mut vec_constructor: Vec<EntryPoint> = vec![];
    let mut vec_external: Vec<EntryPoint> = vec![];
    let mut vec_l1_handler: Vec<EntryPoint> = vec![];

    for x in entrypoints.constructor {
        vec_constructor.push(gen_val_entrypoint(x))
    }
    for x in entrypoints.external {
        vec_external.push(gen_val_entrypoint(x))
    }
    for x in entrypoints.l1_handler {
        vec_l1_handler.push(gen_val_entrypoint(x))
    }

    vec_entrypoints.insert(EntryPointType::Constructor, vec_constructor);
    vec_entrypoints.insert(EntryPointType::L1Handler, vec_l1_handler);
    vec_entrypoints.insert(EntryPointType::External, vec_external);

    vec_entrypoints
}

fn gen_val_entrypoint(x: RawLegacyEntryPoint) -> EntryPoint {
    EntryPoint {
        selector: EntryPointSelector(StarkFelt(x.selector.to_bytes_be())),
        offset: match x.offset {
            LegacyEntrypointOffset::U64AsHex(val) => EntryPointOffset(val),
            LegacyEntrypointOffset::U64AsInt(val) => EntryPointOffset(val),
        },
    }
}

pub fn abi_mapper(abi: Vec<LegacyContractAbiEntry>) -> Vec<RawLegacyAbiEntry> {
    let mut new_abi: Vec<RawLegacyAbiEntry> = Vec::new();

    for x in abi {
        match x {
            LegacyContractAbiEntry::Event(val) => {
                new_abi.push(RawLegacyAbiEntry::Event(RawLegacyEvent {
                    data: val.data,
                    keys: val.keys,
                    name: val.name,
                }));
            }
            LegacyContractAbiEntry::Function(val) => {
                new_abi.push(RawLegacyAbiEntry::Function(RawLegacyFunction {
                    inputs: val.inputs,
                    name: val.name,
                    state_mutability: val.state_mutability,
                    outputs: val.outputs,
                }));
            }
            LegacyContractAbiEntry::Struct(val) => {
                new_abi.push(RawLegacyAbiEntry::Struct(RawLegacyStruct {
                    members: map_abi_struct_members(val.members),
                    name: val.name,
                    size: val.size,
                }));
            }
        }
    }

    new_abi
}

fn map_abi_struct_members(members: Vec<LegacyStructMember>) -> Vec<RawLegacyMember> {
    let mut members_mapped: Vec<RawLegacyMember> = Vec::new();

    for x in members {
        members_mapped.push(RawLegacyMember { name: x.name, offset: x.offset, r#type: x.r#type })
    }

    members_mapped
}
