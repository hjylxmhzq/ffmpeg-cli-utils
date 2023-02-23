
#[macro_export]
macro_rules! owned {
    ($key:expr, $($rest:expr),*) => {
        vec![$key, $($rest),*].into_iter()
            .map(|item| item.to_owned())
            .collect::<Vec<_>>()
    };
    ($key:expr) => {
        vec![$key].into_iter()
            .map(|item| item.to_owned())
            .collect::<Vec<_>>()
    };
    () => {
        Vec::<_>::new()
    };
}
