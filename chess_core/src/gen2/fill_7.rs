use crate::BB;

macro_rules! fill_7 {
    ($($name:ident $([$exclude:expr])*($($c:tt)*),)*) => {
        $(
            #[inline]
            pub fn $name(pieces: BB, empty: BB) -> BB{
                $(let empty = empty & !$exclude;)*
                let mut flood = pieces;
                let pieces = (pieces $($c)*) & empty;
                flood |= pieces;
                let pieces = (pieces $($c)*) & empty;
                flood |= pieces;
                let pieces = (pieces $($c)*) & empty;
                flood |= pieces;
                let pieces = (pieces $($c)*) & empty;
                flood |= pieces;
                let pieces = (pieces $($c)*) & empty;
                flood |= pieces;
                flood |= (pieces $($c)*) & empty;
                (flood $($c)*) $(& !$exclude)*
            }
        )*
    };
}

fill_7! {
    nw[BB::FILE_H]( << 7),
    n( << 8),
    ne[BB::FILE_A]( << 9),
    e[BB::FILE_A]( << 1),
    se[BB::FILE_A]( >> 7),
    s( >> 8),
    sw[BB::FILE_H]( >> 9),
    w[BB::FILE_H]( >> 1),
}
