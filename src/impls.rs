use crate::{
    DeserializeError, DeserializeFromValue, IntoValue, Map, Sequence, Value, ValueKind,
    ValuePointerRef,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    convert::TryFrom,
    hash::Hash,
    str::FromStr,
};

impl<T> Sequence for Vec<T>
where
    T: IntoValue,
{
    type Value = T;
    type Iter = <Self as IntoIterator>::IntoIter;

    fn len(&self) -> usize {
        self.len()
    }

    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<E> DeserializeFromValue<E> for ()
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Null => Ok(()),
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Null],
                location,
            )),
        }
    }
}

impl<E> DeserializeFromValue<E> for bool
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Boolean(b) => Ok(b),
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Boolean],
                location,
            )),
        }
    }
}

macro_rules! deserialize_impl_integer {
    ($t:ty) => {
        impl<E> DeserializeFromValue<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                let err = |kind: ValueKind| {
                    E::incorrect_value_kind(kind, &[ValueKind::Integer], location)
                };

                match value {
                    Value::Integer(x) => <$t>::try_from(x).map_err(|_| {
                        E::unexpected(
                            &format!(
                                "Cannot deserialize {x} into a {}",
                                std::any::type_name::<$t>()
                            ),
                            location,
                        )
                    }),
                    Value::NegativeInteger(x) => <$t>::try_from(x).map_err(|_| {
                        E::unexpected(
                            &format!(
                                "Cannot deserialize {x} into a {}",
                                std::any::type_name::<$t>()
                            ),
                            location,
                        )
                    }),
                    v @ _ => Err(err(v.kind())),
                }
            }
        }
    };
}
deserialize_impl_integer!(u8);
deserialize_impl_integer!(u16);
deserialize_impl_integer!(u32);
deserialize_impl_integer!(u64);
deserialize_impl_integer!(usize);

macro_rules! deserialize_impl_negative_integer {
    ($t:ty) => {
        impl<E> DeserializeFromValue<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                let err = |kind: ValueKind| {
                    E::incorrect_value_kind(
                        kind,
                        &[ValueKind::Integer, ValueKind::NegativeInteger],
                        location,
                    )
                };

                match value {
                    Value::Integer(x) => <$t>::try_from(x).map_err(|_| {
                        E::unexpected(
                            &format!(
                                "Cannot deserialize {x} into a {}",
                                std::any::type_name::<$t>()
                            ),
                            location,
                        )
                    }),
                    Value::NegativeInteger(x) => <$t>::try_from(x).map_err(|_| {
                        E::unexpected(
                            &format!(
                                "Cannot deserialize {x} into a {}",
                                std::any::type_name::<$t>()
                            ),
                            location,
                        )
                    }),
                    v @ _ => Err(err(v.kind())),
                }
            }
        }
    };
}

deserialize_impl_negative_integer!(i8);
deserialize_impl_negative_integer!(i16);
deserialize_impl_negative_integer!(i32);
deserialize_impl_negative_integer!(i64);
deserialize_impl_negative_integer!(isize);

macro_rules! deserialize_impl_float {
    ($t:ty) => {
        impl<E> DeserializeFromValue<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                match value {
                    Value::Integer(x) => {
                        return Ok(x as $t);
                    }
                    Value::NegativeInteger(x) => {
                        return Ok(x as $t);
                    }
                    Value::Float(x) => {
                        return Ok(x as $t);
                    }
                    v @ _ => {
                        return Err(E::incorrect_value_kind(
                            v.kind(),
                            &[
                                ValueKind::Float,
                                ValueKind::Integer,
                                ValueKind::NegativeInteger,
                            ],
                            location,
                        ))
                    }
                };
            }
        }
    };
}
deserialize_impl_float!(f32);
deserialize_impl_float!(f64);

impl<E> DeserializeFromValue<E> for String
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::String(x) => Ok(x),
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::String],
                location,
            )),
        }
    }
}

