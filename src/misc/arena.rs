use std::fmt::Debug;

use super::*;

macro_rules! new_key {
    ($vis:vis $key:ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash)]
        $vis struct $key(Key<$key>);

        impl ArenaKey for $key {
            fn from_inner(value: Key<$key>) -> Self {
                Self(value)
            }

            fn to_inner(self) -> Key<Self> {
                self.0
            }

            fn as_inner(&self) -> &Key<Self> {
                &self.0
            }
        }

        impl std::fmt::Debug for $key {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({:?})", stringify!($key), self.0)
            }
        }
    }
}

pub(crate) use new_key;

pub(crate) struct Arena<K: ArenaKey, Data> {
    data: Vec<Data>,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey, D> Arena<K, D> {
    pub(crate) fn new() -> Self {
        Self {
            data: Vec::new(),
            instance_marker: PhantomData,
        }
    }

    pub(crate) fn add(&mut self, value: D) -> K {
        let index = self.data.len();
        self.data.push(value);
        K::from_inner(Key::new(index))
    }

    pub(crate) fn get(&self, key: K) -> &D {
        &self.data[key.to_inner().index]
    }

    pub(crate) fn get_mut(&mut self, key: K) -> &mut D {
        &mut self.data[key.to_inner().index]
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (K, &D)> {
        self.data
            .iter()
            .enumerate()
            .map(|(index, v)| (K::from_inner(Key::new(index)), v))
    }

    pub(crate) fn map_all<D2>(self, f: impl Fn(D) -> D2) -> Arena<K, D2> {
        Arena {
            data: self.data.into_iter().map(f).collect(),
            instance_marker: PhantomData,
        }
    }
}

impl<K: ArenaKey + Debug, D: Debug> Debug for Arena<K, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

pub(crate) trait ArenaKey: Sized {
    fn from_inner(value: Key<Self>) -> Self;
    fn to_inner(self) -> Key<Self>;
    fn as_inner(&self) -> &Key<Self>;
    fn new_placeholder() -> Self {
        Self::from_inner(Key::new(usize::MAX))
    }
    fn is_placeholder(&self) -> bool {
        self.as_inner().index == usize::MAX
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Key<K: ArenaKey> {
    index: usize,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey> std::fmt::Debug for Key<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<K: ArenaKey> Key<K> {
    fn new(index: usize) -> Self {
        Self {
            index,
            instance_marker: PhantomData,
        }
    }
}
