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

impl<E: Environment> ToLowerBits for Field<E> {
    type Boolean = Boolean<E>;

    ///
    /// Outputs the lower `k` bits of an `n`-bit field element in little-endian representation.
    /// Enforces that the upper `n - k` bits are zero.
    ///
    fn to_lower_bits_le(&self, k: usize) -> Vec<Self::Boolean> {
        // Ensure the size is within the allowed capacity.
        if k > E::BaseField::size_in_bits() {
            E::halt(format!(
                "Attempted to extract {k} bits from a {}-bit base field element",
                E::BaseField::size_in_bits()
            ))
        }

        // Construct a vector of `Boolean`s comprising the bits of the field value.
        let bits = witness!(|self| self.to_bits_le().into_iter().take(k).collect::<Vec<_>>());

        // Reconstruct the bits as a linear combination representing the original field value.
        let mut accumulator = Field::zero();
        let mut coefficient = Field::one();
        for bit in &bits {
            accumulator += Field::from_boolean(bit) * &coefficient;
            coefficient = coefficient.double();
        }

        // Ensure value * 1 == (2^k * b_k + ... + 2^0 * b_0)
        // and ensures that b_n, ..., b_{n-k} are all equal to zero.
        E::assert_eq(self, accumulator);

        bits
    }

    ///
    /// Outputs the lower `k` bits of an `n`-bit field element in big-endian representation.
    /// Enforces that the upper `n - k` bits are zero.
    ///
    fn to_lower_bits_be(&self, k: usize) -> Vec<Self::Boolean> {
        let mut bits_be = self.to_lower_bits_le(k);
        bits_be.reverse();
        bits_be
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;
    use snarkvm_utilities::{bytes_from_bits_le, test_rng, FromBytes, ToBytes, UniformRand};

    const ITERATIONS: usize = 100;

    #[rustfmt::skip]
    fn check_to_lower_k_bits_le<I: IntegerType + Unsigned + ToBytes>(
        mode: Mode,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        let size_in_bits = <Circuit as Environment>::BaseField::size_in_bits();
        let size_in_bytes = (size_in_bits + 7) / 8;

        for i in 0..ITERATIONS {
            // Sample a random unsigned integer.
            let value: I = UniformRand::rand(&mut test_rng());
            let expected = value.to_bytes_le().unwrap().to_bits_le();

            // Construct the unsigned integer as a field element.
            let candidate = {
                let mut field_bytes = bytes_from_bits_le(&expected);
                field_bytes.resize(size_in_bytes, 0u8); // Pad up to byte size.
                Field::<Circuit>::new(mode, FromBytes::from_bytes_le(&field_bytes).unwrap())
            };

            Circuit::scope(&format!("{} {}", mode, i), || {
                let candidate = candidate.to_lower_bits_le(I::BITS);
                assert_eq!(I::BITS, candidate.len());
                for (i, (expected_bit, candidate_bit)) in expected.iter().zip_eq(candidate.iter()).enumerate() {
                    assert_eq!(*expected_bit, candidate_bit.eject_value(), "LSB+{}", i);
                }
                assert_scope!(num_constants, num_public, num_private, num_constraints);
            });
        }
    }

    // 8 bits

    #[test]
    fn test_to_8_bits_constant() {
        check_to_lower_k_bits_le::<u8>(Mode::Constant, 8, 0, 0, 0);
    }

    #[test]
    fn test_to_8_bits_public() {
        check_to_lower_k_bits_le::<u8>(Mode::Public, 0, 0, 8, 9);
    }

    #[test]
    fn test_to_8_bits_private() {
        check_to_lower_k_bits_le::<u8>(Mode::Private, 0, 0, 8, 9);
    }

    // 16 bits

    #[test]
    fn test_to_16_bits_constant() {
        check_to_lower_k_bits_le::<u16>(Mode::Constant, 16, 0, 0, 0);
    }

    #[test]
    fn test_to_16_bits_public() {
        check_to_lower_k_bits_le::<u16>(Mode::Public, 0, 0, 16, 17);
    }

    #[test]
    fn test_to_16_bits_private() {
        check_to_lower_k_bits_le::<u16>(Mode::Private, 0, 0, 16, 17);
    }

    // 32 bits

    #[test]
    fn test_to_32_bits_constant() {
        check_to_lower_k_bits_le::<u32>(Mode::Constant, 32, 0, 0, 0);
    }

    #[test]
    fn test_to_32_bits_public() {
        check_to_lower_k_bits_le::<u32>(Mode::Public, 0, 0, 32, 33);
    }

    #[test]
    fn test_to_32_bits_private() {
        check_to_lower_k_bits_le::<u32>(Mode::Private, 0, 0, 32, 33);
    }

    // 64 bits

    #[test]
    fn test_to_64_bits_constant() {
        check_to_lower_k_bits_le::<u64>(Mode::Constant, 64, 0, 0, 0);
    }

    #[test]
    fn test_to_64_bits_public() {
        check_to_lower_k_bits_le::<u64>(Mode::Public, 0, 0, 64, 65);
    }

    #[test]
    fn test_to_64_bits_private() {
        check_to_lower_k_bits_le::<u64>(Mode::Private, 0, 0, 64, 65);
    }

    // 128 bits

    #[test]
    fn test_to_128_bits_constant() {
        check_to_lower_k_bits_le::<u128>(Mode::Constant, 128, 0, 0, 0);
    }

    #[test]
    fn test_to_128_bits_public() {
        check_to_lower_k_bits_le::<u128>(Mode::Public, 0, 0, 128, 129);
    }

    #[test]
    fn test_to_128_bits_private() {
        check_to_lower_k_bits_le::<u128>(Mode::Private, 0, 0, 128, 129);
    }
}
