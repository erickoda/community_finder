#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct OrderedF64(pub f64);

impl Eq for OrderedF64 {}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
