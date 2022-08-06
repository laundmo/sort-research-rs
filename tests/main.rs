use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;

use sort_comp::new_stable_sort;
use sort_comp::patterns;
use sort_comp::stdlib_stable;

#[cfg(miri)]
const TEST_SIZES: [usize; 23] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200,
];

#[cfg(not(miri))]
const TEST_SIZES: [usize; 29] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 17, 20, 24, 30, 32, 33, 35, 50, 100, 200, 500, 1_000,
    2_048, 10_000, 100_000, 1_000_000,
];

fn sort_comp<T>(v: &mut [T])
where
    T: Ord + Clone + DeepEqual + Debug,
{
    let is_small_test = v.len() <= 100;
    let original_clone = v.to_vec();

    let mut stdlib_sorted_vec = v.to_vec();
    let stdlib_sorted = stdlib_sorted_vec.as_mut_slice();
    stdlib_stable::sort_by(stdlib_sorted, |a, b| a.cmp(b));

    let fluxsort_sorted = v;
    new_stable_sort::sort_by(fluxsort_sorted, |a, b| a.cmp(b));

    assert_eq!(stdlib_sorted.len(), fluxsort_sorted.len());

    for (a, b) in stdlib_sorted.iter().zip(fluxsort_sorted.iter()) {
        if !a.deep_equal(b) {
            let seed = patterns::random_init_seed();

            if is_small_test {
                eprintln!("Seed: {seed}");
                eprintln!("Orginal:  {:?}", original_clone);
                eprintln!("Expected: {:?}", stdlib_sorted);
                eprintln!("Got:      {:?}", fluxsort_sorted);
            } else {
                // Large arrays output them as files.
                let original_name = format!("original_{}.txt", seed);
                let std_name = format!("stdlib_sorted_{}.txt", seed);
                let flux_name = format!("fluxsort_sorted_{}.txt", seed);

                fs::write(&original_name, format!("{:?}", original_clone)).unwrap();
                fs::write(&std_name, format!("{:?}", stdlib_sorted)).unwrap();
                fs::write(&flux_name, format!("{:?}", fluxsort_sorted)).unwrap();

                eprintln!(
                    "Failed comparison, see files {original_name}, {std_name}, and {flux_name}"
                );
            }

            panic!("Test assertion failed!")
        }
    }
}

// The idea of this struct is to have something that might look the same, based on the sort property
// but can still be different. This helps test that the stable sort algorithm is actually stable.
#[derive(Clone, Debug, Eq)]
struct ValueWithExtra {
    key: i32,
    extra: i32,
}

