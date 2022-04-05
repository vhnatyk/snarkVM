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

impl<E: Environment, I: IntegerType> Mul<Integer<E, I>> for Integer<E, I> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        self * &other
    }
}

impl<E: Environment, I: IntegerType> Mul<Integer<E, I>> for &Integer<E, I> {
    type Output = Integer<E, I>;

    fn mul(self, other: Integer<E, I>) -> Self::Output {
        self * &other
    }
}

impl<E: Environment, I: IntegerType> Mul<&Integer<E, I>> for Integer<E, I> {
    type Output = Self;

    fn mul(self, other: &Self) -> Self::Output {
        &self * other
    }
}

impl<E: Environment, I: IntegerType> Mul<&Integer<E, I>> for &Integer<E, I> {
    type Output = Integer<E, I>;

    fn mul(self, other: &Integer<E, I>) -> Self::Output {
        let mut output = self.clone();
        output *= other;
        output
    }
}

impl<E: Environment, I: IntegerType> MulAssign<Integer<E, I>> for Integer<E, I> {
    fn mul_assign(&mut self, other: Integer<E, I>) {
        *self *= &other;
    }
}

impl<E: Environment, I: IntegerType> MulAssign<&Integer<E, I>> for Integer<E, I> {
    fn mul_assign(&mut self, other: &Integer<E, I>) {
        // Stores the product of `self` and `other` in `self`.
        *self = self.mul_checked(other);
    }
}

impl<E: Environment, I: IntegerType> MulChecked<Self> for Integer<E, I> {
    type Output = Self;

