use crate::{Input, Output};

impl<C> Controller for &mut C
where
    C: Controller,
{
    fn update(&mut self, output: &Output, input: &mut Input) {
        (*self).update(output, input);
    }
}

pub trait Controller {
    fn update(&mut self, output: &Output, input: &mut Input);
}

macro_rules! impl_controller_tupple {
    ($($idx:tt $T:tt),* $(,)?) => {
        impl<$($T,)*> Controller for ($($T,)*)
        where
            $($T: Controller,)*
        {
            fn update(&mut self, output: &Output, input: &mut Input) {
                $(self.$idx.update(output, input);)*
            }
        }
    };
}

impl Controller for () {
    fn update(&mut self, _output: &Output, _input: &mut Input) {}
}

impl_controller_tupple!(0 A);
impl_controller_tupple!(0 A, 1 B);
impl_controller_tupple!(0 A, 1 B, 2 C);
impl_controller_tupple!(0 A, 1 B, 2 C, 3 D);
impl_controller_tupple!(0 A, 1 Bf, 2 C, 3 D, 4 E);
impl_controller_tupple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
