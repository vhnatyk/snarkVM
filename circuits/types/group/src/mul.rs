// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<E: Environment> Mul<Scalar<E>> for Group<E> {
    type Output = Group<E>;

    fn mul(self, other: Scalar<E>) -> Self::Output {
        self * &other
    }
}

impl<E: Environment> Mul<Scalar<E>> for &Group<E> {
    type Output = Group<E>;

    fn mul(self, other: Scalar<E>) -> Self::Output {
        self * &other
    }
}

impl<E: Environment> Mul<&Scalar<E>> for Group<E> {
    type Output = Group<E>;

    fn mul(self, other: &Scalar<E>) -> Self::Output {
        let mut output = self;
        output *= other;
        output
    }
}

impl<E: Environment> Mul<&Scalar<E>> for &Group<E> {
    type Output = Group<E>;

    fn mul(self, other: &Scalar<E>) -> Self::Output {
        (*self).clone() * other
    }
}

impl<E: Environment> Mul<Group<E>> for Scalar<E> {
    type Output = Group<E>;

    fn mul(self, other: Group<E>) -> Self::Output {
        other * &self
    }
}

impl<E: Environment> Mul<Group<E>> for &Scalar<E> {
    type Output = Group<E>;

    fn mul(self, other: Group<E>) -> Self::Output {
        &other * self
    }
}

impl<E: Environment> Mul<&Group<E>> for Scalar<E> {
    type Output = Group<E>;

    fn mul(self, other: &Group<E>) -> Self::Output {
        other * &self
    }
}

impl<E: Environment> Mul<&Group<E>> for &Scalar<E> {
    type Output = Group<E>;

    fn mul(self, other: &Group<E>) -> Self::Output {
        other * self
    }
}

impl<E: Environment> MulAssign<Scalar<E>> for Group<E> {
    fn mul_assign(&mut self, other: Scalar<E>) {
        *self *= &other;
    }
}

impl<E: Environment> MulAssign<&Scalar<E>> for Group<E> {
    fn mul_assign(&mut self, other: &Scalar<E>) {
        *self *= other.to_bits_be().as_slice();
    }
}

impl<E: Environment, const N: usize> Mul<[Boolean<E>; N]> for Group<E> {
    type Output = Group<E>;

    fn mul(self, other: [Boolean<E>; N]) -> Self::Output {
        self * &other[..]
    }
}

impl<E: Environment, const N: usize> Mul<[Boolean<E>; N]> for &Group<E> {
    type Output = Group<E>;

    fn mul(self, other: [Boolean<E>; N]) -> Self::Output {
        self * &other[..]
    }
}

impl<E: Environment> Mul<&[Boolean<E>]> for Group<E> {
    type Output = Group<E>;

    fn mul(self, other: &[Boolean<E>]) -> Self::Output {
        let mut output = self;
        output *= other;
        output
    }
}

impl<E: Environment> Mul<&[Boolean<E>]> for &Group<E> {
    type Output = Group<E>;

    fn mul(self, other: &[Boolean<E>]) -> Self::Output {
        (*self).clone() * other
    }
}

impl<E: Environment, const N: usize> Mul<Group<E>> for [Boolean<E>; N] {
    type Output = Group<E>;

    fn mul(self, other: Group<E>) -> Self::Output {
        other * &self[..]
    }
}

impl<E: Environment> Mul<Group<E>> for &[Boolean<E>] {
    type Output = Group<E>;

    fn mul(self, other: Group<E>) -> Self::Output {
        &other * self
    }
}

impl<E: Environment, const N: usize> Mul<&Group<E>> for [Boolean<E>; N] {
    type Output = Group<E>;

    fn mul(self, other: &Group<E>) -> Self::Output {
        other * &self[..]
    }
}

impl<E: Environment> Mul<&Group<E>> for &[Boolean<E>] {
    type Output = Group<E>;

    fn mul(self, other: &Group<E>) -> Self::Output {
        other * self
    }
}

impl<E: Environment, const N: usize> MulAssign<[Boolean<E>; N]> for Group<E> {
    fn mul_assign(&mut self, other: [Boolean<E>; N]) {
        *self *= &other[..];
    }
}

impl<E: Environment> MulAssign<&[Boolean<E>]> for Group<E> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn mul_assign(&mut self, other: &[Boolean<E>]) {
        let base = self.clone();

        let mut output = Group::zero();
        for bit in other.iter() {
            output = output.double();
            output = Ternary::ternary(bit, &(&base + &output), &output);
        }
        *self = output;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_curves::ProjectiveCurve;
    use snarkvm_utilities::{test_rng, UniformRand};

    const ITERATIONS: usize = 10;

    fn check_mul(
        name: &str,
        expected: &<Circuit as Environment>::Affine,
        a: &Group<Circuit>,
        b: &Scalar<Circuit>,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        Circuit::scope(name, || {
            let candidate = a * b;
            assert_eq!(*expected, candidate.eject_value(), "({} * {})", a.eject_value(), b.eject_value());
            assert_scope!(num_constants, num_public, num_private, num_constraints);
        });
        Circuit::reset();
    }

    fn check_mul_assign(
        name: &str,
        expected: &<Circuit as Environment>::Affine,
        a: &Group<Circuit>,
        b: &Scalar<Circuit>,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        Circuit::scope(name, || {
            let mut candidate = a.clone();
            candidate *= b;
            assert_eq!(*expected, candidate.eject_value(), "({} * {})", a.eject_value(), b.eject_value());
            assert_scope!(num_constants, num_public, num_private, num_constraints);
        });
        Circuit::reset();
    }

