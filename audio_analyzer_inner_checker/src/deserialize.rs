use std::marker::PhantomData;

use dashmap::DashMap;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

pub struct DashMapWrapper<
    K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
    V: Clone + for<'de_a> Deserialize<'de_a>,
    Hasher: Default + std::hash::BuildHasher + Clone,
> {
    pub dash_map: DashMap<K, V, Hasher>,
}

impl<'de, K, V, Hasher> Deserialize<'de> for DashMapWrapper<K, V, Hasher>
where
    K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
    V: Clone + for<'de_a> Deserialize<'de_a>,
    Hasher: Default + std::hash::BuildHasher + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dash_map =
            deserialize_dash_map::<DashMap<K, V, Hasher>, D, K, V, Hasher>(deserializer)?;

        Ok(DashMapWrapper { dash_map })
    }
}

// https://serde.rs/stream-array.html

fn deserialize_dash_map<'de, T, D, K, V, Hasher>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + From<DashMap<K, V, Hasher>>,
    D: Deserializer<'de>,
    K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
    V: Clone + for<'de_a> Deserialize<'de_a>,
    Hasher: Default + std::hash::BuildHasher + Clone,
{
    struct MyVisitor<T, K, V, Hasher>(PhantomData<fn() -> (T, K, V, Hasher)>);

    impl<'de, T, K, V, Hasher> Visitor<'de> for MyVisitor<T, K, V, Hasher>
    where
        T: Deserialize<'de> + From<DashMap<K, V, Hasher>>,
        K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
        V: Clone + for<'de_a> Deserialize<'de_a>,
        Hasher: Default + std::hash::BuildHasher + Clone,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<T, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let dash_map = DashMap::with_hasher(Hasher::default());

            while let Some((k, v)) = seq.next_element::<(K, V)>()? {
                dash_map.insert(k, v);
            }

            Ok(T::from(dash_map))
        }
    }

    deserializer.deserialize_seq(MyVisitor(PhantomData))
}