impl<T, E> DeserializeFromValue<E> for Vec<T>
where
    T: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => seq
                .into_iter()
                .enumerate()
                .map(|(index, x)| {
                    let result =
                        T::deserialize_from_value(x.into_value(), location.push_index(index));
                    result
                })
                .collect(),
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Sequence],
                location,
            )),
        }
    }
}

impl<T, E> DeserializeFromValue<E> for Option<T>
where
    T: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Null => Ok(None),
            value => T::deserialize_from_value(value, location).map(Some),
        }
    }
    fn default() -> Option<Self> {
        Some(None)
    }
}

impl<T, E> DeserializeFromValue<E> for Box<T>
where
    T: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        T::deserialize_from_value(value, location).map(Box::new)
    }
}

impl<Key, T, E> DeserializeFromValue<E> for HashMap<Key, T>
where
    Key: FromStr + Hash + Eq,
    T: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut res = HashMap::with_capacity(map.len());
                for (string_key, value) in map.into_iter() {
                    let key = Key::from_str(&string_key).map_err(|_| {
                        E::unexpected(&format!(
                                "The key \"{string_key}\" could not be deserialized into the key type `{}`",
                                std::any::type_name::<Key>()
                            ), location
                        )
                    })?;
                    let value = T::deserialize_from_value(
                        value.into_value(),
                        location.push_key(&string_key),
                    )?;
                    res.insert(key, value);
                }
                Ok(res)
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Map],
                location,
            )),
        }
    }
}

impl<Key, T, E> DeserializeFromValue<E> for BTreeMap<Key, T>
where
    Key: FromStr + Ord,
    T: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut res = BTreeMap::new();
                for (string_key, value) in map.into_iter() {
                    let key = Key::from_str(&string_key).map_err(|_| {
                        E::unexpected(&format!(
                                "The key \"{string_key}\" could not be deserialized into the key type `{}`",
                                std::any::type_name::<Key>()
                            ), location
                        )}
                    )?;
                    let value = T::deserialize_from_value(
                        value.into_value(),
                        location.push_key(&string_key),
                    )?;
                    res.insert(key, value);
                }
                Ok(res)
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Map],
                location,
            )),
        }
    }
}

impl<T, E> DeserializeFromValue<E> for HashSet<T>
where
    T: DeserializeFromValue<E> + Hash + Eq,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let mut res = HashSet::with_capacity(seq.len());
                for (i, value) in seq.into_iter().enumerate() {
                    let value =
                        T::deserialize_from_value(value.into_value(), location.push_index(i))?;
                    res.insert(value);
                }
                Ok(res)
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Sequence],
                location,
            )),
        }
    }
}

impl<T, E> DeserializeFromValue<E> for BTreeSet<T>
where
    T: DeserializeFromValue<E> + Ord,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let mut res = BTreeSet::new();
                for (i, value) in seq.into_iter().enumerate() {
                    let value =
                        T::deserialize_from_value(value.into_value(), location.push_index(i))?;
                    res.insert(value);
                }
                Ok(res)
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Sequence],
                location,
            )),
        }
    }
}

impl<A, B, E> DeserializeFromValue<E> for (A, B)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len != 2 {
                    return Err(E::unexpected(
                        "The sequence should have exactly 2 elements.",
                        location,
                    ));
                }

                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(0),
                )?;
                let b = B::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(1),
                )?;
                Ok((a, b))
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Sequence],
                location,
            )),
        }
    }
}

impl<A, B, C, E> DeserializeFromValue<E> for (A, B, C)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
    C: DeserializeFromValue<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len != 2 {
                    return Err(E::unexpected(
                        "The sequence should have exactly 3 elements.",
                        location,
                    ));
                }

                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(0),
                )?;
                let b = B::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(1),
                )?;
                let c = C::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(2),
                )?;

                Ok((a, b, c))
            }
            v @ _ => Err(E::incorrect_value_kind(
                v.kind(),
                &[ValueKind::Sequence],
                location,
            )),
        }
    }
}
