pub mod point;
pub mod sequence;
pub mod sliding3;

pub fn min_max<I, T>(x0: T, xs: I) -> (T, T)
where
    T: PartialOrd + Copy,
    I: Iterator<Item = T>,
{
    xs.fold((x0, x0), |(min, max), x| {
        if x < min {
            (x, max)
        } else if max < x {
            (min, x)
        } else {
            (min, max)
        }
    })
}
