use super::*;

define_leaf_type! {
    pub(crate) NoneType => AnyType(AnyValueContent::None),
    content: (),
    kind: pub(crate) NoneKind,
    type_name: "none",
    articled_value_name: "a none",
    dyn_impls: {},
}

pub(crate) const fn none() -> AnyValue {
    AnyValue::None(())
}

impl ResolvableArgumentTarget for () {
    type ValueType = NoneType;
}

impl ResolvableOwned<AnyValue> for () {
    fn resolve_from_value(value: AnyValue, context: ResolutionContext) -> FunctionResult<Self> {
        match value {
            AnyValue::None(_) => Ok(()),
            other => context.err("None", other),
        }
    }
}

/// Returns a reference to a `None` value with the `'static` lifetime.
/// MSRV: On Rust 1.68, `&AnyValue::None(())` can't be promoted to `'static` directly.
#[allow(clippy::missing_const_for_thread_local)]
pub(crate) fn static_none_ref() -> &'static AnyValue {
    use std::cell::Cell;
    thread_local! {
        static NONE: Cell<Option<&'static AnyValue>> = Cell::new(None);
    }
    NONE.with(|cell| match cell.get() {
        Some(val) => val,
        None => {
            let val: &'static AnyValue = Box::leak(Box::new(AnyValue::None(())));
            cell.set(Some(val));
            val
        }
    })
}

define_type_features! {
    impl NoneType,
    pub(crate) mod none_interface {}
}
