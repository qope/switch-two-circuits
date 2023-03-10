use hashbrown::HashMap;

use anyhow::Result;
use plonky2::{
    field::extension::Extendable,
    gates::noop::NoopGate,
    hash::hash_types::RichField,
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
    recursion::dummy_circuit::{dummy_circuit, dummy_proof},
};

#[derive(PartialEq, Eq, Clone)]
pub enum CircuitType {
    Circuit0,
    Circuit1,
}

pub struct Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    pub CircuitType,
    pub CircuitData<F, C, D>,
    pub Vec<Target>,
);

pub struct DummyCircuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    pub CircuitData<F, C, D>,
    pub ProofWithPublicInputs<F, C, D>,
);

pub fn make_inner_circuit0<C, F, H, const D: usize>() -> Circuit<F, C, D>
where
    H: AlgebraicHasher<F>,
    C: GenericConfig<D, F = F>,
    F: RichField + Extendable<D>,
{
    let log_num = 10;
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    let mut x = builder.add_virtual_hash();
    let x_start = x;
    for _ in 0..1 << log_num {
        x = builder.hash_n_to_hash_no_pad::<H>(x.elements.to_vec());
    }
    builder.register_public_inputs(&x.elements);
    let data = builder.build::<C>();
    Circuit(CircuitType::Circuit0, data, x_start.elements.to_vec())
}

pub fn make_inner_circuit1<C, F, const D: usize>() -> Circuit<F, C, D>
where
    C: GenericConfig<D, F = F>,
    F: RichField + Extendable<D>,
{
    let log_num_gates = 2;
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    for _ in 0..1 << log_num_gates {
        builder.add_gate(NoopGate, vec![]);
    }
    let x = builder.add_virtual_target();
    let x3 = builder.exp_u64(x, 3);
    builder.register_public_input(x3);
    let data = builder.build::<C>();
    Circuit(CircuitType::Circuit1, data, vec![x, x3])
}

impl<C: GenericConfig<D, F = F>, F: RichField + Extendable<D>, const D: usize> Circuit<F, C, D> {
    pub fn prove(&self) -> Result<ProofWithPublicInputs<F, C, D>> {
        let Circuit(ct, d, t) = self;
        let mut pw = PartialWitness::<F>::new();
        match ct {
            CircuitType::Circuit0 => {
                pw.set_target(t[0], F::from_canonical_u32(1));
                pw.set_target(t[1], F::from_canonical_u32(2));
                pw.set_target(t[2], F::from_canonical_u32(3));
                pw.set_target(t[3], F::from_canonical_u32(4));
            }
            CircuitType::Circuit1 => {
                pw.set_target(t[0], F::from_canonical_u32(2));
                pw.set_target(t[1], F::from_canonical_u32(8));
            }
        }
        let proof = d.prove(pw);
        proof
    }

    pub fn prove_dummy(&self) -> DummyCircuit<F, C, D> {
        let Circuit(_, d, _) = self;
        let dummy_data = dummy_circuit(&d.common);
        let dummy_proof = dummy_proof(&dummy_data, HashMap::new()).unwrap();
        DummyCircuit(dummy_data, dummy_proof)
    }
}
