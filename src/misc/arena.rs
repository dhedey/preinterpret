use super::*;

macro_rules! new_marker {
    ($vis:vis $name:ident) => {
        #[derive(Copy, Clone)]
        $vis struct $name;

        impl ArenaMarker for $name {}
    }
}

pub(crate) use new_marker;

pub(crate) struct WriteOnlyArena<M: ArenaMarker, Data> {
    data: Vec<Data>,
    instance_marker: PhantomData<M>,
}

impl<M: ArenaMarker, D> WriteOnlyArena<M, D> {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new(), instance_marker: PhantomData, }
    }

    pub(crate) fn insert(&mut self, value: D) -> Key<M> {
        let index = self.data.len();
        self.data.push(value);
        Key::new(index)
    }

    pub(crate) fn get(&self, key: Key<M>) -> &D {
        &self.data[key.index]
    }
}

pub(crate) trait ArenaMarker: Copy + Clone {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Key<M: ArenaMarker> {
    index: usize,
    instance_marker: PhantomData<M>,
}

impl<M: ArenaMarker> Key<M> {
    fn new(index: usize) -> Self {
        Self { index, instance_marker: PhantomData, }
    }
}