// Computes A * b where `A` is a row-major nxn matrix and `b` is an n-dimensional column vector.
pub fn multiply_col_vector<const N: usize, const M: usize>(a: [f64; N], b: [f64; M]) -> [f64; M] {
    debug_assert!(M * M == N, "Dimension mismatch");

    let mut output = [0.0; M];
    for i in 0..M {
        let row_i = &a[(i * M)..((i + 1) * M)];
        output[i] = row_i.iter().zip(b).map(|(el1, el2)| el1 * el2).sum();
    }
    output
}

// Multiply two row-major square matrices. Will panic if `N` is not a perfect square.
#[cfg_attr(not(test), allow(dead_code))]
pub fn multiply_nxn<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] {
    let n = N.isqrt();
    debug_assert!(n * n == N, "This function only works on square matrices");

    // Transpose b to make multiplying easier
    let b_col_major = {
        let mut tmp = [0.0; N];
        for (idx, el) in b.iter().enumerate() {
            let i = idx / n;
            let j = idx % n;
            let t_idx = j * n + i;
            tmp[t_idx] = *el;
        }
        tmp
    };

    let mut output = [0.0; N];
    for i in 0..n {
        let row_i = &a[(i * n)..((i + 1) * n)];
        for j in 0..n {
            let col_j = &b_col_major[(j * n)..((j + 1) * n)];
            let idx = n * i + j;
            output[idx] = row_i.iter().zip(col_j).map(|(el1, el2)| el1 * el2).sum();
        }
    }
    output
}

// Input is a row-major 2x2 matrix:
// [a b]
// [c d]
pub fn inverse_2x2(input: [f64; 4]) -> [f64; 4] {
    let [a, b, c, d] = input;
    let det = a * d - b * c;
    [d / det, -b / det, -c / det, a / det]
}

// Input is a row-major 3x3 matrix:
// [a b c]
// [d e f]
// [g h i]
pub fn inverse_3x3(input: [f64; 9]) -> [f64; 9] {
    let [a, b, c, d, e, f, g, h, i] = input;
    let det = a * (e * i - h * f) - d * (b * i - h * c) + g * (b * f - e * c);
    [
        (e * i - f * h) / det,
        (c * h - b * i) / det,
        (b * f - c * e) / det,
        (f * g - d * i) / det,
        (a * i - c * g) / det,
        (c * d - a * f) / det,
        (d * h - e * g) / det,
        (b * g - a * h) / det,
        (a * e - b * d) / det,
    ]
}

#[test]
fn test_multiply_col_vec() {
    // The famous bat and ball problem:
    // The sum of bat and ball is 1.10, the
    // difference between bat and ball is 1.0,
    // therefore the bat costs 1.05 and ball costs 0.05.
    let a = [1.0, 1.0, 1.0, -1.0];
    let b = [1.10, 1.0];
    let solution = multiply_col_vector(inverse_2x2(a), b);
    assert!((solution[0] - 1.05).abs() < 1e-7);
    assert!((solution[1] - 0.05).abs() < 1e-7);
}

#[test]
fn test_inverse_2x2() {
    let input = [8.1, 10.5, 6.7, 1.0];
    let inverse = inverse_2x2(input);
    let product = multiply_nxn(input, inverse);
    for (calc, exp) in product.into_iter().zip([1.0, 0.0, 0.0, 1.0]) {
        assert!((calc - exp).abs() < 1e-7);
    }
}

#[test]
fn test_inverse_3x3() {
    let input = [6.0, 8.0, 4.0, 7.0, 5.0, 3.0, 2.0, 9.0, 1.0];
    let inverse = inverse_3x3(input);
    let product = multiply_nxn(input, inverse);
    let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    for (calc, exp) in product.into_iter().zip(identity) {
        assert!((calc - exp).abs() < 1e-7);
    }
}
