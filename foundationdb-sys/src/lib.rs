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
    (@end ($d:tt) [$(($min1:tt $max1:tt $($version1:tt)*))*][$(($min2:tt $($version2:tt)*))*][$(($max3:tt $($version3:tt)*))*]) => {
        #[macro_export]
        macro_rules! if_cfg_api_versions {
            $(
                (min=$min1, max=$max1 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any(feature=$min1 $(,feature=$version1)* ,feature=$max1 $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any(feature=$min1 $(,feature=$version1)* ,feature=$max1 $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (min=$min1, max=$max1 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any(feature=$min1 $(,feature=$version1)* ,feature=$max1 $d(,feature=$feature)*))]
                    $d($then)*
                };
            )*
            $(
                (min=$min2 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any(feature=$min2 $(,feature=$version2)* $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any(feature=$min2 $(,feature=$version2)* $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (min=$min2 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any(feature=$min2 $(,feature=$version2)* $d(,feature=$feature)*))]
                    $d($then)*
                };
            )*
            $(
                (max=$max3 $d(,feature = $feature:literal)* => {$d($then:tt)*} else {$d($else:tt)*}) => {{
                    #[cfg(any($(feature=$version3,)* feature=$max3 $d(,feature=$feature)*))]
                    { $d($then)* }
                    #[cfg(not(any($(feature=$version3,)* feature=$max3 $d(,feature=$feature)*)))]
                    { $d($else)* }
                }};
                (max=$max3 $d(,feature = $feature:literal)* => $d($then:tt)*) => {
                    #[cfg(any($(feature=$version3,)* feature=$max3 $d(,feature=$feature)*))]
                    { $d($then)* }
                };
            )*
        }
    };
}

versions_expand![
    "fdb-5_1", "fdb-5_2", "fdb-6_0", "fdb-6_1", "fdb-6_2", "fdb-6_3", "fdb-7_0", "fdb-7_1",
    "fdb-7_3", "fdb-7_4"
];
