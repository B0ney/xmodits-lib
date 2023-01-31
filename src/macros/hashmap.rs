// #[macro_export]
// macro_rules! Hash {
//     () => {
//         compile_fail!()
//     };
// }
// macro_mash_map!(
//     LOADERS {
//         "it" | "mptm" => IT,
//         "s3m" => S3M,
//         "mod" => MOD,
//         "xm" => XM,
//         "umx" => UMX,
//         _ => () // <-- optional
//     }
// );
// trait MacroHash<T> {
//     fn get()
// }

// /// Compile-time, immutable hash-like struct
// pub struct MacroHash<K, V, const S: usize> {
//     pub key_values: [(K, V); S]
// }

// impl <K, V, const S: usize>MacroHash<K,V, S> {
//     pub fn get(key: K) -> Option<V> {
//         match key {
//             "" => (),
//             _ => None
//         }
//         // None
//     }
// }

// #[test]
// fn a() {
//     let f = MacroHash::<usize, usize, 1> { key_values: [(7,5)]};
// }
