use super::{domain::Rotation, hash_point, Proof, SRS};
use crate::arithmetic::{get_challenge_scalar, Challenge, Curve, CurveAffine, Field};
use crate::polycommit::Params;
use crate::transcript::Hasher;

impl<C: CurveAffine> Proof<C> {
    /// Returns
    pub fn verify<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        params: &Params<C>,
        srs: &SRS<C>,
    ) -> bool {
        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = HBase::init(C::Base::one());

        // Hash the prover's advice commitments into the transcript
        for commitment in &self.advice_commitments {
            hash_point(&mut transcript, commitment)
                .expect("proof cannot contain points at infinity");
        }

        // Sample x_0 challenge
        let x_0: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Sample x_1 challenge
        let x_1: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Check permutations
        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = Vec::with_capacity(params.n as usize);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..params.n {
                omega_powers.push(cur);
                cur *= &srs.domain.get_omega();
            }
        }

        for i in 0..params.n as usize {
            // z(\omega^{-1} X) (a(X) + \beta X + \gamma) (b(X) + \delta \beta X + \gamma) (c(X) + \delta^2 \beta X + \gamma)
            let left_perm_eval = self.permutation_product_inv_evals[i];

            // z(X) (a(X) + \beta s_a(X) + \gamma) (b(X) + \beta s_b(X) + \gamma) (c(X) + \beta s_c(X) + \gamma)
            let right_perm_eval = self.permutation_product_evals[i];

            if left_perm_eval != right_perm_eval {
                return false;
            }
        }

        // Sample x_2 challenge, which keeps the gates linearly independent.
        let x_2: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain a commitment to h(X) in the form of multiple pieces of degree n - 1
        for c in &self.h_commitments {
            hash_point(&mut transcript, c).expect("proof cannot contain points at infinity");
        }

        // Sample x_3 challenge, which is used to ensure the circuit is
        // satisfied with high probability.
        let x_3: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Hash together all the openings provided by the prover into a new
        // transcript on the scalar field.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());

        for eval in self
            .advice_evals
            .iter()
            .chain(self.fixed_evals.iter())
            .chain(self.h_evals.iter())
        {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        // Evaluate the circuit using the custom gates provided
        let mut h_eval = C::Scalar::zero();
        for poly in srs.meta.gates.iter() {
            h_eval *= &x_2;

            let evaluation: C::Scalar = poly.evaluate(
                &|index| self.fixed_evals[index],
                &|index| self.advice_evals[index],
                &|a, b| a + &b,
                &|a, b| a * &b,
                &|a, scalar| a * &scalar,
            );

            h_eval += &evaluation;
        }
        let xn = x_3.pow(&[params.n as u64, 0, 0, 0]);

        // Compute the expected h(x) value
        let mut expected_h_eval = C::Scalar::zero();
        let mut cur = C::Scalar::one();
        for eval in &self.h_evals {
            expected_h_eval += &(cur * eval);
            cur *= &xn;
        }

        if h_eval != (expected_h_eval * &(xn - &C::Scalar::one())) {
            return false;
        }

        // We are now convinced the circuit is satisfied so long as the
        // polynomial commitments open to the correct values.

        // Sample x_4 for compressing openings at the same points together
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compress the commitments and expected evaluations at x_3 together
        // using the challenge x_4
        let mut q_commitments: Vec<Option<C::Projective>> = vec![None; srs.meta.rotations.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); srs.meta.rotations.len()];
        {
            let mut accumulate = |point_index: usize, new_commitment, eval| {
                q_commitments[point_index] = q_commitments[point_index]
                    .map(|mut commitment| {
                        commitment *= x_4;
                        commitment += new_commitment;
                        commitment
                    })
                    .or_else(|| Some(new_commitment.to_projective()));
                q_evals[point_index] *= &x_4;
                q_evals[point_index] += &eval;
            };

            for (query_index, &(wire, ref at)) in srs.meta.advice_queries.iter().enumerate() {
                let point_index = (*srs.meta.rotations.get(at).unwrap()).0;
                accumulate(
                    point_index,
                    self.advice_commitments[wire.0],
                    self.advice_evals[query_index],
                );
            }

            for (query_index, &(wire, ref at)) in srs.meta.fixed_queries.iter().enumerate() {
                let point_index = (*srs.meta.rotations.get(at).unwrap()).0;
                accumulate(
                    point_index,
                    srs.fixed_commitments[wire.0],
                    self.fixed_evals[query_index],
                );
            }

            let current_index = (*srs.meta.rotations.get(&Rotation::default()).unwrap()).0;
            for (h_commitment, h_eval) in self.h_commitments.iter().zip(self.h_evals.iter()) {
                accumulate(current_index, *h_commitment, *h_eval);
            }
        }

        // Sample a challenge x_5 for keeping the multi-point quotient
        // polynomial terms linearly independent.
        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain the commitment to the multi-point quotient polynomial f(X).
        hash_point(&mut transcript, &self.f_commitment)
            .expect("proof cannot contain points at infinity");

        // Sample a challenge x_6 for checking that f(X) was committed to
        // correctly.
        let x_6: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        for eval in self.q_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        // We can compute the expected f_eval at x_6 using the q_evals provided
        // by the prover and from x_5
        let mut f_eval = C::Scalar::zero();
        for (&row, &point_index) in srs.meta.rotations.iter() {
            let mut eval = self.q_evals[point_index.0];

            let point = srs.domain.rotate_omega(x_3, row);
            eval = eval - &q_evals[point_index.0];
            eval = eval * &(x_6 - &point).invert().unwrap();

            f_eval *= &x_5;
            f_eval += &eval;
        }

        // Sample a challenge x_7 that we will use to collapse the openings of
        // the various remaining polynomials at x_6 together.
        let x_7: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compute the final commitment that has to be opened
        let mut f_commitment: C::Projective = self.f_commitment.to_projective();
        for (_, &point_index) in srs.meta.rotations.iter() {
            f_commitment *= x_7;
            f_commitment = f_commitment + &q_commitments[point_index.0].as_ref().unwrap();
            f_eval *= &x_7;
            f_eval += &self.q_evals[point_index.0];
        }

        // Verify the opening proof
        params.verify_proof(
            &self.opening,
            &mut transcript,
            x_6,
            &f_commitment.to_affine(),
            f_eval,
        )
    }
}
