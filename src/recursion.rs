/// This file deals with the `RecursiveProof` struct and its top-level methods
/// `create_proof()` and `verify()`.
/// It also knows about the `Accumulator` and `Deferred` structs which are
/// part of `RecursiveProof`.
use crate::arithmetic::{CurveAffine, Field};
use crate::plonk::{circuit::Circuit, prover::Proof, verifier::Deferred};
use crate::polycommit::{Accumulator, Params};
use std::marker::PhantomData;

#[macro_use]
pub mod circuit;
pub mod util;
pub mod verifier;

pub use circuit::*;

#[derive(Debug)]
pub enum Error {
    SynthesisError,
    IncompatibleParams,
    ConstraintSystemFailure,
}

pub struct RecursiveProof<E1: CurveAffine, E2: CurveAffine> {
    proof: Proof<E1>, // PLONK proof (which contains an inner product proof)
    local_accumulator: Accumulator<E1>, // accumulator lives in polycommit
    remote_accumulator: Accumulator<E2>,
    remote_deferred: Deferred<E2::Scalar>, // will change based on PLONK changes
}

impl<E1, E2> RecursiveProof<E1, E2>
where
    E1: CurveAffine<Base = <E2 as CurveAffine>::Scalar>,
    E2: CurveAffine<Base = <E1 as CurveAffine>::Scalar>,
{
    pub fn create_proof<CS: RecursiveCircuit<E1::Scalar> + RecursiveCircuit<E2::Scalar>>(
        e1params: &Params<E1>,
        e2params: &Params<E2>,
        old_proof: Option<&RecursiveProof<E2, E1>>,
        circuit: &CS,
    ) -> Result<Self, Error> {
        unimplemented!()
    }

    pub fn verify<CS: RecursiveCircuit<E1::Scalar> + RecursiveCircuit<E2::Scalar>>(
        &self,
        e1params: &Params<E1>,
        e2params: &Params<E2>,
        circuit: &CS,
    ) -> Result<bool, Error> {
        unimplemented!()
    }
}

impl<C: CurveAffine> Accumulator<C> {
    /// Creates a phony instance of metadata from a "previous"
    /// proof that never existed; used to bootstrap the cycle.
    pub fn dummy(params: &Params<C>) -> Self {
        unimplemented!()
    }

    pub fn verify<CS: Circuit<C::Scalar>>(
        &self,
        params: &Params<C>,
        circuit: &CS,
    ) -> Result<bool, Error> {
        unimplemented!()
    }
}

impl<F: Field> Deferred<F> {
    pub fn dummy(k: usize) -> Self {
        unimplemented!()
    }

    pub fn compute(&self, k: usize) -> (F, F) {
        unimplemented!()
    }

    pub fn verify(&self, k: usize) -> bool {
        unimplemented!()
    }
}
