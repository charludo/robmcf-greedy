use serde::{de::Visitor, Deserialize, Deserializer};

use super::Matrix;

struct MatrixVisitor<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<'de, T> Visitor<'de> for MatrixVisitor<T>
where
    T: Deserialize<'de> + Clone,
{
    type Value = Matrix<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("could not deserialize Matrix")
    }

    fn visit_seq<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut matrix: Vec<Vec<T>> = vec![];
        while let Some(element) = map.next_element::<Vec<T>>()? {
            matrix.push(element);
        }
        Ok(Matrix::from_rows(&matrix))
    }
}

impl<'de, T> Deserialize<'de> for Matrix<T>
where
    T: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MatrixVisitor {
            _phantom: std::marker::PhantomData,
        })
    }
}
