// A 2-tuple where both components have the same type. Both `(T, T)` and `&(T, T)`
// satisfy this interface.
pub trait Point {
    type Coord;

    fn fst(self) -> Self::Coord;
    fn snd(self) -> Self::Coord;
}

impl<T> Point for (T, T) {
    type Coord = T;

    fn fst(self) -> Self::Coord {
        self.0
    }

    fn snd(self) -> Self::Coord {
        self.1
    }
}

impl<'a, T> Point for &'a (T, T) {
    type Coord = &'a T;

    fn fst(self) -> Self::Coord {
        &self.0
    }

    fn snd(self) -> Self::Coord {
        &self.1
    }
}
