use num_traits;

pub fn binaryBoolAnd<T>(left: T, right: T) -> bool
    where T: num_traits::Unsigned + std::ops::BitAnd<Output = T>{
    if left & right == T::zero() {
        false
    }else {
        true
    }
}