    #[inline]
    fn mul_checked(&self, other: &Integer<E, I>) -> Self::Output {
        // Determine the variable mode.
        if self.is_constant() && other.is_constant() {
            // Compute the product and return the new constant.
            match self.eject_value().checked_mul(&other.eject_value()) {
                Some(value) => Integer::new(Mode::Constant, value),
                None => E::halt("Integer overflow on multiplication of two constants"),
            }
        } else if I::is_signed() {
            // Multiply the absolute value of `self` and `other` in the base field.
            // Note that it is safe to use abs_wrapped since we want I::MIN to be interpreted as an unsigned number.
            let (product, carry) = Self::mul_with_carry(&self.abs_wrapped(), &other.abs_wrapped(), true);

            // We need to check that the abs(a) * abs(b) did not exceed the unsigned maximum.
            let carry_bits_nonzero = carry.iter().fold(Boolean::constant(false), |a, b| a | b);

            // If the product should be positive, then it cannot exceed the signed maximum.
            let operands_same_sign = &self.msb().is_equal(other.msb());
            let positive_product_overflows = operands_same_sign & product.msb();

            // If the product should be negative, then it cannot exceed the absolute value of the signed minimum.
            let negative_product_underflows = {
                let lower_product_bits_nonzero =
                    product.bits_le[..(I::BITS - 1)].iter().fold(Boolean::constant(false), |a, b| a | b);
                let negative_product_lt_or_eq_signed_min =
                    !product.msb() | (product.msb() & !lower_product_bits_nonzero);
                !operands_same_sign & !negative_product_lt_or_eq_signed_min
            };

            // Ensure there are no overflows.
            let overflow = carry_bits_nonzero | positive_product_overflows | negative_product_underflows;
            E::assert_eq(overflow, E::zero());

            // Return the product of `self` and `other` with the appropriate sign.
            Self::ternary(operands_same_sign, &product, &Self::zero().sub_wrapped(&product))
        } else {
            // Compute the product of `self` and `other`.
            let (product, carry) = Self::mul_with_carry(self, other, true);

            // For unsigned multiplication, check that none of the carry bits are set.
            let overflow = carry.iter().fold(Boolean::constant(false), |a, b| a | b);
            E::assert_eq(overflow, E::zero());

            // Return the product of `self` and `other`.
            product
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_utilities::{test_rng, UniformRand};
    use test_utilities::*;

    use std::{ops::RangeInclusive, panic::RefUnwindSafe};

    const ITERATIONS: usize = 32;

    #[rustfmt::skip]
    fn check_mul<I: IntegerType + std::panic::RefUnwindSafe>(
        name: &str,
        first: I,
        second: I,
        mode_a: Mode,
        mode_b: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        let a = Integer::<Circuit, I>::new(mode_a, first);
        let b = Integer::<Circuit, I>::new(mode_b, second);
        let case = format!("({} * {})", a.eject_value(), b.eject_value());
        match first.checked_mul(&second) {
            Some(value) => {
                check_operation_passes(name, &case, value, &a, &b, Integer::mul_checked, num_constants, num_public, num_private, num_constraints);
                // Commute the operation.
                let a = Integer::<Circuit, I>::new(mode_a, second);
                let b = Integer::<Circuit, I>::new(mode_b, first);
                check_operation_passes(name, &case, value, &a, &b, Integer::mul_checked, num_constants, num_public, num_private, num_constraints);
            },
            None => {
                match (mode_a, mode_b) {
                    (Mode::Constant, Mode::Constant) => {
                        check_operation_halts(&a, &b, Integer::mul_checked);
                        // Commute the operation.
                        let a = Integer::<Circuit, I>::new(mode_a, second);
                        let b = Integer::<Circuit, I>::new(mode_b, first);
                        check_operation_halts(&a, &b, Integer::mul_checked);
                    },
                    _ => {
                        check_operation_fails(name, &case, &a, &b, Integer::mul_checked, num_constants, num_public, num_private, num_constraints);
                        // Commute the operation.
                        let a = Integer::<Circuit, I>::new(mode_a, second);
                        let b = Integer::<Circuit, I>::new(mode_b, first);
                        check_operation_fails(name, &case, &a, &b, Integer::mul_checked, num_constants, num_public, num_private, num_constraints);
                    }
                }
            }
        }
    }

    #[rustfmt::skip]
    fn run_test<I: IntegerType + std::panic::RefUnwindSafe>(
        mode_a: Mode,
        mode_b: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        let check_mul = | name: &str, first: I, second: I | check_mul(name, first, second, mode_a, mode_b, num_constants, num_public, num_private, num_constraints);

        for i in 0..ITERATIONS {
            // TODO (@pranav) Uniform random sampling almost always produces arguments that result in an overflow.
            //  Is there a better method for sampling arguments?
            let first: I = UniformRand::rand(&mut test_rng());
            let second: I = UniformRand::rand(&mut test_rng());

            let name = format!("Mul: {} * {} {}", mode_a, mode_b, i);
            check_mul(&name, first, second);

            let name = format!("Double: {} * {} {}", mode_a, mode_b, i);
            check_mul(&name, first, I::one() + I::one());

            let name = format!("Square: {} * {} {}", mode_a, mode_b, i);
            check_mul(&name, first, first);
        }

        // Check specific cases common to signed and unsigned integers.
        check_mul("1 * MAX", I::one(), I::MAX);
        check_mul("MAX * 1", I::MAX, I::one());
        check_mul("1 * MIN",I::one(), I::MIN);
        check_mul("MIN * 1",I::MIN, I::one());
        check_mul("0 * MAX", I::zero(), I::MAX);
        check_mul( "MAX * 0", I::MAX, I::zero());
        check_mul( "0 * MIN", I::zero(), I::MIN);
        check_mul( "MIN * 0", I::MIN, I::zero());
        check_mul("1 * 1", I::one(), I::one());

        // Check common overflow cases.
        check_mul("MAX * 2", I::MAX, I::one() + I::one());
        check_mul("2 * MAX", I::one() + I::one(), I::MAX);

        // Check additional corner cases for signed integers.
        if I::is_signed() {
            check_mul("MAX * -1", I::MAX, I::zero() - I::one());
            check_mul("-1 * MAX", I::zero() - I::one(), I::MAX);

            check_mul("MIN * -1", I::MIN, I::zero() - I::one());
            check_mul("-1 * MIN", I::zero() - I::one(), I::MIN);
            check_mul("MIN * -2", I::MIN, I::zero() - I::one() - I::one());
            check_mul("-2 * MIN", I::zero() - I::one() - I::one(), I::MIN);
        }
    }

    #[rustfmt::skip]
    fn run_exhaustive_test<I: IntegerType + RefUnwindSafe>(
        mode_a: Mode,
        mode_b: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) where
        RangeInclusive<I>: Iterator<Item = I>,
    {
        for first in I::MIN..=I::MAX {
            for second in I::MIN..=I::MAX {
                let name = format!("Mul: ({} * {})", first, second);
                check_mul(&name, first, second, mode_a, mode_b, num_constants, num_public, num_private, num_constraints);
            }
        }
    }

    #[test]
    fn test_u8_constant_times_constant() {
        type I = u8;
        run_test::<I>(Mode::Constant, Mode::Constant, 8, 0, 0, 0);
    }

    #[test]
    fn test_u8_constant_times_public() {
        type I = u8;
        run_test::<I>(Mode::Constant, Mode::Public, 0, 0, 23, 25);
    }

    #[test]
    fn test_u8_constant_times_private() {
        type I = u8;
        run_test::<I>(Mode::Constant, Mode::Private, 0, 0, 23, 25);
    }

    #[test]
    fn test_u8_public_times_constant() {
        type I = u8;
        run_test::<I>(Mode::Public, Mode::Constant, 0, 0, 23, 25);
    }

    #[test]
    fn test_u8_private_times_constant() {
        type I = u8;
        run_test::<I>(Mode::Private, Mode::Constant, 0, 0, 23, 25);
    }

    #[test]
    fn test_u8_public_times_public() {
        type I = u8;
        run_test::<I>(Mode::Public, Mode::Public, 0, 0, 24, 26);
    }

    #[test]
    fn test_u8_public_times_private() {
        type I = u8;
        run_test::<I>(Mode::Public, Mode::Private, 0, 0, 24, 26);
    }

    #[test]
    fn test_u8_private_times_public() {
        type I = u8;
        run_test::<I>(Mode::Private, Mode::Public, 0, 0, 24, 26);
    }

    #[test]
    fn test_u8_private_times_private() {
        type I = u8;
        run_test::<I>(Mode::Private, Mode::Private, 0, 0, 24, 26);
    }

    // Tests for i8

    #[test]
    fn test_i8_constant_times_constant() {
        type I = i8;
        run_test::<I>(Mode::Constant, Mode::Constant, 8, 0, 0, 0);
    }

    #[test]
    fn test_i8_constant_times_public() {
        type I = i8;
        run_test::<I>(Mode::Constant, Mode::Public, 32, 0, 69, 73);
    }

    #[test]
    fn test_i8_constant_times_private() {
        type I = i8;
        run_test::<I>(Mode::Constant, Mode::Private, 32, 0, 69, 73);
    }

    #[test]
    fn test_i8_public_times_constant() {
        type I = i8;
        run_test::<I>(Mode::Public, Mode::Constant, 32, 0, 69, 73);
    }

    #[test]
    fn test_i8_private_times_constant() {
        type I = i8;
        run_test::<I>(Mode::Private, Mode::Constant, 32, 0, 69, 73);
    }

    #[test]
    fn test_i8_public_times_public() {
        type I = i8;
        run_test::<I>(Mode::Public, Mode::Public, 24, 0, 88, 93);
    }

    #[test]
    fn test_i8_public_times_private() {
        type I = i8;
        run_test::<I>(Mode::Public, Mode::Private, 24, 0, 88, 93);
    }

    #[test]
    fn test_i8_private_times_public() {
        type I = i8;
        run_test::<I>(Mode::Private, Mode::Public, 24, 0, 88, 93);
    }

    #[test]
    fn test_i8_private_times_private() {
        type I = i8;
        run_test::<I>(Mode::Private, Mode::Private, 24, 0, 88, 93);
    }

    // Tests for u16

    #[test]
    fn test_u16_constant_times_constant() {
        type I = u16;
        run_test::<I>(Mode::Constant, Mode::Constant, 16, 0, 0, 0);
    }

    #[test]
    fn test_u16_constant_times_public() {
        type I = u16;
        run_test::<I>(Mode::Constant, Mode::Public, 0, 0, 47, 49);
    }

    #[test]
    fn test_u16_constant_times_private() {
        type I = u16;
        run_test::<I>(Mode::Constant, Mode::Private, 0, 0, 47, 49);
    }

    #[test]
    fn test_u16_public_times_constant() {
        type I = u16;
        run_test::<I>(Mode::Public, Mode::Constant, 0, 0, 47, 49);
    }

    #[test]
    fn test_u16_private_times_constant() {
        type I = u16;
        run_test::<I>(Mode::Private, Mode::Constant, 0, 0, 47, 49);
    }

    #[test]
    fn test_u16_public_times_public() {
        type I = u16;
        run_test::<I>(Mode::Public, Mode::Public, 0, 0, 48, 50);
    }

    #[test]
    fn test_u16_public_times_private() {
        type I = u16;
        run_test::<I>(Mode::Public, Mode::Private, 0, 0, 48, 50);
    }

    #[test]
    fn test_u16_private_times_public() {
        type I = u16;
        run_test::<I>(Mode::Private, Mode::Public, 0, 0, 48, 50);
    }

    #[test]
    fn test_u16_private_times_private() {
        type I = u16;
        run_test::<I>(Mode::Private, Mode::Private, 0, 0, 48, 50);
    }

    // Tests for i16

    #[test]
    fn test_i16_constant_times_constant() {
        type I = i16;
        run_test::<I>(Mode::Constant, Mode::Constant, 16, 0, 0, 0);
    }

    #[test]
    fn test_i16_constant_times_public() {
        type I = i16;
        run_test::<I>(Mode::Constant, Mode::Public, 64, 0, 133, 137);
    }

    #[test]
    fn test_i16_constant_times_private() {
        type I = i16;
        run_test::<I>(Mode::Constant, Mode::Private, 64, 0, 133, 137);
    }

    #[test]
    fn test_i16_public_times_constant() {
        type I = i16;
        run_test::<I>(Mode::Public, Mode::Constant, 64, 0, 133, 137);
    }

    #[test]
    fn test_i16_private_times_constant() {
        type I = i16;
        run_test::<I>(Mode::Private, Mode::Constant, 64, 0, 133, 137);
    }

    #[test]
    fn test_i16_public_times_public() {
        type I = i16;
        run_test::<I>(Mode::Public, Mode::Public, 48, 0, 168, 173);
    }

    #[test]
    fn test_i16_public_times_private() {
        type I = i16;
        run_test::<I>(Mode::Public, Mode::Private, 48, 0, 168, 173);
    }

    #[test]
    fn test_i16_private_times_public() {
        type I = i16;
        run_test::<I>(Mode::Private, Mode::Public, 48, 0, 168, 173);
    }

    #[test]
    fn test_i16_private_times_private() {
        type I = i16;
        run_test::<I>(Mode::Private, Mode::Private, 48, 0, 168, 173);
    }

    // Tests for u32

    #[test]
    fn test_u32_constant_times_constant() {
        type I = u32;
        run_test::<I>(Mode::Constant, Mode::Constant, 32, 0, 0, 0);
    }

    #[test]
    fn test_u32_constant_times_public() {
        type I = u32;
        run_test::<I>(Mode::Constant, Mode::Public, 0, 0, 95, 97);
    }

    #[test]
    fn test_u32_constant_times_private() {
        type I = u32;
        run_test::<I>(Mode::Constant, Mode::Private, 0, 0, 95, 97);
    }

    #[test]
    fn test_u32_public_times_constant() {
        type I = u32;
        run_test::<I>(Mode::Public, Mode::Constant, 0, 0, 95, 97);
    }

    #[test]
    fn test_u32_private_times_constant() {
        type I = u32;
        run_test::<I>(Mode::Private, Mode::Constant, 0, 0, 95, 97);
    }

    #[test]
    fn test_u32_public_times_public() {
        type I = u32;
        run_test::<I>(Mode::Public, Mode::Public, 0, 0, 96, 98);
    }

    #[test]
    fn test_u32_public_times_private() {
        type I = u32;
        run_test::<I>(Mode::Public, Mode::Private, 0, 0, 96, 98);
    }

    #[test]
    fn test_u32_private_times_public() {
        type I = u32;
        run_test::<I>(Mode::Private, Mode::Public, 0, 0, 96, 98);
    }

    #[test]
    fn test_u32_private_times_private() {
        type I = u32;
        run_test::<I>(Mode::Private, Mode::Private, 0, 0, 96, 98);
    }

    // Tests for i32

    #[test]
    fn test_i32_constant_times_constant() {
        type I = i32;
        run_test::<I>(Mode::Constant, Mode::Constant, 32, 0, 0, 0);
    }

    #[test]
    fn test_i32_constant_times_public() {
        type I = i32;
        run_test::<I>(Mode::Constant, Mode::Public, 128, 0, 261, 265);
    }

    #[test]
    fn test_i32_constant_times_private() {
        type I = i32;
        run_test::<I>(Mode::Constant, Mode::Private, 128, 0, 261, 265)
    }

    #[test]
    fn test_i32_public_times_constant() {
        type I = i32;
        run_test::<I>(Mode::Public, Mode::Constant, 128, 0, 261, 265);
    }

    #[test]
    fn test_i32_private_times_constant() {
        type I = i32;
        run_test::<I>(Mode::Private, Mode::Constant, 128, 0, 261, 265);
    }

    #[test]
    fn test_i32_public_times_public() {
        type I = i32;
        run_test::<I>(Mode::Public, Mode::Public, 96, 0, 328, 333);
    }

    #[test]
    fn test_i32_public_times_private() {
        type I = i32;
        run_test::<I>(Mode::Public, Mode::Private, 96, 0, 328, 333);
    }

    #[test]
    fn test_i32_private_times_public() {
        type I = i32;
        run_test::<I>(Mode::Private, Mode::Public, 96, 0, 328, 333);
    }

    #[test]
    fn test_i32_private_times_private() {
        type I = i32;
        run_test::<I>(Mode::Private, Mode::Private, 96, 0, 328, 333);
    }

    // Tests for u64

    #[test]
    fn test_u64_constant_times_constant() {
        type I = u64;
        run_test::<I>(Mode::Constant, Mode::Constant, 64, 0, 0, 0);
    }

    #[test]
    fn test_u64_constant_times_public() {
        type I = u64;
        run_test::<I>(Mode::Constant, Mode::Public, 0, 0, 191, 193);
    }

    #[test]
    fn test_u64_constant_times_private() {
        type I = u64;
        run_test::<I>(Mode::Constant, Mode::Private, 0, 0, 191, 193);
    }

    #[test]
    fn test_u64_public_times_constant() {
        type I = u64;
        run_test::<I>(Mode::Public, Mode::Constant, 0, 0, 191, 193);
    }

    #[test]
    fn test_u64_private_times_constant() {
        type I = u64;
        run_test::<I>(Mode::Private, Mode::Constant, 0, 0, 191, 193);
    }

    #[test]
    fn test_u64_public_times_public() {
        type I = u64;
        run_test::<I>(Mode::Public, Mode::Public, 0, 0, 192, 194);
    }

    #[test]
    fn test_u64_public_times_private() {
        type I = u64;
        run_test::<I>(Mode::Public, Mode::Private, 0, 0, 192, 194);
    }

    #[test]
    fn test_u64_private_times_public() {
        type I = u64;
        run_test::<I>(Mode::Private, Mode::Public, 0, 0, 192, 194);
    }

    #[test]
    fn test_u64_private_times_private() {
        type I = u64;
        run_test::<I>(Mode::Private, Mode::Private, 0, 0, 192, 194);
    }

    // Tests for i64

    #[test]
    fn test_i64_constant_times_constant() {
        type I = i64;
        run_test::<I>(Mode::Constant, Mode::Constant, 64, 0, 0, 0);
    }

    #[test]
    fn test_i64_constant_times_public() {
        type I = i64;
        run_test::<I>(Mode::Constant, Mode::Public, 256, 0, 517, 521);
    }

    #[test]
    fn test_i64_constant_times_private() {
        type I = i64;
        run_test::<I>(Mode::Constant, Mode::Private, 256, 0, 517, 521);
    }

    #[test]
    fn test_i64_public_times_constant() {
        type I = i64;
        run_test::<I>(Mode::Public, Mode::Constant, 256, 0, 517, 521);
    }

    #[test]
    fn test_i64_private_times_constant() {
        type I = i64;
        run_test::<I>(Mode::Private, Mode::Constant, 256, 0, 517, 521);
    }

    #[test]
    fn test_i64_public_times_public() {
        type I = i64;
        run_test::<I>(Mode::Public, Mode::Public, 192, 0, 648, 653);
    }

    #[test]
    fn test_i64_public_times_private() {
        type I = i64;
        run_test::<I>(Mode::Public, Mode::Private, 192, 0, 648, 653);
    }

    #[test]
    fn test_i64_private_times_public() {
        type I = i64;
        run_test::<I>(Mode::Private, Mode::Public, 192, 0, 648, 653);
    }

    #[test]
    fn test_i64_private_times_private() {
        type I = i64;
        run_test::<I>(Mode::Private, Mode::Private, 192, 0, 648, 653);
    }

    // Tests for u128

    #[test]
    fn test_u128_constant_times_constant() {
        type I = u128;
        run_test::<I>(Mode::Constant, Mode::Constant, 128, 0, 0, 0);
    }

    #[test]
    fn test_u128_constant_times_public() {
        type I = u128;
        run_test::<I>(Mode::Constant, Mode::Public, 0, 0, 513, 516);
    }

    #[test]
    fn test_u128_constant_times_private() {
        type I = u128;
        run_test::<I>(Mode::Constant, Mode::Private, 0, 0, 513, 516);
    }

    #[test]
    fn test_u128_public_times_constant() {
        type I = u128;
        run_test::<I>(Mode::Public, Mode::Constant, 0, 0, 513, 516);
    }

    #[test]
    fn test_u128_private_times_constant() {
        type I = u128;
        run_test::<I>(Mode::Private, Mode::Constant, 0, 0, 513, 516);
    }

    #[test]
    fn test_u128_public_times_public() {
        type I = u128;
        run_test::<I>(Mode::Public, Mode::Public, 0, 0, 517, 520);
    }

    #[test]
    fn test_u128_public_times_private() {
        type I = u128;
        run_test::<I>(Mode::Public, Mode::Private, 0, 0, 517, 520);
    }

    #[test]
    fn test_u128_private_times_public() {
        type I = u128;
        run_test::<I>(Mode::Private, Mode::Public, 0, 0, 517, 520);
    }

    #[test]
    fn test_u128_private_times_private() {
        type I = u128;
        run_test::<I>(Mode::Private, Mode::Private, 0, 0, 517, 520);
    }

    // Tests for i128

    #[test]
    fn test_i128_constant_times_constant() {
        type I = i128;
        run_test::<I>(Mode::Constant, Mode::Constant, 128, 0, 0, 0);
    }

    #[test]
    fn test_i128_constant_times_public() {
        type I = i128;
        run_test::<I>(Mode::Constant, Mode::Public, 512, 0, 1159, 1164);
    }

    #[test]
    fn test_i128_constant_times_private() {
        type I = i128;
        run_test::<I>(Mode::Constant, Mode::Private, 512, 0, 1159, 1164);
    }

    #[test]
    fn test_i128_public_times_constant() {
        type I = i128;
        run_test::<I>(Mode::Public, Mode::Constant, 512, 0, 1159, 1164);
    }

    #[test]
    fn test_i128_private_times_constant() {
        type I = i128;
        run_test::<I>(Mode::Private, Mode::Constant, 512, 0, 1159, 1164);
    }

    #[test]
    fn test_i128_public_times_public() {
        type I = i128;
        run_test::<I>(Mode::Public, Mode::Public, 384, 0, 1421, 1427);
    }

    #[test]
    fn test_i128_public_times_private() {
        type I = i128;
        run_test::<I>(Mode::Public, Mode::Private, 384, 0, 1421, 1427);
    }

    #[test]
    fn test_i128_private_times_public() {
        type I = i128;
        run_test::<I>(Mode::Private, Mode::Public, 384, 0, 1421, 1427);
    }

    #[test]
    fn test_i128_private_times_private() {
        type I = i128;
        run_test::<I>(Mode::Private, Mode::Private, 384, 0, 1421, 1427);
    }

    // Exhaustive tests for u8.

    #[test]
    #[ignore]
    fn test_exhaustive_u8_constant_times_constant() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Constant, 8, 0, 0, 0);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_constant_times_public() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Public, 0, 0, 23, 25);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_constant_times_private() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Private, 0, 0, 23, 25);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_public_times_constant() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Constant, 0, 0, 23, 25);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_private_times_constant() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Constant, 0, 0, 23, 25);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_public_times_public() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Public, 0, 0, 24, 26);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_public_times_private() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Private, 0, 0, 24, 26);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_private_times_public() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Public, 0, 0, 24, 26);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_u8_private_times_private() {
        type I = u8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Private, 0, 0, 24, 26);
    }

    // Tests for i8

    #[test]
    #[ignore]
    fn test_exhaustive_i8_constant_times_constant() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Constant, 8, 0, 0, 0);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_constant_times_public() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Public, 32, 0, 69, 73);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_constant_times_private() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Constant, Mode::Private, 32, 0, 69, 73);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_public_times_constant() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Constant, 32, 0, 69, 73);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_private_times_constant() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Constant, 32, 0, 69, 73);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_public_times_public() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Public, 24, 0, 88, 93);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_public_times_private() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Public, Mode::Private, 24, 0, 88, 93);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_private_times_public() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Public, 24, 0, 88, 93);
    }

    #[test]
    #[ignore]
    fn test_exhaustive_i8_private_times_private() {
        type I = i8;
        run_exhaustive_test::<I>(Mode::Private, Mode::Private, 24, 0, 88, 93);
    }
}