    #[test]
    fn test_constant_times_scalar_constant() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Constant, base);
            let b = Scalar::<Circuit>::new(Mode::Constant, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 1757, 0, 0, 0);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 1757, 0, 0, 0);
        }
    }

    #[test]
    fn test_constant_times_scalar_public() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Constant, base);
            let b = Scalar::<Circuit>::new(Mode::Public, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 757, 0, 2500, 2500);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 757, 0, 2500, 2500);
        }
    }

    #[test]
    fn test_constant_times_scalar_private() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Constant, base);
            let b = Scalar::<Circuit>::new(Mode::Private, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 757, 0, 2500, 2500);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 757, 0, 2500, 2500);
        }
    }

    #[allow(clippy::identity_op)]
    #[test]
    fn test_public_times_scalar_constant() {
        use snarkvm_utilities::BigInteger;

        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let (num_constant, num_private, num_constraints) = {
                const MODULUS_BITS: usize = 251;
                let num_nonzero_bits = scalar.to_repr().to_biguint().bits() as usize;
                let num_leading_zero_bits = MODULUS_BITS - num_nonzero_bits;

                let num_constant = 2
                    + (3 /* DOUBLE constant */ + 2/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (1 /* DOUBLE private */ + 2/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits; // Typically around 760.
                let num_private = 2
                    + (0 /* DOUBLE constant */ + 3/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (5 /* DOUBLE private */ + 6/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits
                    - 10; // Typically around 2700.
                let num_constraints = 2
                    + (0 /* DOUBLE constant */ + 3/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (5 /* DOUBLE private */ + 6/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits
                    - 10; // Typically around 2700.

                (num_constant, num_private, num_constraints)
            };

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Public, base);
            let b = Scalar::<Circuit>::new(Mode::Constant, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, num_constant, 0, num_private, num_constraints);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, num_constant, 0, num_private, num_constraints);
        }
    }

    #[allow(clippy::identity_op)]
    #[test]
    fn test_private_times_scalar_constant() {
        use snarkvm_utilities::BigInteger;

        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let (num_constant, num_private, num_constraints) = {
                const MODULUS_BITS: usize = 251;
                let num_nonzero_bits = scalar.to_repr().to_biguint().bits() as usize;
                let num_leading_zero_bits = MODULUS_BITS - num_nonzero_bits;

                let num_constant = 2
                    + (3 /* DOUBLE constant */ + 2/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (1 /* DOUBLE private */ + 2/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits; // Typically around 760.
                let num_private = 2
                    + (0 /* DOUBLE constant */ + 3/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (5 /* DOUBLE private */ + 6/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits
                    - 10; // Typically around 2700.
                let num_constraints = 2
                    + (0 /* DOUBLE constant */ + 3/* public ADD constant */ + 0/* TERNARY */) * num_leading_zero_bits
                    + (5 /* DOUBLE private */ + 6/* public ADD private */ + 0/* TERNARY */) * num_nonzero_bits
                    - 10; // Typically around 2700.

                (num_constant, num_private, num_constraints)
            };

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Private, base);
            let b = Scalar::<Circuit>::new(Mode::Constant, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, num_constant, 0, num_private, num_constraints);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, num_constant, 0, num_private, num_constraints);
        }
    }

    #[test]
    fn test_public_times_scalar_public() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Public, base);
            let b = Scalar::<Circuit>::new(Mode::Public, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 755, 0, 3255, 3255);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 755, 0, 3255, 3255);
        }
    }

    #[test]
    fn test_public_times_scalar_private() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Public, base);
            let b = Scalar::<Circuit>::new(Mode::Private, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 755, 0, 3255, 3255);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 755, 0, 3255, 3255);
        }
    }

    #[test]
    fn test_private_times_scalar_public() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Private, base);
            let b = Scalar::<Circuit>::new(Mode::Public, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 755, 0, 3255, 3255);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 755, 0, 3255, 3255);
        }
    }

    #[test]
    fn test_private_times_scalar_private() {
        for i in 0..ITERATIONS {
            let base: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
            let scalar: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());

            let expected = (base * scalar).into();
            let a = Group::<Circuit>::new(Mode::Private, base);
            let b = Scalar::<Circuit>::new(Mode::Private, scalar);

            let name = format!("Mul: a * b {}", i);
            check_mul(&name, &expected, &a, &b, 755, 0, 3255, 3255);
            let name = format!("MulAssign: a * b {}", i);
            check_mul_assign(&name, &expected, &a, &b, 755, 0, 3255, 3255);
        }
    }

    #[test]
    fn test_mul_matches() {
        // Sample two random elements.
        let a: <Circuit as Environment>::Affine = UniformRand::rand(&mut test_rng());
        let b: <Circuit as Environment>::ScalarField = UniformRand::rand(&mut test_rng());
        let expected = (a * b).to_affine();

        // Constant
        let base = Group::<Circuit>::new(Mode::Constant, a);
        let scalar = Scalar::<Circuit>::new(Mode::Constant, b);
        let candidate_a = base * scalar;
        assert_eq!(expected, candidate_a.eject_value());

        // Private
        let base = Group::<Circuit>::new(Mode::Private, a);
        let scalar = Scalar::<Circuit>::new(Mode::Private, b);
        let candidate_b = base * scalar;
        assert_eq!(expected, candidate_b.eject_value());
    }
}
