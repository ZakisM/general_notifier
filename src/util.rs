#[macro_export]
macro_rules! hash_id {
    ( $( $field:expr ),+ ) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        $(
            $field.hash(&mut hasher);
        )+

        format!("{:x}", hasher.finish())
    }}
}
