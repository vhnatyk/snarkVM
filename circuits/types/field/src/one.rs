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

impl<E: Environment> One for Field<E> {
    type Boolean = Boolean<E>;

    fn one() -> Self {
        E::one().into()
    }

    fn is_one(&self) -> Self::Boolean {
        self.is_equal(&Field::one())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;

    #[test]
    fn test_one() {
        let one = <Circuit as Environment>::BaseField::one();

        Circuit::scope("One", || {
            assert_scope!(0, 0, 0, 0);
            let candidate = Field::<Circuit>::one();
            assert_eq!(one, candidate.eject_value());
            assert_scope!(0, 0, 0, 0);
        });
    }

    #[test]
    fn test_is_one() {
        let candidate = Field::<Circuit>::one();
        // Should equal 1.
        assert!(candidate.is_one().eject_value());
        // Should not equal 0.
        assert!(!candidate.is_zero().eject_value());
    }
}
