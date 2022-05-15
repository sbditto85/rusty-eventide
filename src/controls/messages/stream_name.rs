use crate::messaging::{
    category::{Category, CategoryTypes},
    stream_name::StreamName,
};

pub fn stream_name(mut category: Category, _id: Option<&str>, types: CategoryTypes) -> StreamName {
    // TODO: finish this out

    if types.len() > 0 {
        let type_list: String = types.join("+");
        category += &type_list;
    }

    category
}
