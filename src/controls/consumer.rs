use crate::{consumer::Consumer, controls};

// use crate::messaging::SubstituteGetter;

// pub fn example(category: &str) -> Consumer<SubstituteGetter> {
//     todo!("")
// }

pub mod builder;
pub mod subscription_substitute;

pub fn example() -> Consumer {
    Consumer::new(controls::messages::category::example())
}
