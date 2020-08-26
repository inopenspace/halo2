use super::{
    circuit::{AllocatedBit, RecursiveCircuit, Variable, VerificationCircuit},
    Error,
};
use crate::arithmetic::{CurveAffine, Field};
use crate::plonk::circuit::ConstraintSystem;
use crate::transcript::Hasher;

#[derive(Clone, Copy, Debug)]
pub struct AllocatedNum<F: Field> {
    value: Option<F>,
    var: Variable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coeff<F: Field> {
    Zero,
    One,
    NegativeOne,
    Full(F),
}

#[derive(Clone, Copy, Debug)]
pub enum Num<F: Field> {
    Constant(Coeff<F>),
    Allocated(Coeff<F>, AllocatedNum<F>),
}

#[derive(Clone, Debug)]
pub struct Combination<F: Field> {
    value: Option<F>,
    terms: Vec<Num<F>>,
}

impl<
        'a,
        E1: CurveAffine,
        E2: CurveAffine<Base = E1::Scalar>,
        Inner: RecursiveCircuit<E1::Scalar>,
    > VerificationCircuit<'a, E1, E2, Inner>
{
    fn compute_b<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        x: AllocatedNum<E1::Scalar>,
        challenges: &[AllocatedNum<E1::Scalar>],
        challenges_inv: &[AllocatedNum<E1::Scalar>],
    ) -> Result<Combination<E1::Scalar>, Error> {
        unimplemented!()
    }

    fn num_equal_unless_base_case<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        base_case: AllocatedBit,
        lhs: &Combination<E1::Scalar>,
        rhs: &Combination<E1::Scalar>,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn equal_unless_base_case<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        base_case: AllocatedBit,
        mut lhs: &[AllocatedBit],
        mut rhs: &[AllocatedBit],
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn obtain_scalar_from_bits<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        bits: &[AllocatedBit],
    ) -> Result<AllocatedNum<E1::Scalar>, Error> {
        unimplemented!()
    }

    fn witness_bits_from_fe<F: Field, CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        value: F,
    ) -> Result<Vec<AllocatedBit>, Error> {
        unimplemented!()
    }

    fn verify_inner_product<CS: ConstraintSystem<E1::Scalar>, H: Hasher<E1::Scalar>>(
        &self,
        mut cs: CS,
        base_case: &AllocatedBit,
        transcript: &mut H,
        commitments: &[E2],
        openings: &[E2],
        b: &[&[AllocatedBit]],
    ) -> Result<(E2, Vec<Vec<AllocatedBit>>), Error> {
        unimplemented!()
    }

    fn commit_point<CS: ConstraintSystem<E1::Scalar>, H: Hasher<E1::Scalar>>(
        &self,
        mut cs: CS,
        transcript: &mut H,
        point: &E2,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn get_challenge_scalar<CS: ConstraintSystem<E1::Scalar>>(
        &self,
        mut cs: CS,
        bits: &[AllocatedBit],
    ) -> Result<AllocatedNum<E1::Scalar>, Error> {
        unimplemented!()
    }

    fn get_challenge<CS: ConstraintSystem<E1::Scalar>, H: Hasher<E1::Scalar>>(
        &self,
        mut cs: CS,
        transcript: &mut H,
    ) -> Result<Vec<AllocatedBit>, Error> {
        unimplemented!()
    }
}
