use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

mod circuit_maker;

use circuit_maker::{make_inner_circuit0, make_inner_circuit1, CircuitType};

use crate::circuit_maker::{Circuit, DummyCircuit};

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    let mut pw = PartialWitness::<F>::new();

    let circuit0 = make_inner_circuit0::<C, F, D>();
    let circuit1 = make_inner_circuit1::<C, F, D>();

    let Circuit(_, data0, _) = &circuit0;
    let Circuit(_, data1, _) = &circuit1;
    let pt0 = builder.add_virtual_proof_with_pis::<C>(&data0.common);
    builder.register_public_inputs(&pt0.public_inputs);
    let pt1 = builder.add_virtual_proof_with_pis::<C>(&data1.common);
    builder.register_public_inputs(&pt1.public_inputs);
    let vc0 = builder.add_virtual_verifier_data(data0.common.config.fri_config.cap_height);
    let vc1 = builder.add_virtual_verifier_data(data1.common.config.fri_config.cap_height);

    let verify_circuit = CircuitType::Circuit0;

    match verify_circuit {
        CircuitType::Circuit0 => {
            // circuit0のproofを作る
            let proof = circuit0.prove().unwrap();
            pw.set_proof_with_pis_target(&pt0, &proof);
            pw.set_verifier_data_target(&vc0, &data0.verifier_only);

            // circuit1のdummy proofを作る
            let DummyCircuit(dummy_data, dummy_proof) = circuit1.prove_dummy();
            pw.set_proof_with_pis_target(&pt1, &dummy_proof);
            pw.set_verifier_data_target(&vc1, &dummy_data.verifier_only);
        }
        CircuitType::Circuit1 => {
            // circuit1のproofを作る
            let proof = circuit1.prove().unwrap();
            pw.set_proof_with_pis_target(&pt1, &proof);
            pw.set_verifier_data_target(&vc1, &data1.verifier_only);

            // circuit0のdummy proofを作る
            let DummyCircuit(dummy_data, dummy_proof) = circuit0.prove_dummy();
            pw.set_proof_with_pis_target(&pt0, &dummy_proof);
            pw.set_verifier_data_target(&vc0, &dummy_data.verifier_only);
        }
    }
    let data = builder.build::<C>();
    let proof = data.prove(pw).unwrap();
    dbg!(data.common.degree_bits());
    dbg!(proof.public_inputs);
}
