#[macro_export]
macro_rules! string_build {
    ($($x:expr),+) => {{
        let mut len = 0;
        $(
            len += $x.len();
        )+
        let mut string = String::with_capacity(len);
        $(
            string.push_str($x);
        )+
        string
    }};
}

