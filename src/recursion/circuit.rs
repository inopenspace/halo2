use super::{Accumulator, Deferred, Error, RecursiveProof};
use crate::arithmetic::{CurveAffine, Field};
use crate::plonk;
use crate::plonk::circuit::{Circuit, ConstraintSystem};
use crate::polycommit::Params;
use std::marker::PhantomData;

pub trait RecursiveCircuit<F: Field> {
    fn base_payload(&self) -> Vec<bool>;

    fn synthesize<E1: CurveAffine, E2: CurveAffine, CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
    ) -> Result<(), Error>;
}

pub(crate) struct VerificationCircuit<
    'a,
    C1: CurveAffine,
    C2: CurveAffine,
    CS: RecursiveCircuit<C1::Scalar>,
> {
    pub(crate) _marker: PhantomData<(C1, C2)>,
    pub(crate) params: &'a Params<C2>,
    pub(crate) inner_circuit: &'a CS,
    pub(crate) local_accumulator: Option<Accumulator<C1>>,
    pub(crate) remote_accumulator: Option<Accumulator<C2>>,
    pub(crate) remote_deferred: Option<Deferred<C2::Scalar>>,
    pub(crate) proof: Option<&'a RecursiveProof<C2, C1>>,
    pub(crate) base_case: Option<bool>,
}

impl<
        'a,
        E1: CurveAffine,
        E2: CurveAffine<Base = E1::Scalar>,
        Inner: RecursiveCircuit<E1::Scalar>,
    > Circuit<E1::Scalar> for VerificationCircuit<'a, E1, E2, Inner>
{
    fn synthesize(&self, cs: &mut impl ConstraintSystem<E1::Scalar>) -> Result<(), plonk::Error> {
        unimplemented!()
    }
}
