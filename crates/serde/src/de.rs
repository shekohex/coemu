//! Deserializer for Binary Packets.

use crate::TQSerdeError;
use bytes::Buf;
use serde::de::{self, Deserialize, DeserializeSeed, SeqAccess, Visitor};
use std::io::Cursor;

struct Deserializer<'de> {
    input: Cursor<&'de [u8]>,
}

impl<'de> Deserializer<'de> {
    fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: Cursor::new(input),
        }
    }
}
/// Deserialize the given Bytes into `T`.
pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T, TQSerdeError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(s);
    let t = T::deserialize(&mut deserializer)?;
    if !deserializer.input.get_ref().is_empty() {
        Ok(t)
    } else {
        Err(TQSerdeError::Eof)
    }
}

macro_rules! impl_nums {
    ($ty:ty, $dser_method:ident, $visitor_method:ident, $reader_method:ident) => {
        #[inline]
        fn $dser_method<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
        where
            V: serde::de::Visitor<'de>,
        {
            use std::mem::size_of;
            let value = self.input.get_uint_le(size_of::<$ty>()) as $ty;
            visitor.$visitor_method(value)
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = TQSerdeError;

    impl_nums!(u8, deserialize_u8, visit_u8, read_u8);

    impl_nums!(u16, deserialize_u16, visit_u16, read_u16);

    impl_nums!(u32, deserialize_u32, visit_u32, read_u32);

    impl_nums!(u64, deserialize_u64, visit_u64, read_u64);

    impl_nums!(i8, deserialize_i8, visit_i8, read_i8);

    impl_nums!(i16, deserialize_i16, visit_i16, read_i16);

    impl_nums!(i32, deserialize_i32, visit_i32, read_i32);

    impl_nums!(i64, deserialize_i64, visit_i64, read_i64);

    impl_nums!(f32, deserialize_f32, visit_f32, read_f32);

    impl_nums!(f64, deserialize_f64, visit_f64, read_f64);

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(TQSerdeError::DeserializeAnyNotSupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        // 0 = false, 1 = true
        let value = self.input.get_u8();
        match value {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(TQSerdeError::InvalidBool),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        let value = self.input.get_u8();
        visitor.visit_char(value as char)
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(TQSerdeError::Unspported)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        let length = self.input.get_u8();
        let string_bytes = self.input.copy_to_bytes(length as usize);
        let val = String::from_utf8_lossy(&string_bytes);
        let val = val.trim_end_matches('\0');
        visitor.visit_string(val.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        // This implementation assumes that these bytes are stringlist bytes.
        // This means that the first byte is the length of the stringlist
        // and the rest of the bytes are the strings.
        // With that being said, we can just copy the bytes and pass it to the
        // visitor.
        Visitor::visit_bytes(visitor, self.input.chunk())
    }

    fn deserialize_byte_buf<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        let length = self.input.get_u8();
        let bytes = self.input.copy_to_bytes(length as usize);
        visitor.visit_byte_buf(bytes.to_vec())
    }

    fn deserialize_option<V>(
        self,
        _visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(TQSerdeError::Unspported)
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Visitor::visit_unit(visitor)
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        let len = serde::Deserialize::deserialize(&mut *self)?;

        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Access { de: self, len })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(TQSerdeError::Unspported)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        Err(TQSerdeError::Unspported)
    }

    fn deserialize_identifier<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, TQSerdeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct Access<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for Access<'a, 'de> {
    type Error = TQSerdeError;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, TQSerdeError>
    where
        T: DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            let value = DeserializeSeed::deserialize(seed, &mut *self.de)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn test_struct_de() {
    use crate::String16;
    use serde::Deserialize;
    #[derive(Deserialize, Debug, PartialEq)]
    struct MsgAccount {
        username: String16,
        password: String16,
        realm: String16,
        var_string: String,
        code: u32,
    }

    let test: MsgAccount = from_bytes(&[
        0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x31, 0x32, 0x33, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x65, 0x75, 0x73, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa, 0x76, 0x61, 0x72,
        0x5f, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x2, 0x0, 0x0, 0x0,
    ])
    .unwrap();
    assert_eq!(
        MsgAccount {
            username: "testing".into(),
            password: "123".into(),
            realm: "Zeus".into(),
            var_string: "var_string".into(),
            code: 2
        },
        test
    );
}
