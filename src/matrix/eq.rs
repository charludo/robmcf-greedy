use super::Matrix;

impl<T> PartialEq for Matrix<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.num_rows() != other.num_rows() {
            return false;
        }
        if self.num_columns() != other.num_columns() {
            return false;
        }
        for (s, t) in self.indices() {
            if self.get(s, t) != other.get(s, t) {
                return false;
            }
        }
        true
    }
}

impl<T> Eq for Matrix<T> where T: PartialEq {}