impl PartialOrd for ValueWithExtra {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Ord for ValueWithExtra {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for ValueWithExtra {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LargeStackVal {
    val: [u128; 4],
}

trait DeepEqual {
    fn deep_equal(&self, other: &Self) -> bool;
}

impl DeepEqual for () {
    fn deep_equal(&self, _other: &Self) -> bool {
        true
    }
}

impl DeepEqual for i32 {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for String {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for LargeStackVal {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for ValueWithExtra {
    fn deep_equal(&self, other: &Self) -> bool {
        self.key.eq(&other.key) && self.extra.eq(&other.extra)
    }
}

fn test_impl<T: Ord + Clone + DeepEqual + Debug>(pattern_fn: impl Fn(usize) -> Vec<T>) {
    for test_size in TEST_SIZES {
        let mut test_data = pattern_fn(test_size);
        sort_comp(test_data.as_mut_slice());
    }
}

pub trait DynTrait: Debug {
    fn get_val(&self) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynValA {
    value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynValB {
    value: i32,
}

impl DynTrait for DynValA {
    fn get_val(&self) -> i32 {
        self.value
    }
}
impl DynTrait for DynValB {
    fn get_val(&self) -> i32 {
        self.value
    }
}

impl PartialOrd for dyn DynTrait {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_val().partial_cmp(&other.get_val())
    }
}

impl Ord for dyn DynTrait {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for dyn DynTrait {
    fn eq(&self, other: &Self) -> bool {
        self.get_val() == other.get_val()
    }
}

impl Eq for dyn DynTrait {}

impl DeepEqual for Rc<dyn DynTrait> {
    fn deep_equal(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

// --- TESTS ---

#[test]
fn basic() {
    sort_comp::<i32>(&mut []);
    sort_comp::<()>(&mut []);
    sort_comp::<()>(&mut [()]);
    sort_comp::<()>(&mut [(), ()]);
    sort_comp::<()>(&mut [(), (), ()]);
    sort_comp(&mut [2, 3]);
    sort_comp(&mut [2, 3, 6]);
    sort_comp(&mut [2, 3, 99, 6]);
    sort_comp(&mut [2, 7709, 400, 90932]);
    sort_comp(&mut [15, -1, 3, -1, -3, -1, 7]);
}

#[test]
fn value_with_extra() {
    let a = ValueWithExtra { key: 6, extra: 9 };
    let b = ValueWithExtra { key: 7, extra: 9 };
    let c = ValueWithExtra { key: 7, extra: 10 };

    assert!(a < b);
    assert!(a < c);
    assert!(b > a);
    assert!(c > a);
    assert!(a != b);
    assert!(a != c);
    assert!(b == c);
    assert!(b == c);

    assert!(!a.deep_equal(&b));
    assert!(!a.deep_equal(&c));
    assert!(!b.deep_equal(&c));
}

#[test]
fn random() {
    test_impl(patterns::random);
}

#[test]
fn all_equal() {
    test_impl(patterns::all_equal);
}

#[test]
fn ascending() {
    test_impl(patterns::ascending);
}

#[test]
fn descending() {
    test_impl(patterns::descending);
}

#[test]
fn ascending_saw() {
    test_impl(|test_size| patterns::ascending_saw(test_size, test_size / 5));
    test_impl(|test_size| patterns::ascending_saw(test_size, test_size / 20));
}

#[test]
fn descending_saw() {
    test_impl(|test_size| patterns::descending_saw(test_size, test_size / 5));
    test_impl(|test_size| patterns::descending_saw(test_size, test_size / 20));
}

#[test]
fn pipe_organ() {
    test_impl(patterns::pipe_organ);
}

#[test]
fn random_duplicates() {
    // This test is designed to stress test stable sorting.
    test_impl(|test_size| {
        let random = patterns::random(test_size);
        let uni = patterns::random_uniform(test_size, 0..(test_size / 10) as i32);

        uni.into_iter()
            .zip(random.into_iter())
            .map(|(key, extra)| ValueWithExtra { key, extra })
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_str() {
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| format!("{}", val))
            .collect::<Vec<_>>()
    });
}

#[test]
fn random_large_val() {
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| {
                let val_abs = val.abs() as u128;
                LargeStackVal {
                    val: [
                        val_abs.wrapping_sub(6),
                        val_abs.wrapping_add(3),
                        val_abs.wrapping_sub(2),
                        val_abs,
                    ],
                }
            })
            .collect::<Vec<_>>()
    });
}

#[test]
fn dyn_val() {
    // Dyn values are fat pointers, something the implementation might have overlooked.
    test_impl(|test_size| {
        patterns::random(test_size)
            .into_iter()
            .map(|val| -> Rc<dyn DynTrait> {
                if val < (i32::MAX / 2) {
                    Rc::new(DynValA { value: val })
                } else {
                    Rc::new(DynValB { value: val })
                }
            })
            .collect::<Vec<Rc<dyn DynTrait>>>()
    });
}

#[test]
fn comp_panic() {
    // Test that sorting upholds panic safety.
    // This means, no non trivial duplicates even if a comparison panics.
    // The invariant being checked is, will miri complain.

    let seed = patterns::random_init_seed();

    for test_size in TEST_SIZES {
        // Needs to be non trivial dtor.
        let mut values = patterns::random(test_size)
            .into_iter()
            .map(|val| vec![val, val, val])
            .collect::<Vec<Vec<i32>>>();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            new_stable_sort::sort_by(&mut values, |a, b| {
                if a[0].abs() < (i32::MAX / test_size as i32) {
                    panic!(
                        "Explicit panic. Seed: {}. test_size: {}. a: {} b: {}",
                        seed, test_size, a[0], b[0]
                    );
                }

                a[0].cmp(&b[0])
            });

            values
                .get(values.len().saturating_sub(1))
                .map(|val| val[0])
                .unwrap_or(66)
        }));
    }
}
