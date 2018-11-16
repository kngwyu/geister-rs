use num_traits::PrimInt;
use rand::{
    distributions::uniform::SampleUniform, thread_rng, Error as RndError, Rng, RngCore, SeedableRng,
};
use rand_xorshift::XorShiftRng;
use std::mem;
use std::ops::{Bound, Range, RangeBounds};

/// a set implementation using Fenwick Tree
#[derive(Clone, Debug)]
pub struct FenwickSet {
    inner: FenwickTree,
    num_elements: usize,
    max_val_excluded: usize,
}

impl Default for FenwickSet {
    fn default() -> Self {
        FenwickSet::with_capacity(10)
    }
}

impl FenwickSet {
    /// create a new set with capacity [0..n)
    pub fn with_capacity(n: usize) -> Self {
        assert!(
            n <= 50_000_000,
            "We can't construct too big FenwickSet: size {}",
            n
        );
        FenwickSet {
            inner: FenwickTree::new(n),
            num_elements: 0,
            max_val_excluded: n,
        }
    }
    /// create a new set from range `range` with the capacity [0..range.end)
    /// and already have elements [range.start..range.end)
    pub fn from_range(range: Range<usize>) -> Self {
        let mut set = FenwickSet::with_capacity(range.end);
        range.for_each(|i| {
            set.insert(i);
        });
        set
    }
    /// Insert an element `elem` into set
    /// if `elem` is already in the set, return false.
    /// if not, return true.
    pub fn insert(&mut self, elem: usize) -> bool {
        if elem >= self.max_val_excluded || self.contains(elem) {
            false
        } else {
            self.inner.add(elem, 1);
            self.num_elements += 1;
            true
        }
    }
    /// Remove an element `elem` from set
    /// if `elem` is in the set, return true.
    /// if not, return false.
    pub fn remove(&mut self, elem: usize) -> bool {
        if elem >= self.max_val_excluded || !self.contains(elem) || self.num_elements == 0 {
            false
        } else {
            self.inner.add(elem, -1);
            self.num_elements -= 1;
            true
        }
    }
    /// Checks if the set cotains a element `elem`
    pub fn contains(&self, elem: usize) -> bool {
        if elem >= self.max_val_excluded {
            return false;
        }
        self.inner.sum_range(elem..elem + 1) == 1
    }
    /// return nth-smallest element in the set
    pub fn nth(&self, n: usize) -> Option<usize> {
        let res = self.inner.lower_bound(n as i32 + 1);
        if res >= self.max_val_excluded {
            None
        } else {
            Some(res)
        }
    }
    /// return how many elements the set has
    pub fn len(&self) -> usize {
        self.num_elements
    }
    /// select one integer randomly from the set
    pub fn select<R: Rng>(&self, rng: &mut R) -> Option<usize> {
        if self.num_elements == 0 {
            return None;
        }
        let num = rng.gen_range(0, self.num_elements);
        self.nth(num)
    }
    pub fn iter<'a>(&'a self) -> FwsIter<'a> {
        FwsIter {
            fwt: &self.inner,
            current: 0,
            before: 0,
        }
    }
}

impl IntoIterator for FenwickSet {
    type Item = usize;
    type IntoIter = FwsIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        FwsIntoIter {
            fwt: self.inner,
            current: 0,
            before: 0,
        }
    }
}

/// Iterator for FenwickSet which has entitty
pub struct FwsIntoIter {
    fwt: FenwickTree,
    current: isize,
    before: i32,
}

impl Iterator for FwsIntoIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        fws_iter_next(&self.fwt, &mut self.current, &mut self.before)
    }
}

/// Iterator for FenwickSet which has reference
pub struct FwsIter<'a> {
    fwt: &'a FenwickTree,
    current: isize,
    before: i32,
}

impl<'a> Iterator for FwsIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        fws_iter_next(&self.fwt, &mut self.current, &mut self.before)
    }
}

#[inline]
fn fws_iter_next(fwt: &FenwickTree, current: &mut isize, before: &mut i32) -> Option<usize> {
    while *current < fwt.len {
        *current += 1;
        let sum = fwt.sum(*current as usize);
        let diff = sum - *before;
        *before = sum;
        if diff == 1 {
            return Some(*current as usize - 1);
        }
    }
    None
}

/// simple 0-indexed fenwick tree
#[derive(Clone, Debug)]
struct FenwickTree {
    inner: Vec<i32>,
    len: isize,
}

