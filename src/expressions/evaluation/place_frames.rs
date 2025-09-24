//! A preinterpret place frame is just used for the target of an assignment.
//! The name is inspired by Rust places, but it is a subtly different concept.
//!
//! They're similar to mutable references, but behave slightly differently:
//! * They can create entries in objects, e.g. `x["new_key"] = value`
//!
//! Realistically, perhaps they should just be moved to be value frames taking
//! mutable references.
#![allow(unused)] // TODO[unused-clearup]
use super::*;

/// Handlers which return a Place
pub(super) enum AnyPlaceFrame {
    Grouped(PlaceGrouper),
    Indexed(PlaceIndexer),
    PropertyAccessed(PlacePropertyAccessor),
}

impl AnyPlaceFrame {
    pub(super) fn handle_item(
        self,
        context: Context<PlaceType>,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            Self::Grouped(frame) => frame.handle_item(context, item),
            Self::Indexed(frame) => frame.handle_item(context, item),
            Self::PropertyAccessed(frame) => frame.handle_item(context, item),
        }
    }
}

struct PrivateUnit;

pub(super) struct PlaceGrouper(PrivateUnit);

impl PlaceGrouper {
    pub(super) fn start(context: PlaceContext, inner: ExpressionNodeId) -> NextAction {
        let frame = Self(PrivateUnit);
        context.handle_node_as_place(frame, inner)
    }
}

impl EvaluationFrame for PlaceGrouper {
    type ReturnType = PlaceType;

    fn into_any(self) -> AnyPlaceFrame {
        AnyPlaceFrame::Grouped(self)
    }

    fn handle_item(
        self,
        context: PlaceContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(context.return_place(item.expect_place()))
    }
}

pub(super) struct PlaceIndexer {
    access: IndexAccess,
    state: PlaceIndexerPath,
}

enum PlaceIndexerPath {
    PlacePath { index: ExpressionNodeId },
    IndexPath { place: MutableValue },
}

impl PlaceIndexer {
    pub(super) fn start(
        context: PlaceContext,
        source: ExpressionNodeId,
        access: IndexAccess,
        index: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            access,
            state: PlaceIndexerPath::PlacePath { index },
        };
        context.handle_node_as_place(frame, source)
    }
}

impl EvaluationFrame for PlaceIndexer {
    type ReturnType = PlaceType;

    fn into_any(self) -> AnyPlaceFrame {
        AnyPlaceFrame::Indexed(self)
    }

    fn handle_item(
        mut self,
        context: PlaceContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            PlaceIndexerPath::PlacePath { index } => {
                let place = item.expect_place();
                self.state = PlaceIndexerPath::IndexPath { place };
                // If we do my_obj["my_key"] = 1 then the "my_key" place is created,
                // so mutable reference indexing takes an owned index...
                // But we auto-clone the key in that case, so we can still pass a shared ref
                context.handle_node_as_shared(self, index)
            }
            PlaceIndexerPath::IndexPath { place } => {
                let index = item.expect_shared();
                let output = place.resolve_indexed(self.access, &index, true)?;
                context.return_place(output)
            }
        })
    }
}

pub(super) struct PlacePropertyAccessor {
    access: PropertyAccess,
}

impl PlacePropertyAccessor {
    pub(super) fn start(
        context: PlaceContext,
        source: ExpressionNodeId,
        access: PropertyAccess,
    ) -> NextAction {
        let frame = Self { access };
        context.handle_node_as_place(frame, source)
    }
}

impl EvaluationFrame for PlacePropertyAccessor {
    type ReturnType = PlaceType;

    fn into_any(self) -> AnyPlaceFrame {
        AnyPlaceFrame::PropertyAccessed(self)
    }

    fn handle_item(
        self,
        context: PlaceContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let place = item.expect_place();
        let output = place.resolve_property(&self.access, true)?;
        Ok(context.return_place(output))
    }
}
