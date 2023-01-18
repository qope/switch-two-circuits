use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

mod circuit_maker;

use circuit_maker::{make_inner_circuit0, make_inner_circuit1};

pub fn make_outer_circuit<C, F, const D: usize>(
    condition: BoolTarget,
    data: [CircuitData<F, C, D>; 2],
) -> CircuitData<F, C, D>
where
    C: GenericConfig<D, F = F>,
    F: RichField + Extendable<D>,
{
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    // builder.conditionally_verify_cyclic_proof_or_dummy(
    //     condition,
    //     cyclic_proof_with_pis,
    //     common_data,
    // );
    // builder.conditionally_verify_proof(condition, proof_with_pis0, inner_verifier_data0, proof_with_pis1, inner_verifier_data1, inner_common_data)
    todo!()
}
fn main() {
    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    // let config = CircuitConfig::standard_recursion_config();
    // let mut builder = CircuitBuilder::<F, D>::new(config.clone());

    // let data0 = make_inner_circuit_sized::<C, F, D>(5);
    // let data1 = make_inner_circuit_sized::<C, F, D>(10);

    // dbg!(data0.common.fri_params);
    // dbg!(data1.common.fri_params);
    // dbg!(data0.common.fri_params == data1.common.fri_params);
}
