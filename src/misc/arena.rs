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
        let index = key.to_inner().index;
        match self.data.get(index) {
            Some(value) => value,
            None => panic!("{}", invalid_key_message(index)),
        }
    }

    pub(crate) fn get_mut(&mut self, key: K) -> &mut D {
        let index = key.to_inner().index;
        match self.data.get_mut(index) {
            Some(value) => value,
            None => panic!("{}", invalid_key_message(index)),
        }
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

fn invalid_key_message(key_index: usize) -> &'static str {
    if key_index == PLACEHOLDER_KEY_INDEX {
        "Attempted to access an arena with a placeholder key. The key must be properly initialized before use."
    } else {
        "Arena key does not exist in this arena."
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
        Self::from_inner(Key::new(PLACEHOLDER_KEY_INDEX))
    }
    fn is_placeholder(&self) -> bool {
        self.as_inner().index == PLACEHOLDER_KEY_INDEX
    }
}

const PLACEHOLDER_KEY_INDEX: usize = usize::MAX;

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
