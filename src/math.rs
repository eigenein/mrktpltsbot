use std::ops::{Div, Rem};

pub fn div_rem<T: Div<Output = T> + Rem<Output = T> + Copy>(left: T, right: T) -> (T, T) {
    let quotient = left / right;
    let remainder = left % right;
    (quotient, remainder)
}
