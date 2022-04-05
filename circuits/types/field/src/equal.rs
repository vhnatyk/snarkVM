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

impl<E: Environment> Equal<Self> for Field<E> {
    type Boolean = Boolean<E>;

    ///
    /// Returns `true` if `self` and `other` are equal.
    ///
    /// This method costs 3 constraints.
    ///
    fn is_equal(&self, other: &Self) -> Self::Boolean {
        !self.is_not_equal(other)
    }

    ///
    /// Returns `true` if `self` and `other` are *not* equal.
    ///
    /// This method constructs a boolean that indicates if
    /// `self` and `other ` are *not* equal to each other.
    ///
    /// This method costs 3 constraints.
    ///
    fn is_not_equal(&self, other: &Self) -> Self::Boolean {
        match (self.is_constant(), other.is_constant()) {
            (true, true) => witness!(|self, other| self != other),
            _ => {
                // Compute a boolean that is `true` if `this` and `that` are not equivalent.
                let is_neq: Boolean<E> = witness!(|self, other| self != other);

                // Assign the expected multiplier.
                let multiplier: Field<E> = witness!(|self, other| {
                    match self != other {
                        true => (self - other).inverse().expect("Failed to compute a native inverse"),
                        false => E::BaseField::one(),
                    }
                });

                //
                // Inequality Enforcement
                // ----------------------------------------------------------------
                // Check 1:  (a - b) * multiplier = is_neq
                // Check 2:  (a - b) * not(is_neq) = 0
                //
                //
                // Case 1: a == b AND is_neq == 0 (honest)
                // ----------------------------------------------------------------
                // Check 1:  (a - b) * 1 = 0
                //                 a - b = 0
                // => As a == b, is_neq is correct.
                //
                // Check 2:  (a - b) * not(0) = 0
                //                      a - b = 0
                // => As a == b, is_neq is correct.
                //
                // Remark: While the multiplier = 1 here, letting multiplier := n,
                //         for n as any field element, also holds.
                //
                //
                // Case 2: a == b AND is_neq == 1 (dishonest)
                // ----------------------------------------------------------------
                // Check 1:  (a - b) * 1 = 1
                //                 a - b = 1
                // => As a == b, the is_neq is incorrect.
                //
                // Remark: While the multiplier = 1 here, letting multiplier := n,
                //         for n as any field element, also holds.
                //
                //
                // Case 3a: a != b AND is_neq == 0 AND multiplier = 0 (dishonest)
                // ----------------------------------------------------------------
                // Check 2:  (a - b) * not(0) = 0
                //                      a - b = 0
                // => As a != b, is_neq is incorrect.
                //
                // Case 3b: a != b AND is_neq == 0 AND multiplier = 1 (dishonest)
                // ----------------------------------------------------------------
                // Check 1:  (a - b) * 1 = 0
                //                 a - b = 0
                // => As a != b, is_neq is incorrect.
                //
                // Remark: While the multiplier = 1 here, letting multiplier = n,
                //         for n as any field element (n != 0), also holds.
                //
                //
                // Case 4a: a != b AND is_neq == 1 AND multiplier = n [!= (a - b)^(-1)] (dishonest)
                // ---------------------------------------------------------------------------------
                // Check 1:  (a - b) * n = 1
                // => As n != (a - b)^(-1), is_neq is incorrect.
                //
                // Case 4b: a != b AND is_neq == 1 AND multiplier = (a - b)^(-1) (honest)
                // ---------------------------------------------------------------------------------
                // Check 1:  (a - b) * (a - b)^(-1) = 1
                //                                1 = 1
                // => is_neq is trivially correct.
                //
                // Check 2:  (a - b) * not(1) = 0
                //                          0 = 0
                // => is_neq is trivially correct.
                //

                // Compute `self` - `other`.
                let delta = self - other;

                // Negate `is_neq`.
                let is_eq = !is_neq.clone();

                // Check 1: (a - b) * multiplier = is_neq
                E::enforce(|| (&delta, &multiplier, &is_neq));

                // Check 2: (a - b) * not(is_neq) = 0
                E::enforce(|| (delta, is_eq, E::zero()));

                is_neq
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuits_environment::Circuit;

    const ITERATIONS: usize = 200;

    #[test]
    fn test_is_equal() {
        let zero = <Circuit as Environment>::BaseField::zero();
        let one = <Circuit as Environment>::BaseField::one();

        // Basic `true` and `false` cases
        {
            let mut accumulator = one + one;

            for _ in 0..ITERATIONS {
                let a = Field::<Circuit>::new(Mode::Private, accumulator);
                let b = Field::<Circuit>::new(Mode::Private, accumulator);
                let is_eq = a.is_equal(&b);
                assert!(is_eq.eject_value()); // true

                let a = Field::<Circuit>::new(Mode::Private, one);
                let b = Field::<Circuit>::new(Mode::Private, accumulator);
                let is_eq = a.is_equal(&b);
                assert!(!is_eq.eject_value()); // false

                let a = Field::<Circuit>::new(Mode::Private, accumulator);
                let b = Field::<Circuit>::new(Mode::Private, accumulator - one);
                let is_eq = a.is_equal(&b);
                assert!(!is_eq.eject_value()); // false

                accumulator += one;
            }
        }

        // Constant == Constant
        Circuit::scope("Constant == Constant", || {
            let mut accumulator = zero;

            for i in 0..ITERATIONS {
                let a = Field::<Circuit>::new(Mode::Constant, accumulator);
                let b = Field::<Circuit>::new(Mode::Constant, accumulator);

                let is_eq = a.is_equal(&b);
                assert!(is_eq.eject_value());
                assert_scope!((i + 1) * 3, 0, 0, 0);

                accumulator += one;
            }
        });

        // Public == Public
        Circuit::scope("Public == Public", || {
            let mut accumulator = zero;

            for i in 0..ITERATIONS {
                let a = Field::<Circuit>::new(Mode::Public, accumulator);
                let b = Field::<Circuit>::new(Mode::Public, accumulator);
                let is_eq = a.is_equal(&b);
                assert!(is_eq.eject_value());
                assert_scope!(0, (i + 1) * 2, (i + 1) * 2, (i + 1) * 3);

                accumulator += one;
            }
        });

        // Public == Private
        Circuit::scope("Public == Private", || {
            let mut accumulator = zero;

            for i in 0..ITERATIONS {
                let a = Field::<Circuit>::new(Mode::Public, accumulator);
                let b = Field::<Circuit>::new(Mode::Private, accumulator);
                let is_eq = a.is_equal(&b);
                assert!(is_eq.eject_value());
                assert_scope!(0, i + 1, (i + 1) * 3, (i + 1) * 3);

                accumulator += one;
            }
        });

        // Private == Private
        Circuit::scope("Private == Private", || {
            let mut accumulator = zero;

            for i in 0..ITERATIONS {
                let a = Field::<Circuit>::new(Mode::Private, accumulator);
                let b = Field::<Circuit>::new(Mode::Private, accumulator);
                let is_eq = a.is_equal(&b);
                assert!(is_eq.eject_value());
                assert!(Circuit::is_satisfied());
                assert_scope!(0, 0, (i + 1) * 4, (i + 1) * 3);

                accumulator += one;
            }
        });
    }

    #[test]
    fn test_is_neq_cases() {
        let zero = <Circuit as Environment>::BaseField::zero();
        let one = <Circuit as Environment>::BaseField::one();
        let two = one + one;
        let five = two + two + one;

        // Inequality Enforcement
        // ----------------------------------------------------------------
        // Check 1:  (a - b) * multiplier = is_neq
        // Check 2:  (a - b) * not(is_neq) = 0

        let enforce = |a: Field<Circuit>, b: Field<Circuit>, multiplier: Field<Circuit>, is_neq: Boolean<Circuit>| {
            // Compute `self` - `other`.
            let delta = &a - &b;

            // Negate `is_neq`.
            let is_eq = !is_neq.clone();

            // Check 1: (a - b) * multiplier = is_neq
            Circuit::enforce(|| (delta.clone(), multiplier, is_neq.clone()));

            // Check 2: (a - b) * not(is_neq) = 0
            Circuit::enforce(|| (delta, is_eq, Circuit::zero()));
        };

        //
        // Case 1: a == b AND is_neq == 0 (honest)
        // ----------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, five);
        let multiplier = Field::<Circuit>::new(Mode::Private, one);
        let is_neq = Boolean::new(Mode::Private, false);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(Circuit::is_satisfied());
        Circuit::reset();

        //
        // Case 2: a == b AND is_neq == 1 (dishonest)
        // ----------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, five);
        let multiplier = Field::<Circuit>::new(Mode::Private, one);
        let is_neq = Boolean::new(Mode::Private, true);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(!Circuit::is_satisfied());
        Circuit::reset();

        // Case 3a: a != b AND is_neq == 0 AND multiplier = 0 (dishonest)
        // ----------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, two);
        let multiplier = Field::<Circuit>::new(Mode::Private, zero);
        let is_neq = Boolean::new(Mode::Private, false);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(!Circuit::is_satisfied());
        Circuit::reset();

        //
        // Case 3b: a != b AND is_neq == 0 AND multiplier = 1 (dishonest)
        // ----------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, two);
        let multiplier = Field::<Circuit>::new(Mode::Private, one);
        let is_neq = Boolean::new(Mode::Private, false);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(!Circuit::is_satisfied());
        Circuit::reset();

        //
        // Case 4a: a != b AND is_neq == 1 AND multiplier = n [!= (a - b)^(-1)] (dishonest)
        // ---------------------------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, two);
        let multiplier = Field::<Circuit>::new(Mode::Private, two);
        let is_neq = Boolean::new(Mode::Private, true);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(!Circuit::is_satisfied());
        Circuit::reset();

        //
        // Case 4b: a != b AND is_neq == 1 AND multiplier = (a - b)^(-1) (honest)
        // ---------------------------------------------------------------------------------

        let a = Field::<Circuit>::new(Mode::Private, five);
        let b = Field::<Circuit>::new(Mode::Private, two);
        let multiplier =
            Field::<Circuit>::new(Mode::Private, (five - two).inverse().expect("Failed to compute a native inverse"));
        let is_neq = Boolean::new(Mode::Private, true);

        assert!(Circuit::is_satisfied());
        enforce(a, b, multiplier, is_neq);
        assert!(Circuit::is_satisfied());
        Circuit::reset();
    }
}
