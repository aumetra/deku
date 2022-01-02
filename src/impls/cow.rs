use crate::{DekuError, DekuRead, DekuWrite};
use bitvec::prelude::*;
use std::borrow::Cow;

impl<'a, T, Ctx> DekuRead<'a, Ctx> for Cow<'a, T>
where
    T: ToOwned + ?Sized,
    <T as ToOwned>::Owned: DekuRead<'a, Ctx>,
    Ctx: Copy,
{
    /// Read a T from input and store as Cow<T>
    fn read(
        input: &'a BitSlice<Msb0, u8>,
        inner_ctx: Ctx,
    ) -> Result<(&'a BitSlice<Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, val) = <<T as ToOwned>::Owned>::read(input, inner_ctx)?;
        Ok((rest, Cow::Owned(val)))
    }
}

impl<'a, T, Ctx> DekuWrite<Ctx> for Cow<'a, T>
where
    T: ToOwned + ?Sized,
    <T as ToOwned>::Owned: DekuWrite<Ctx>,
    &'a T: DekuWrite<Ctx>,
    Ctx: Copy,
{
    /// Write T from Cow<T>
    fn write(&self, output: &mut BitVec<Msb0, u8>, inner_ctx: Ctx) -> Result<(), DekuError> {
        match self {
            Cow::Borrowed(val) => val.write(output, inner_ctx),
            Cow::Owned(val) => val.write(output, inner_ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ctx::{Limit, Size},
        native_endian,
    };
    use rstest::rstest;

    #[rstest(input, expected, expected_rest,
        case(
            &[0xEF, 0xBE],
            Cow::Owned(native_endian!(0xBEEF_u16)),
            bits![Msb0, u8;]
        ),
    )]
    fn test_cow(input: &[u8], expected: Cow<u16>, expected_rest: &BitSlice<Msb0, u8>) {
        let bit_slice = input.view_bits::<Msb0>();
        let (rest, res_read) = <Cow<u16>>::read(bit_slice, ()).unwrap();
        assert_eq!(expected, res_read);
        assert_eq!(expected_rest, rest);

        let mut res_write = bitvec![Msb0, u8;];
        res_read.write(&mut res_write, ()).unwrap();
        assert_eq!(input.to_vec(), res_write.into_vec());
    }

    #[rstest(input, expected, expected_rest,
        case(
            &[0xEF, 0xBE],
            Cow::Owned(vec![native_endian!(0xBEEF_u16)]),
            bits![Msb0, u8;]
        ),
    )]
    fn test_unsized_cow(
        input: &[u8],
        expected: Cow<'_, [u16]>,
        expected_rest: &BitSlice<Msb0, u8>,
    ) {
        let bit_slice = input.view_bits::<Msb0>();
        let (rest, res_read) =
            <Cow<'_, [u16]>>::read(bit_slice, Limit::from(Size::Bytes(2))).unwrap();
        assert_eq!(expected, res_read);
        assert_eq!(expected_rest, rest);

        let mut res_write = bitvec![Msb0, u8;];
        res_read.write(&mut res_write, ()).unwrap();
        assert_eq!(input.to_vec(), res_write.into_vec());
    }
}
