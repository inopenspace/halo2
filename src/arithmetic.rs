//! This module provides common utilities, traits and structures for group,
//! field and polynomial arithmetic.

use crossbeam_utils::thread;

#[macro_use]
mod macros;
mod curves;
mod fields;

pub use curves::*;
pub use fields::*;

/// This represents an element of a group with basic operations that can be
/// performed. This allows an FFT implementation (for example) to operate
/// generically over either a field or elliptic curve group.
pub trait Group: Copy + Clone + Send + Sync + 'static {
    /// The group is assumed to be of prime order $p$. `Scalar` is the
    /// associated scalar field of size $p$.
    type Scalar: Field;

    /// Returns the additive identity of the group.
    fn group_zero() -> Self;

    /// Adds `rhs` to this group element.
    fn group_add(&mut self, rhs: &Self);

    /// Subtracts `rhs` from this group element.
    fn group_sub(&mut self, rhs: &Self);

    /// Scales this group element by a scalar.
    fn group_scale(&mut self, by: &Self::Scalar);
}

/// This is a 128-bit verifier challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge(pub(crate) u128);

/// This algorithm applies the mapping of Algorithm 1 from the
/// [Halo](https://eprint.iacr.org/2019/1021) paper.
pub fn get_challenge_scalar<F: Field>(challenge: Challenge) -> F {
    let mut acc = (F::ZETA + F::one()).double();

    for i in (0..64).rev() {
        let should_negate = ((challenge.0 >> ((i << 1) + 1)) & 1) == 1;
        let should_endo = ((challenge.0 >> (i << 1)) & 1) == 1;

        let q = if should_negate { -F::one() } else { F::one() };
        let q = if should_endo { q * F::ZETA } else { q };
        acc = acc + q + acc;
    }

    acc
}

fn multiexp_serial<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C], acc: &mut C::Projective) {
    let coeffs: Vec<[u8; 32]> = coeffs.iter().map(|a| a.to_bytes()).collect();

    let c = if bases.len() < 4 {
        1
    } else if bases.len() < 32 {
        3
    } else {
        (f64::from(bases.len() as u32)).ln().ceil() as usize
    };

    fn get_at(segment: usize, c: usize, bytes: &[u8; 32]) -> usize {
        let skip_bits = segment * c;
        let skip_bytes = skip_bits / 8;

        if skip_bytes >= 32 {
            return 0;
        }

        let mut v = [0; 8];
        for (v, o) in v.iter_mut().zip(bytes[skip_bytes..].iter()) {
            *v = *o;
        }

        let mut tmp = u64::from_le_bytes(v);
        tmp >>= skip_bits - (skip_bytes * 8);
        tmp = tmp % (1 << c);

        tmp as usize
    }

    let segments = (256 / c) + 1;

    for current_segment in (0..segments).rev() {
        for _ in 0..c {
            *acc = acc.double();
        }

        #[derive(Clone, Copy)]
        enum Bucket<C: CurveAffine> {
            None,
            Affine(C),
            Projective(C::Projective),
        }

        impl<C: CurveAffine> Bucket<C> {
            fn add_assign(&mut self, other: &C) {
                *self = match *self {
                    Bucket::None => Bucket::Affine(*other),
                    Bucket::Affine(a) => Bucket::Projective(a + *other),
                    Bucket::Projective(mut a) => {
                        a += *other;
                        Bucket::Projective(a)
                    }
                }
            }

            fn add(self, mut other: C::Projective) -> C::Projective {
                match self {
                    Bucket::None => other,
                    Bucket::Affine(a) => {
                        other += a;
                        other
                    }
                    Bucket::Projective(a) => other + &a,
                }
            }
        }

        let mut buckets: Vec<Bucket<C>> = vec![Bucket::None; (1 << c) - 1];

        for (coeff, base) in coeffs.iter().zip(bases.iter()) {
            let coeff = get_at(current_segment, c, coeff);
            if coeff != 0 {
                buckets[coeff - 1].add_assign(base);
            }
        }

        // Summation by parts
        // e.g. 3a + 2b + 1c = a +
        //                    (a) + b +
        //                    ((a) + b) + c
        let mut running_sum = C::Projective::zero();
        for exp in buckets.into_iter().rev() {
            running_sum = exp.add(running_sum);
            *acc = *acc + &running_sum;
        }
    }
}

/// Performs a multi-exponentiation operation.
///
/// This function will panic if coeffs and bases have a different length.
///
/// This will use multithreading if beneficial.
pub fn best_multiexp<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) -> C::Projective {
    assert_eq!(coeffs.len(), bases.len());

    let num_cpus = num_cpus::get();
    if coeffs.len() > num_cpus {
        let chunk = coeffs.len() / num_cpus;
        let num_chunks = coeffs.chunks(chunk).len();
        let mut results = vec![C::Projective::zero(); num_chunks];
        thread::scope(|scope| {
            let chunk = coeffs.len() / num_cpus;

            for ((coeffs, bases), acc) in coeffs
                .chunks(chunk)
                .zip(bases.chunks(chunk))
                .zip(results.iter_mut())
            {
                scope.spawn(move |_| {
                    multiexp_serial(coeffs, bases, acc);
                });
            }
        })
        .unwrap();
        results.iter().fold(C::Projective::zero(), |a, b| a + b)
    } else {
        let mut acc = C::Projective::zero();
        multiexp_serial(coeffs, bases, &mut acc);
        acc
    }
}

