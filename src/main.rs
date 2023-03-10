use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::{
        hash_types::{MerkleCapTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
    },
};

use std::time::Instant;

mod circuit_maker;

use circuit_maker::{make_inner_circuit0, make_inner_circuit1, CircuitType};

use crate::circuit_maker::{Circuit, DummyCircuit};

// make conditional verifyc circuit target
fn junction<F, C, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    data0a: &CircuitData<F, C, D>,
    data0b: &CircuitData<F, C, D>,
    data1a: &CircuitData<F, C, D>,
    data1b: &CircuitData<F, C, D>,
) -> (BoolTarget, VerifierCircuitTarget, VerifierCircuitTarget)
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    C::Hasher: AlgebraicHasher<F>,
{
    let [scap0a, scap0b, scap1a, scap1b] = [data0a, data0b, data1a, data1b].map(|x| {
        MerkleCapTarget(
            x.verifier_only
                .constants_sigmas_cap
                .0
                .iter()
                .cloned()
                .map(|t| builder.constant_hash(t))
                .collect::<Vec<_>>(),
        )
    });
    let [cd0a, cd0b, cd1a, cd1b] = [data0a, data0b, data1a, data1b]
        .map(|x| builder.constant_hash(x.verifier_only.circuit_digest));
    let b = builder.add_virtual_bool_target_safe();
    let b_not = builder.not(b);
    let scap0 = builder.select_cap(b, &scap0a, &scap0b);
    let cd0 = builder.select_hash(b, cd0a, cd0b);
    let scap1 = builder.select_cap(b_not, &scap1a, &scap1b);
    let cd1 = builder.select_hash(b_not, cd1a, cd1b);

    let vc0 = VerifierCircuitTarget {
        constants_sigmas_cap: scap0,
        circuit_digest: cd0,
    };
    let vc1 = VerifierCircuitTarget {
        constants_sigmas_cap: scap1,
        circuit_digest: cd1,
    };
    (b, vc0, vc1)
}

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
    let (b, vc0, vc1) = junction::<F, C, D>(&mut builder, data0, &dummy_data0, data1, &dummy_data1);

    builder.verify_proof::<C>(&pt0, &vc0, &data0.common);
    builder.verify_proof::<C>(&pt1, &vc1, &data1.common);

    let verify_circuit = CircuitType::Circuit1;

    let time = match verify_circuit {
        CircuitType::Circuit0 => {
            let now = Instant::now();
            // circuit0: real, circuit1: dummy????????????????????????
            pw.set_bool_target(b, true);
            // circuit0???proof?????????
            let proof = circuit0.prove().unwrap();
            pw.set_proof_with_pis_target(&pt0, &proof);
            pw.set_verifier_data_target(&vc0, &data0.verifier_only);
            // circuit1???dummy proof???set
            pw.set_proof_with_pis_target(&pt1, &dummy_proof1);
            pw.set_verifier_data_target(&vc1, &dummy_data1.verifier_only);
            now.elapsed().as_millis()
        }
        CircuitType::Circuit1 => {
            let now = Instant::now();
            // circuit1: real, circuit0: dummy????????????????????????
            pw.set_bool_target(b, false);

            // circuit1???proof?????????
            let proof = circuit1.prove().unwrap();
            pw.set_proof_with_pis_target(&pt1, &proof);
            pw.set_verifier_data_target(&vc1, &data1.verifier_only);

            // circuit0???dummy proof???set
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
