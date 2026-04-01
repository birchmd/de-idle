/// Iterator combinator that creates a sliding window of length 3.
pub struct Sliding3<T, I> {
    inner: I,
    a: Option<T>,
    b: Option<T>,
}

impl<T: Copy, I: Iterator<Item = T>> Iterator for Sliding3<T, I> {
    type Item = [T; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let fst = self.a.take()?;
        let snd = self.b.take()?;
        let trd = self.inner.next()?;
        self.a = Some(snd);
        self.b = Some(trd);
        Some([fst, snd, trd])
    }
}

impl<T, I: Iterator<Item = T>> Sliding3<T, I> {
    pub fn new(mut inner: I) -> Self {
        let a = inner.next();
        let b = inner.next();
        Self { inner, a, b }
    }
}

#[test]
fn test_sliding3() {
    let xs: [u8; 5] = [0, 1, 2, 3, 4];
    let ys: Vec<[u8; 3]> = Sliding3::new(xs.iter().copied()).collect();
    assert_eq!(ys, vec![[0, 1, 2], [1, 2, 3], [2, 3, 4]])
}
