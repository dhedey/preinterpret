use super::*;

macro_rules! new_key {
    ($vis:vis $key:ident($marker:ident)) => {
        #[derive(Copy, Clone)]
        $vis struct $marker;
        impl ArenaMarker for $marker {}

        $vis type $key = Key<$marker>;
    }
}

pub(crate) use new_key;

pub(crate) struct AppendOnlyArena<K: ArenaKey, Data> {
    data: Vec<Data>,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey, D> AppendOnlyArena<K, D> {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new(), instance_marker: PhantomData, }
    }

    pub(crate) fn add(&mut self, value: D) -> Key<K::Marker> {
        let index = self.data.len();
        self.data.push(value);
        Key::new(index)
    }

    #[allow(unused)]
    pub(crate) fn get(&self, key: Key<K::Marker>) -> &D {
        &self.data[key.index]
    }

    pub(crate) fn into_rc_arena(self) -> RcArena<K, D> {
        RcArena {
            data: self.data.into(),
            instance_marker: PhantomData,
        }
    }
}

/// A cheaply clonable read-only arena.
pub(crate) struct RcArena<K: ArenaKey, D> {
    data: Rc<[D]>,
    instance_marker: PhantomData<K>,
}

impl<K: ArenaKey, D> Clone for RcArena<K, D> {
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
            instance_marker: PhantomData,
        }
    }
}

impl<K: ArenaKey, D> RcArena<K, D> {
    pub(crate) fn get(&self, key: Key<K::Marker>) -> &D {
        &self.data[key.index]
    }
}

pub(crate) trait ArenaMarker: Copy + Clone {}

pub(crate) trait ArenaKey: private::Sealed {
    type Marker: ArenaMarker;
}

mod private {
    pub(crate) trait Sealed {}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Key<M: ArenaMarker> {
    index: usize,
    instance_marker: PhantomData<M>,
}

impl<M: ArenaMarker> private::Sealed for Key<M> {}
impl<M: ArenaMarker> ArenaKey for Key<M> {
    type Marker = M;
}

impl<M: ArenaMarker> Key<M> {
    fn new(index: usize) -> Self {
        Self { index, instance_marker: PhantomData, }
    }
}