impl FenwickTree {
    fn new(length: usize) -> Self {
        FenwickTree {
            inner: vec![0; length + 1],
            len: length as isize,
        }
    }
    /// add plus to array[idx]
    fn add(&mut self, idx: usize, plus: i32) {
        let mut idx = (idx + 1) as isize;
        while idx <= self.len {
            self.inner[idx as usize] += plus;
            idx += idx & -idx;
        }
    }
    /// return sum of range 0..range_max
    fn sum(&self, range_max: usize) -> i32 {
        let mut sum = 0;
        let mut idx = range_max as isize;
        while idx > 0 {
            sum += self.inner[idx as usize];
            idx -= idx & -idx;
        }
        sum
    }
    /// return sum of range 0..range_max
    fn sum_range(&self, range: Range<usize>) -> i32 {
        let sum1 = self.sum(range.end);
        if range.start == 0 {
            return sum1;
        } else {
            let sum2 = self.sum(range.start);
            sum1 - sum2
        }
    }
    /// return minimum i where array[0] + array[1] + ... + array[i] >= query (1 <= i <= N)
    fn lower_bound(&self, mut query: i32) -> usize {
        if query <= 0 {
            return 0;
        }
        let mut k = 1;
        while k <= self.len {
            k *= 2;
        }
        let mut cur = 0;
        while k > 0 {
            k /= 2;
            let nxt = cur + k;
            if nxt > self.len {
                continue;
            }
            let val = self.inner[nxt as usize];
            if val < query {
                query -= val;
                cur += k;
            }
        }
        cur as usize
    }
}

fn bounds_to_range<T: PrimInt>(r: impl RangeBounds<T>) -> Range<T> {
    let s = match r.start_bound() {
        Bound::Excluded(t) => *t + T::one(),
        Bound::Included(t) => *t,
        Bound::Unbounded => T::min_value(),
    };
    let g = match r.end_bound() {
        Bound::Excluded(t) => *t,
        Bound::Included(t) => *t + T::one(),
        Bound::Unbounded => T::max_value(),
    };
    s..g
}

/// wrapper of XorShiftRng
#[derive(Clone, Debug)]
pub struct RngHandle(XorShiftRng);

impl Default for RngHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub fn gen_seed() -> u128 {
    let mut rng = thread_rng();
    rng.gen()
}

impl RngHandle {
    fn gen_seed(seed: u128) -> [u8; 16] {
        unsafe { mem::transmute::<_, [u8; 16]>(seed) }
    }
    /// create new Rng by specified seed
    pub fn from_seed(seed: u128) -> Self {
        let seed = Self::gen_seed(seed);
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// create new Rng by random seed
    pub fn new() -> Self {
        let seed: [u8; 16] = thread_rng().gen();
        RngHandle(XorShiftRng::from_seed(seed))
    }
    /// select some values randomly from given range
    pub fn select<T: PrimInt>(&mut self, range: impl RangeBounds<T>) -> RandomSelecter<T> {
        let range = bounds_to_range(range);
        let width = range.end - range.start;
        let width = width.to_usize().expect("[RngHandle::select] NumCast error");
        if width > 10_000_000 {
            panic!("[RngHandle::select] too large range");
        }
        RandomSelecter {
            offset: range.start,
            selected: FenwickSet::from_range(0..width),
            rng: self,
        }
    }
    /// select some values randomly using given FenwickSet
    pub fn select_with<T: PrimInt>(&mut self, set: FenwickSet) -> RandomSelecter<T> {
        RandomSelecter {
            offset: T::zero(),
            selected: set,
            rng: self,
        }
    }
    /// wrapper of gen_range which takes Range
    pub fn range<T: PrimInt + SampleUniform>(&mut self, range: impl RangeBounds<T>) -> T {
        let range = bounds_to_range(range);
        let (s, e) = (range.start, range.end);
        assert!(s < e, "invalid range!!");
        self.0.gen_range(s, e)
    }
    /// judge an event with happenig probability 1 / p_inv happens or not
    pub fn does_happen(&mut self, p_inv: u32) -> bool {
        self.gen_range(0, p_inv) == 0
    }
    /// judge an event with p % chance happens or not
    pub fn parcent(&mut self, p: Parcent) -> bool {
        p.valid_check();
        self.range(1..=100) <= p.0
    }
}

impl RngCore for RngHandle {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }
    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }
    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RndError> {
        self.0.try_fill_bytes(dest)
    }
}

/// Iterator for RngHandle::select
pub struct RandomSelecter<'a, T: PrimInt> {
    offset: T,
    selected: FenwickSet,
    rng: &'a mut RngHandle,
}

impl<'a, T: PrimInt> Iterator for RandomSelecter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let num_rests = self.selected.len();
        if num_rests == 0 {
            return None;
        }
        let n = self.rng.gen_range(0, num_rests);
        let res = self
            .selected
            .nth(n)
            .expect("[RandomSelecter::next] no nth element(maybe logic bug)");
        self.selected.remove(res);
        let res = T::from(res).expect("[RngSelect::Iterator::next] NumCast error") + self.offset;
        Some(res)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Parcent(u32);

impl Parcent {
    fn valid_check(self) {
        debug_assert!(self.0 <= 100, "Invalid parcentage {}", self.0);
    }
    pub const fn new(u: u32) -> Self {
        Parcent(u)
    }
}