/// Performs a radix-$2$ Fast-Fourier Transformation (FFT) on a vector of size
/// $n = 2^k$, when provided `log_n` = $k$ and an element of multiplicative
/// order $n$ called `omega` ($\omega$). The result is that the vector `a`, when
/// interpreted as the coefficients of a polynomial of degree $n - 1$, is
/// transformed into the evaluations of this polynomial at each of the $n$
/// distinct powers of $\omega$. This transformation is invertible by providing
/// $\omega^{-1}$ in place of $\omega$ and dividing each resulting field element
/// by $n$.
///
/// This will use multithreading if beneficial.
pub fn best_fft<G: Group>(a: &mut [G], omega: G::Scalar, log_n: u32) {
    let cpus = num_cpus::get();
    let log_cpus = log2_floor(cpus);

    if log_n <= log_cpus {
        serial_fft(a, omega, log_n);
    } else {
        parallel_fft(a, omega, log_n, log_cpus);
    }
}

fn serial_fft<G: Group>(a: &mut [G], omega: G::Scalar, log_n: u32) {
    fn bitreverse(mut n: u32, l: u32) -> u32 {
        let mut r = 0;
        for _ in 0..l {
            r = (r << 1) | (n & 1);
            n >>= 1;
        }
        r
    }

    let n = a.len() as u32;
    assert_eq!(n, 1 << log_n);

    for k in 0..n {
        let rk = bitreverse(k, log_n);
        if k < rk {
            a.swap(rk as usize, k as usize);
        }
    }

    let mut m = 1;
    for _ in 0..log_n {
        let w_m = omega.pow(&[u64::from(n / (2 * m)), 0, 0, 0]);

        let mut k = 0;
        while k < n {
            let mut w = G::Scalar::one();
            for j in 0..m {
                let mut t = a[(k + j + m) as usize];
                t.group_scale(&w);
                a[(k + j + m) as usize] = a[(k + j) as usize];
                a[(k + j + m) as usize].group_sub(&t);
                a[(k + j) as usize].group_add(&t);
                w *= &w_m;
            }

            k += 2 * m;
        }

        m *= 2;
    }
}

fn parallel_fft<G: Group>(a: &mut [G], omega: G::Scalar, log_n: u32, log_cpus: u32) {
    assert!(log_n >= log_cpus);

    let num_cpus = 1 << log_cpus;
    let log_new_n = log_n - log_cpus;
    let mut tmp = vec![vec![G::group_zero(); 1 << log_new_n]; num_cpus];
    let new_omega = omega.pow(&[num_cpus as u64, 0, 0, 0]);

    thread::scope(|scope| {
        let a = &*a;

        for (j, tmp) in tmp.iter_mut().enumerate() {
            scope.spawn(move |_| {
                // Shuffle into a sub-FFT
                let omega_j = omega.pow(&[j as u64, 0, 0, 0]);
                let omega_step = omega.pow(&[(j as u64) << log_new_n, 0, 0, 0]);

                let mut elt = G::Scalar::one();

                for (i, tmp) in tmp.iter_mut().enumerate() {
                    for s in 0..num_cpus {
                        let idx = (i + (s << log_new_n)) % (1 << log_n);
                        let mut t = a[idx];
                        t.group_scale(&elt);
                        tmp.group_add(&t);
                        elt *= &omega_step;
                    }
                    elt *= &omega_j;
                }

                // Perform sub-FFT
                serial_fft(tmp, new_omega, log_new_n);
            });
        }
    })
    .unwrap();

    // Unshuffle
    let mask = (1 << log_cpus) - 1;
    for (idx, a) in a.iter_mut().enumerate() {
        *a = tmp[idx & mask][idx >> log_cpus];
    }
}

/// This evaluates a provided polynomial (in coefficient form) at `point`.
pub fn eval_polynomial<F: Field>(poly: &[F], point: F) -> F {
    // TODO: parallelize?
    let mut acc = F::zero();
    let mut cur = F::one();
    for coeff in poly {
        acc += &(cur * coeff);
        cur *= &point;
    }
    acc
}

/// This computes the inner product of two vectors `a` and `b`.
///
/// This function will panic if the two vectors are not the same size.
pub fn compute_inner_product<F: Field>(a: &[F], b: &[F]) -> F {
    // TODO: parallelize?
    assert_eq!(a.len(), b.len());

    let mut acc = F::zero();
    for (a, b) in a.iter().zip(b.iter()) {
        acc += (*a) * (*b);
    }

    acc
}

/// Divides polynomial `a` in `X` by `X - b` with
/// no remainder.
pub fn kate_division<'a, F: Field, I: IntoIterator<Item = &'a F>>(a: I, mut b: F) -> Vec<F>
where
    I::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    b = -b;
    let a = a.into_iter();

    let mut q = vec![F::zero(); a.len() - 1];

    let mut tmp = F::zero();
    for (q, r) in q.iter_mut().rev().zip(a.rev()) {
        let mut lead_coeff = *r;
        lead_coeff.sub_assign(&tmp);
        *q = lead_coeff;
        tmp = lead_coeff;
        tmp.mul_assign(&b);
    }

    q
}

/// This simple utility function will parallelize an operation that is to be
/// performed over a mutable slice.
pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Clone>(v: &mut [T], f: F) {
    let n = v.len();
    let num_cpus = num_cpus::get();
    let mut chunk = (n as usize) / num_cpus;
    if chunk < num_cpus {
        chunk = n as usize;
    }

    thread::scope(|scope| {
        for (chunk_num, v) in v.chunks_mut(chunk).enumerate() {
            let f = f.clone();
            scope.spawn(move |_| {
                let start = chunk_num * chunk;
                f(v, start);
            });
        }
    })
    .unwrap();
}

fn log2_floor(num: usize) -> u32 {
    assert!(num > 0);

    let mut pow = 0;

    while (1 << (pow + 1)) <= num {
        pow += 1;
    }

    pow
}
