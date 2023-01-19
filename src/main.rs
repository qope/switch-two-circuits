use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::poseidon::PoseidonHash,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

use std::time::Instant;

mod circuit_maker;

use circuit_maker::{make_inner_circuit0, make_inner_circuit1, CircuitType};

use crate::circuit_maker::{Circuit, DummyCircuit};

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    type H = PoseidonHash;
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    let mut pw = PartialWitness::<F>::new();

    let circuit0 = make_inner_circuit0::<C, F, H, D>();
    let circuit1 = make_inner_circuit1::<C, F, D>();

    let DummyCircuit(dummy_data0, dummy_proof0) = circuit0.prove_dummy();
    let DummyCircuit(dummy_data1, dummy_proof1) = circuit1.prove_dummy();

    let Circuit(_, data0, _) = &circuit0;
    let Circuit(_, data1, _) = &circuit1;
    let pt0 = builder.add_virtual_proof_with_pis::<C>(&data0.common);
    builder.register_public_inputs(&pt0.public_inputs);
    let pt1 = builder.add_virtual_proof_with_pis::<C>(&data1.common);
    builder.register_public_inputs(&pt1.public_inputs);
    let vc0 = builder.add_virtual_verifier_data(data0.common.config.fri_config.cap_height);
    let vc1 = builder.add_virtual_verifier_data(data1.common.config.fri_config.cap_height);

    builder.verify_proof::<C>(&pt0, &vc0, &data0.common);
    builder.verify_proof::<C>(&pt1, &vc1, &data1.common);

    let verify_circuit = CircuitType::Circuit1;

    let time = match verify_circuit {
        CircuitType::Circuit0 => {
            let now = Instant::now();
            // circuit0のproofを作る
            let proof = circuit0.prove().unwrap();
            pw.set_proof_with_pis_target(&pt0, &proof);
            pw.set_verifier_data_target(&vc0, &data0.verifier_only);

            // circuit1のdummy proofをset
            pw.set_proof_with_pis_target(&pt1, &dummy_proof1);
            pw.set_verifier_data_target(&vc1, &dummy_data1.verifier_only);
            now.elapsed().as_millis()
        }
        CircuitType::Circuit1 => {
            let now = Instant::now();
            // circuit1のproofを作る
            let proof = circuit1.prove().unwrap();
            pw.set_proof_with_pis_target(&pt1, &proof);
            pw.set_verifier_data_target(&vc1, &data1.verifier_only);

            // circuit0のdummy proofをset
            pw.set_proof_with_pis_target(&pt0, &dummy_proof0);
            pw.set_verifier_data_target(&vc0, &dummy_data0.verifier_only);
            now.elapsed().as_millis()
        }
    };
    dbg!(time);
    let data = builder.build::<C>();
    let proof = data.prove(pw).unwrap();
    dbg!(data.common.degree_bits());
    dbg!(proof.public_inputs);
}
