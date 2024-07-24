#![allow(incomplete_features)]
#![feature(specialization, const_for)]
use std::{marker::PhantomData, ops::Deref};

use bytemuck::Zeroable;
#[macro_use]
extern crate static_assertions;
pub mod plugininterfaces;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// pub struct IntegralConstant<T> {
//     value: T,
// }

//pub trait HasIID<T, U, const Val: bool>: BoolConstant<Val> {}

// #[derive(Default)]
// pub struct HasIIDType<T, U>(PhantomData<T>, PhantomData<U>);
// impl BoolConstant<true> for TrueType {}

// impl<Ty, const Val: Ty> Deref for I
// where
//     I: IntegralConstant<Ty, Val>,
// {
//     fn deref(&self) -> &Self::Target {
//         Self::value
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
