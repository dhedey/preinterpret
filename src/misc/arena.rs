use super::*;

macro_rules! new_key {
    ($vis:vis $key:ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        $vis struct $key(Key<$key>);

        impl ArenaKey for $key {
            fn from_inner(value: Key<$key>) -> Self {
                Self(value)
            }

            fn to_inner(self) -> Key<Self> {
                self.0
            }
        }
    }
}

pub(crate) use new_key;

pub(crate) struct AppendOnlyArena<K: ArenaKey, Data> {
    data: Vec<Data>,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey, D> AppendOnlyArena<K, D> {
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

    #[allow(unused)]
    pub(crate) fn get(&self, key: K) -> &D {
        &self.data[key.to_inner().index]
    }

    pub(crate) fn get_mut(&mut self, key: K) -> &mut D {
        &mut self.data[key.to_inner().index]
    }

    pub(crate) fn into_read_only(self) -> ReadOnlyArena<K, D> {
        ReadOnlyArena {
            data: self.data.into(),
            instance_marker: PhantomData,
        }
    }
}

/// A cheaply clonable read-only arena.
pub(crate) struct ReadOnlyArena<K: ArenaKey, D> {
    data: Rc<[D]>,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey, D> Clone for ReadOnlyArena<K, D> {
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
            instance_marker: PhantomData,
        }
    }
}

impl<K: ArenaKey, D> ReadOnlyArena<K, D> {
    pub(crate) fn get(&self, key: K) -> &D {
        &self.data[key.to_inner().index]
    }
}

pub(crate) trait ArenaKey: Sized {
    fn from_inner(value: Key<Self>) -> Self;
    fn to_inner(self) -> Key<Self>;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Key<K: ArenaKey> {
    index: usize,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey> Key<K> {
    fn new(index: usize) -> Self {
        Self {
            index,
            instance_marker: PhantomData,
        }
    }
}
