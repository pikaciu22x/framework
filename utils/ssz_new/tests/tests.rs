use ethereum_types::U256;
use ssz_new::{SszDecode, SszEncode};
use ssz_new_derive::{SszDecode, SszEncode};

#[derive(SszEncode, SszDecode, PartialEq, Debug)]
struct Fixed {
    a: u16,
    b: bool,
}

#[derive(SszEncode, SszDecode, PartialEq, Debug)]
struct Variable {
    a: u16,
    b: Vec<u8>,
    c: bool,
}

#[derive(SszEncode, SszDecode, PartialEq, Debug)]
struct Nested {
    fixed: Fixed,
    variable: Variable,
}

#[derive(SszEncode, SszDecode, PartialEq, Debug)]
struct Skippable {
    stay_1: [u8; 4],

    #[ssz(skip_serializing)]
    #[ssz(skip_deserializing)]
    skip_1: u8,

    #[ssz(skip_serializing)]
    #[ssz(skip_deserializing)]
    skip_2: Vec<u8>,

    stay_2: Vec<u8>,
}

#[derive(SszEncode, SszDecode, PartialEq, Debug)]
struct NestedVariable {
    a: Vec<U256>,
    b: Vec<U256>,
}

mod serialize_derive {
    use crate::*;

    #[test]
    fn is_fixed_size() {
        assert!(!<Nested as SszEncode>::is_ssz_fixed_len());
        assert!(!<Variable as SszEncode>::is_ssz_fixed_len());
        assert!(<Fixed as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn serialize_fixed_struct() {
        let fixed = Fixed { a: 22, b: true };

        assert_eq!(fixed.as_ssz_bytes(), vec![22, 0, 1])
    }

    #[test]
    fn serialize_variable_struct() {
        let variable = Variable {
            a: u16::max_value(),
            b: vec![1, 2, 3, 4, 5],
            c: false,
        };

        assert_eq!(
            variable.as_ssz_bytes(),
            vec![
                u8::max_value(),
                u8::max_value(),
                7,
                0,
                0,
                0,
                0,
                1,
                2,
                3,
                4,
                5
            ]
        )
    }

    #[test]
    fn serialize_nested_struct() {
        let nested = Nested {
            fixed: Fixed { a: 5, b: false },
            variable: Variable {
                a: 80,
                b: vec![1, 2, 3, 4],
                c: true,
            },
        };

        assert_eq!(
            nested.as_ssz_bytes(),
            vec![5, 0, 0, 7, 0, 0, 0, 80, 0, 7, 0, 0, 0, 1, 1, 2, 3, 4]
        );
    }
}

mod deserialize_derive {
    use crate::*;

    #[test]
    fn deserialize_fixed_struct() {
        let fixed = Fixed { a: 22, b: true };

        assert_eq!(Fixed::from_ssz_bytes(&[22, 0, 1]).unwrap(), fixed);
    }

    #[test]
    fn deserialize_variable_struct() {
        let variable = Variable {
            a: u16::max_value(),
            b: vec![1, 2, 3, 4, 5],
            c: false,
        };

        assert_eq!(
            Variable::from_ssz_bytes(&[
                u8::max_value(),
                u8::max_value(),
                7,
                0,
                0,
                0,
                0,
                1,
                2,
                3,
                4,
                5
            ])
            .unwrap(),
            variable
        );
    }

    #[test]
    fn deserialize_nested_struct() {
        let nested = Nested {
            fixed: Fixed { a: 5, b: false },
            variable: Variable {
                a: 80,
                b: vec![1, 2, 3, 4],
                c: true,
            },
        };

        assert_eq!(
            Nested::from_ssz_bytes(&[5, 0, 0, 7, 0, 0, 0, 80, 0, 7, 0, 0, 0, 1, 1, 2, 3, 4])
                .unwrap(),
            nested
        );
    }

    #[test]
    fn skip_fields() {
        let skippable = Skippable {
            stay_1: [1, 2, 3, 4],
            stay_2: vec![1, 2, 3, 4, 5],
            skip_1: 42,
            skip_2: vec![6, 7, 8, 9, 10],
        };

        let serialized = skippable.as_ssz_bytes();
        assert_eq!(serialized, vec![1, 2, 3, 4, 8, 0, 0, 0, 1, 2, 3, 4, 5]);

        let skippable = Skippable::from_ssz_bytes(serialized.as_slice()).expect("Test");
        assert_eq!(skippable.skip_1, <u8>::default());
        assert_eq!(skippable.skip_2, <Vec<u8>>::default());
    }
}

mod round_trips {
    use crate::*;

    #[test]
    fn nested_variable() {
        let item = NestedVariable {
            a: vec![
                U256::from_dec_str("12345").expect("Test"),
                U256::from_dec_str("12345").expect("Test"),
                U256::from_dec_str("12345").expect("Test"),
                U256::from_dec_str("12345").expect("Test"),
            ],
            b: vec![U256::from_dec_str("12345").expect("Test")],
        };

        assert_round_trip(&item);
        assert_eq!(
            NestedVariable::ssz_fixed_len(),
            ssz_new::BYTES_PER_LENGTH_OFFSET
        );
    }

    fn assert_round_trip<T: SszEncode + SszDecode + PartialEq + std::fmt::Debug>(t: &T) {
        assert_eq!(&T::from_ssz_bytes(&t.as_ssz_bytes()).expect("Test"), t);
    }
}
