use std::slice;
use std::vec;
use std::iter;
use std::cmp;

#[derive(Clone)]
/// Create an iterator running multiple iterators in lockstep.
///
/// The iterator **Zip\<(I, J, ..., M)\>** is formed from a tuple of iterators and yields elements
/// until any of the subiterators yields **None**.
///
/// Iterator element type is like **(A, B, ..., E)** where **A** to **E** are the respective
/// subiterator types.
///
/// ## Example
///
/// ```
/// use itertools::Zip;
///
/// // Iterate over three sequences side-by-side
/// let mut xs = [0, 0, 0];
/// let ys = [69, 107, 101];
///
/// for (i, a, b) in Zip::new((0i32..100, xs.iter_mut(), ys.iter())) {
///    *a = i ^ *b;
/// }
///
/// assert_eq!(xs, [69, 106, 103]);
/// ```
pub struct Zip<T> {
    t: T
}

impl<T> Zip<T> where Zip<T>: Iterator
{
    /// Create a new **Zip** from a tuple of iterators.
    pub fn new(t: T) -> Zip<T>
    {
        Zip{t: t}
    }
}

macro_rules! impl_zip_iter {
    ($($B:ident),*) => (
        #[allow(non_snake_case)]
        impl<$($B),*> Iterator for Zip<($($B,)*)>
            where
            $(
                $B: Iterator,
            )*
        {
            type Item = ($($B::Item,)*);

            fn next(&mut self) -> Option<
                    ($($B::Item,)*)
                >
            {
                let &mut Zip { t : ($(ref mut $B,)*)} = self;
                // WARNING: partial consume possible
                // Zip worked the same.
                $(
                    let $B = match $B.next() {
                        None => return None,
                        Some(elt) => elt
                    };
                )*
                Some(($($B,)*))
            }

            fn size_hint(&self) -> (usize, Option<usize>)
            {
                let low = ::std::usize::MAX;
                let high = None;
                let &Zip { t : ($(ref $B,)*) } = self;
                $(
                    // update estimate
                    let (l, h) = $B.size_hint();
                    let low = cmp::min(low, l);
                    let high = match (high, h) {
                        (Some(u1), Some(u2)) => Some(cmp::min(u1, u2)),
                        _ => high.or(h)
                    };
                )*
                (low, high)
            }
        }
    );
}

impl_zip_iter!(A);
impl_zip_iter!(A, B);
impl_zip_iter!(A, B, C);
impl_zip_iter!(A, B, C, D);
impl_zip_iter!(A, B, C, D, E);
impl_zip_iter!(A, B, C, D, E, F);
impl_zip_iter!(A, B, C, D, E, F, G);
impl_zip_iter!(A, B, C, D, E, F, G, H);
impl_zip_iter!(A, B, C, D, E, F, G, H, I);


/// A **TrustedIterator** has exact size, always.
///
/// **Note:** TrustedIterator is *Experimental.*
pub unsafe trait TrustedIterator : ExactSizeIterator
{
    /* no methods */
}

unsafe impl TrustedIterator for ::std::ops::Range<usize> { }
unsafe impl TrustedIterator for ::std::ops::Range<u32> { }
unsafe impl TrustedIterator for ::std::ops::Range<i32> { }
unsafe impl TrustedIterator for ::std::ops::Range<u16> { }
unsafe impl TrustedIterator for ::std::ops::Range<i16> { }
unsafe impl TrustedIterator for ::std::ops::Range<u8> { }
unsafe impl TrustedIterator for ::std::ops::Range<i8> { }
unsafe impl<'a, T> TrustedIterator for slice::Iter<'a, T> { }
unsafe impl<'a, T> TrustedIterator for slice::IterMut<'a, T> { }
unsafe impl<T> TrustedIterator for vec::IntoIter<T> { }

unsafe impl<I> TrustedIterator for iter::Rev<I> where
    I: DoubleEndedIterator + TrustedIterator,
{ }
unsafe impl<I> TrustedIterator for iter::Take<I> where
    I: TrustedIterator,
{ }


#[derive(Clone)]
/// Create an iterator running multiple iterators in lockstep.
///
/// **ZipTrusted** is an experimental version of **Zip**, and it can only use iterators that are
/// known to provide their exact size up front. The lockstep iteration can then compile to faster
/// code, ideally not checking more than once per lap for the end of iteration.
///
/// The iterator **ZipTrusted\<(I, J, ..., M)\>** is formed from a tuple of iterators and yields elements
/// until any of the subiterators yields **None**.
///
/// Iterator element type is like **(A, B, ..., E)** where **A** to **E** are the respective
/// subiterator types.
///
/// ## Example
///
/// ```
/// use itertools::ZipTrusted;
///
/// // Iterate over three sequences side-by-side
/// let mut xs = [0, 0, 0];
/// let ys = [69, 107, 101];
///
/// for (i, a, b) in ZipTrusted::new((0..100, xs.iter_mut(), ys.iter())) {
///    *a = i ^ *b;
/// }
///
/// assert_eq!(xs, [69, 106, 103]);
/// ```
pub struct ZipTrusted<T> {
    length: usize,
    t: T
}

trait SetLength {
    fn set_length(&mut self);
}

impl<T> ZipTrusted<T> where ZipTrusted<T>: SetLength
{
    /// Create a new **ZipTrusted** from a tuple of iterators.
    #[inline]
    pub fn new(t: T) -> ZipTrusted<T>
    {
        let mut iter = ZipTrusted {
            length: 0,
            t: t,
        };
        iter.set_length();
        iter
    }
}

macro_rules! impl_zip_trusted {
    ($($B:ident),*) => (
        #[allow(non_snake_case)]
        impl<$($B),*> SetLength for ZipTrusted<($($B,)*)>
            where
            $(
                $B: TrustedIterator,
            )*
        {
            #[inline]
            fn set_length(&mut self)
            {
                let len = ::std::usize::MAX;
                let ($(ref $B,)*) = self.t;
                $(
                    let (l, h) = $B.size_hint();
                    let len = cmp::min(len, l);
                    debug_assert!(Some(l) == h);
                )*
                self.length = len;
            }
        }

        #[allow(non_snake_case)]
        impl<$($B),*> Iterator for ZipTrusted<($($B,)*)>
            where
            $(
                $B: TrustedIterator,
            )*
        {
            type Item = ($($B::Item,)*);

            fn next(&mut self) -> Option<<Self as Iterator>::Item>
            {
                let ($(ref mut $B,)*) = self.t;

                if self.length == 0 {
                    return None
                }
                $(
                    let next_opt = $B.next();
                    let $B;
                    unsafe {
                        ::std::intrinsics::assume(match next_opt {
                            None => false,
                            Some(_) => true,
                        });
                        $B = match next_opt {
                            None => return None,
                            Some(elt) => elt
                        };
                    }
                )*
                self.length -= 1;
                Some(($($B,)*))
            }

            fn size_hint(&self) -> (usize, Option<usize>)
            {
                (self.length, Some(self.length))
            }
        }
    );
}

impl_zip_trusted!(A);
impl_zip_trusted!(A, B);
impl_zip_trusted!(A, B, C);
impl_zip_trusted!(A, B, C, D);
impl_zip_trusted!(A, B, C, D, E);
impl_zip_trusted!(A, B, C, D, E, F);
impl_zip_trusted!(A, B, C, D, E, F, G);
impl_zip_trusted!(A, B, C, D, E, F, G, H);
impl_zip_trusted!(A, B, C, D, E, F, G, H, I);

