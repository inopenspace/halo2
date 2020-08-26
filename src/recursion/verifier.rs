use super::{
    circuit::{RecursiveCircuit, VerificationCircuit},
    Accumulator, AllocatedBit, Deferred, Error, RecursiveProof,
};
use crate::arithmetic::CurveAffine;
use crate::plonk::circuit::{Circuit, ConstraintSystem};
use crate::polycommit::Params;
use crate::transcript::Hasher;

use std::marker::PhantomData;

impl<E1, E2> RecursiveProof<E1, E2>
where
    E1: CurveAffine<Base = <E2 as CurveAffine>::Scalar>,
    E2: CurveAffine<Base = <E1 as CurveAffine>::Scalar>,
{
    /// verify_inner
    pub(crate) fn verify_inner<CS: RecursiveCircuit<E1::Scalar> + RecursiveCircuit<E2::Scalar>>(
        &self,
        e1params: &Params<E1>,
    ) -> Result<
        (
            bool,
            Deferred<E1::Scalar>,
            Accumulator<E1>,
            Accumulator<E2>,
            Vec<u8>,
        ),
        Error,
    > {
        unimplemented!()
    }
}

impl<
        'a,
        E1: CurveAffine,
        E2: CurveAffine<Base = E1::Scalar>,
        Inner: RecursiveCircuit<E1::Scalar>,
    > VerificationCircuit<'a, E1, E2, Inner>
{
    fn verify_deferred<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        mut deferred: Deferred<E1::Scalar>,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn verify_accumulator<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        base_case: AllocatedBit,
        k_commitment: &E2,
        old_accumulator: &Accumulator<E1>,
        new_deferred: &Deferred<E2::Scalar>,
        new_accumulator: &Accumulator<E2>,
    ) -> Result<(), Error> {
        unimplemented!()
    }
}
