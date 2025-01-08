use std::marker::PhantomData;

use dashmap::DashMap;
use serde::{de::{SeqAccess, Visitor}, Deserialize, Deserializer};
use std::fmt;

#[derive(Deserialize)]
struct DashMapWrapper<
    K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
    V: Clone + for<'de_a> Deserialize<'de_a>,
    Hasher: Default + std::hash::BuildHasher + Clone,
> {
    #[serde(deserialize_with = "deserialize_dash_map")]
    dash_map: DashMap<K, V, Hasher>,
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
    struct MyVisitor<T, K, V, Hasher>(PhantomData<fn() -> (T, K, V, Hasher)>)
    where
        K: Eq + std::hash::Hash + Clone + for<'de_a> Deserialize<'de_a>,
        V: Clone + for<'de_a> Deserialize<'de_a>,
        Hasher: Default + std::hash::BuildHasher + Clone;

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
            let dash_map = DashMap::<K, V, Hasher>::default();
            while let Some((key, value)) = seq.next_element()? {
                dash_map.insert(key, value);
            }
            Ok(T::from(dash_map))
        }
    }

    let visitor = MyVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}
