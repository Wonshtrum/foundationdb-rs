#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const FDB_API_VERSION: u32 = FDB_LATEST_API_VERSION;

macro_rules! versions_expand {
    ($($version:tt),+ $(,)?) => {
        versions_expand!(@pre [][$($version)+][][]);
    };
    (@pre [$($max:tt)*][$min:tt $($other:tt)*][$(($($acc2:tt)*))*][$(($($acc3:tt)*))*]) => {
        versions_expand!(@pre [$($max)* $min][$($other)*][$(($($acc2)*))* ($min $($other)*)][$(($($acc3)*))* ($min $($max)*)]);
    };
    (@pre [$($version:tt)*][][$(($($acc2:tt)*))*][$(($($acc3:tt)*))*]) => {
        versions_expand!(@mid ["drop" $($version)*][][][$(($($acc2)*))*][$(($($acc3)*))*]);
    };
    (@mid [$min:tt $($version:tt)*][$max:tt $($other:tt)*][$(($($acc1:tt)*))*][$(($($acc2:tt)*))*][$(($($acc3:tt)*))*]) => {
        versions_expand!(@mid [$min $($version)* $max][$($other)*][$(($($acc1)*))* ($min $max $($version)*)][$(($($acc2)*))*][$(($($acc3)*))*]);
    };
    (@mid [$drop:tt $min:tt $($version:tt)*][][$(($($acc1:tt)*))*][$(($($acc2:tt)*))*][$(($($acc3:tt)*))*]) => {
        versions_expand!(@mid [$min][$($version)*][$(($($acc1)*))*][$(($($acc2)*))*][$(($($acc3)*))*]);
    };
    (@mid [$drop:tt][][$(($($acc1:tt)*))*][$(($($acc2:tt)*))*][$(($($acc3:tt)*))*]) => {
        versions_expand!(@end ($) [$(($($acc1)*))*][$(($($acc2)*))*][$(($($acc3)*))*]);
    };
    (@end ($d:tt)
        [$((($min_name1:tt, $min1:tt) ($max_name1:tt, $max1:tt) $(($version_name1:tt, $version1:tt))*))*]
        [$((($min_name2:tt, $min2:tt) $(($version_name2:tt, $version2:tt))*))*]
        [$((($max_name3:tt, $max3:tt) $(($version_name3:tt, $version3:tt))*))*]
    ) => {
        #[macro_export]
        macro_rules! if_cfg_api_versions {
            $(
                (min=$min1, max=$max1 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any(feature=$min_name1 $(,feature=$version_name1)* ,feature=$max_name1 $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any(feature=$min_name1 $(,feature=$version_name1)* ,feature=$max_name1 $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (min=$min1, max=$max1 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any(feature=$min_name1 $(,feature=$version_name1)* ,feature=$max_name1 $d(,feature=$feature)*))]
                    $d($then)*
                };
            )*
            $(
                (min=$min2 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any(feature=$min_name2 $(,feature=$version_name2)* $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any(feature=$min_name2 $(,feature=$version_name2)* $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (min=$min2 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any(feature=$min_name2 $(,feature=$version_name2)* $d(,feature=$feature)*))]
                    $d($then)*
                };
            )*
            $(
                (max=$max3 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any($(feature=$version_name3,)* feature=$max_name3 $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any($(feature=$version_name3,)* feature=$max_name3 $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (max=$max3 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any($(feature=$version_name3,)* feature=$max_name3 $d(,feature=$feature)*))]
                    $d($then)*
                };
            )*
        }
    };
}

versions_expand![
    ("fdb-5_1", 510),
    ("fdb-5_2", 520),
    ("fdb-6_0", 600),
    ("fdb-6_1", 610),
    ("fdb-6_2", 620),
    ("fdb-6_3", 630),
    ("fdb-7_0", 700),
    ("fdb-7_1", 710),
    ("fdb-7_3", 730),
];
