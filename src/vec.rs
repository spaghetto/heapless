use core::{fmt, ops, ptr, slice};

use generic_array::{ArrayLength, GenericArray};

use __core::mem::{self, ManuallyDrop};

/// A fixed capacity [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html)
pub struct Vec<T, N>
where
    N: ArrayLength<T>,
{
    buffer: ManuallyDrop<GenericArray<T, N>>,
    len: usize,
}

impl<T, N> Vec<T, N>
where
    N: ArrayLength<T>,
{
    /// Constructs a new, empty vector with a fixed capacity of `N`
    pub const fn new() -> Self {
        Vec {
            buffer: ManuallyDrop::new(unsafe { mem::uninitialized() }),
            len: 0,
        }
    }

    /// Returns the maximum number of elements the vector can hold
    pub fn capacity(&self) -> usize {
        self.buffer.as_slice().len()
    }

    /// Clears the vector, removing all values.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Clones and appends all elements in a slice to the `Vec`.
    ///
    /// Iterates over the slice `other`, clones each element, and then appends
    /// it to this `Vec`. The `other` vector is traversed in-order.
    ///
    /// # Examples
    ///
    /// ```
    /// use heapless::Vec;
    /// use heapless::consts::*;
    ///
    /// let mut vec = Vec::<u8, U8>::new();
    /// vec.push(1).unwrap();
    /// vec.extend_from_slice(&[2, 3, 4]).unwrap();
    /// assert_eq!(*vec, [1, 2, 3, 4]);
    /// ```
    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
    where
        T: Clone,
    {
        if self.len() + other.len() > self.capacity() {
            // won't fit in the `Vec`; don't modify anything and return an error
            Err(())
        } else {
            for elem in other {
                self.push(elem.clone()).ok();
            }
            Ok(())
        }
    }

    /// Removes the last element from a vector and return it, or `None` if it's empty
    pub fn pop(&mut self) -> Option<T> {
        let buffer = self.buffer.as_slice();

        if self.len != 0 {
            self.len -= 1;
            let item = unsafe { ptr::read(&buffer[self.len]) };
            Some(item)
        } else {
            None
        }
    }

    /// Appends an `item` to the back of the collection
    ///
    /// Returns back the `item` if the vector is full
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let capacity = self.capacity();
        let buffer = self.buffer.as_mut_slice();

        if self.len < capacity {
            // NOTE(ptr::write) the memory slot that we are about to write to is uninitialized. We
            // use `ptr::write` to avoid running `T`'s destructor on the uninitialized memory
            unsafe { ptr::write(&mut buffer[self.len], item) }
            self.len += 1;
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            // drop any extra elements
            while len < self.len {
                // decrement len before the drop_in_place(), so a panic on Drop
                // doesn't re-drop the just-failed value.
                self.len -= 1;
                let len = self.len;
                ptr::drop_in_place(self.get_unchecked_mut(len));
            }
        }
    }

    /// Resizes the Vec in-place so that len is equal to new_len.
    ///
    /// If new_len is greater than len, the Vec is extended by the
    /// difference, with each additional slot filled with value. If
    /// new_len is less than len, the Vec is simply truncated.
    ///
    /// See also [`resize_default`](struct.Vec.html#method.resize_default).
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), ()>
    where
        T: Clone,
    {
        if new_len > self.capacity() {
            return Err(());
        }

        if new_len > self.len {
            while self.len < new_len {
                self.push(value.clone()).ok();
            }
        } else {
            self.truncate(new_len);
        }

        Ok(())
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `Default::default()`.
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// See also [`resize`](struct.Vec.html#method.resize).
    pub fn resize_default(&mut self, new_len: usize) -> Result<(), ()>
    where
        T: Clone + Default,
    {
        self.resize(new_len, T::default())
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use heapless::Vec;
    /// use heapless::consts::*;
    ///
    /// let mut v: Vec<_, U8> = Vec::new();
    /// v.push("foo").unwrap();
    /// v.push("bar").unwrap();
    /// v.push("baz").unwrap();
    /// v.push("qux").unwrap();
    ///
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(&*v, ["foo", "qux", "baz"]);
    ///
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(&*v, ["baz", "qux"]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        let length = self.len();
        self.swap(index, length - 1);
        self.pop().unwrap()
    }
}

impl<T, N> fmt::Debug for Vec<T, N>
where
    T: fmt::Debug,
    N: ArrayLength<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice: &[T] = &**self;
        slice.fmt(f)
    }
}

impl<T, N> Drop for Vec<T, N>
where
    N: ArrayLength<T>,
{
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(&mut self[..]) }
    }
}

impl<'a, T, N> IntoIterator for &'a Vec<T, N>
where
    N: ArrayLength<T>,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, N> IntoIterator for &'a mut Vec<T, N>
where
    N: ArrayLength<T>,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, N1, N2> PartialEq<Vec<T, N2>> for Vec<T, N1>
where
    N1: ArrayLength<T>,
    N2: ArrayLength<T>,
    T: PartialEq,
{
    fn eq(&self, rhs: &Vec<T, N2>) -> bool {
        PartialEq::eq(&**self, &**rhs)
    }
}

impl<T, N> Eq for Vec<T, N>
where
    N: ArrayLength<T>,
    T: Eq,
{
}

impl<T, N> ops::Deref for Vec<T, N>
where
    N: ArrayLength<T>,
{
    type Target = [T];

    fn deref(&self) -> &[T] {
        let buffer = self.buffer.as_slice();
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &buffer[..self.len]
        unsafe { slice::from_raw_parts(buffer.as_ptr(), self.len) }
    }
}

impl<T, N> ops::DerefMut for Vec<T, N>
where
    N: ArrayLength<T>,
{
    fn deref_mut(&mut self) -> &mut [T] {
        let len = self.len();
        let buffer = self.buffer.as_mut_slice();

        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &mut buffer[..len]
        unsafe { slice::from_raw_parts_mut(buffer.as_mut_ptr(), len) }
    }
}

impl<T, N> AsRef<Vec<T, N>> for Vec<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, N> AsMut<Vec<T, N>> for Vec<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, N> AsRef<[T]> for Vec<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, N> AsMut<[T]> for Vec<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

#[cfg(test)]
mod tests {
    use consts::*;
    use Vec;

    #[test]
    fn drop() {
        struct Droppable;
        impl Droppable {
            fn new() -> Self {
                unsafe {
                    COUNT += 1;
                }
                Droppable
            }
        }
        impl Drop for Droppable {
            fn drop(&mut self) {
                unsafe {
                    COUNT -= 1;
                }
            }
        }

        static mut COUNT: i32 = 0;

        {
            let mut v: Vec<Droppable, U2> = Vec::new();
            v.push(Droppable::new()).ok().unwrap();
            v.push(Droppable::new()).ok().unwrap();
            v.pop().unwrap();
        }

        assert_eq!(unsafe { COUNT }, 0);

        {
            let mut v: Vec<Droppable, U2> = Vec::new();
            v.push(Droppable::new()).ok().unwrap();
            v.push(Droppable::new()).ok().unwrap();
        }

        assert_eq!(unsafe { COUNT }, 0);
    }

    #[test]
    fn eq() {
        let mut xs: Vec<i32, U4> = Vec::new();
        let mut ys: Vec<i32, U8> = Vec::new();

        assert_eq!(xs, ys);

        xs.push(1).unwrap();
        ys.push(1).unwrap();

        assert_eq!(xs, ys);
    }

    #[test]
    fn full() {
        let mut v: Vec<i32, U4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        assert!(v.push(4).is_err());
    }

    #[test]
    fn iter() {
        let mut v: Vec<i32, U4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        let mut items = v.iter();

        assert_eq!(items.next(), Some(&0));
        assert_eq!(items.next(), Some(&1));
        assert_eq!(items.next(), Some(&2));
        assert_eq!(items.next(), Some(&3));
        assert_eq!(items.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut v: Vec<i32, U4> = Vec::new();

        v.push(0).unwrap();
        v.push(1).unwrap();
        v.push(2).unwrap();
        v.push(3).unwrap();

        let mut items = v.iter_mut();

        assert_eq!(items.next(), Some(&mut 0));
        assert_eq!(items.next(), Some(&mut 1));
        assert_eq!(items.next(), Some(&mut 2));
        assert_eq!(items.next(), Some(&mut 3));
        assert_eq!(items.next(), None);
    }

    #[test]
    fn push_and_pop() {
        let mut v: Vec<i32, U4> = Vec::new();
        assert_eq!(v.len(), 0);

        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);

        v.push(0).unwrap();
        assert_eq!(v.len(), 1);

        assert_eq!(v.pop(), Some(0));
        assert_eq!(v.len(), 0);

        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn resize_size_limit() {
        let mut v: Vec<u8, U4> = Vec::new();

        v.resize(0, 0).unwrap();
        v.resize(4, 0).unwrap();
        v.resize(5, 0).err().expect("full");
    }

    #[test]
    fn resize_length_cases() {
        let mut v: Vec<u8, U4> = Vec::new();

        assert_eq!(v.len(), 0);

        // Grow by 1
        v.resize(1, 0).unwrap();
        assert_eq!(v.len(), 1);

        // Grow by 2
        v.resize(3, 0).unwrap();
        assert_eq!(v.len(), 3);

        // Resize to current size
        v.resize(3, 0).unwrap();
        assert_eq!(v.len(), 3);

        // Shrink by 1
        v.resize(2, 0).unwrap();
        assert_eq!(v.len(), 2);

        // Shrink by 2
        v.resize(0, 0).unwrap();
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn resize_contents() {
        let mut v: Vec<u8, U4> = Vec::new();

        // New entries take supplied value when growing
        v.resize(1, 17).unwrap();
        assert_eq!(v[0], 17);

        // Old values aren't changed when growing
        v.resize(2, 18).unwrap();
        assert_eq!(v[0], 17);
        assert_eq!(v[1], 18);

        // Old values aren't changed when length unchanged
        v.resize(2, 0).unwrap();
        assert_eq!(v[0], 17);
        assert_eq!(v[1], 18);

        // Old values aren't changed when shrinking
        v.resize(1, 0).unwrap();
        assert_eq!(v[0], 17);
    }

    #[test]
    fn resize_default() {
        let mut v: Vec<u8, U4> = Vec::new();

        // resize_default is implemented using resize, so just check the
        // correct value is being written.
        v.resize_default(1).unwrap();
        assert_eq!(v[0], 0);
    }
}
