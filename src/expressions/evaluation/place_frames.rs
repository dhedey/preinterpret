#![allow(unused)] // TODO[unused-clearup]
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestedPlaceOwnership {
    LateBound,
    SharedReference,
    MutableReference,
}

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
        let ownership = context.requested_ownership();
        context.handle_node_as_place(frame, inner, ownership)
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
        Ok(context.return_any(item.expect_any_place()))
    }
}

pub(super) struct PlaceIndexer {
    access: IndexAccess,
    state: PlaceIndexerPath,
}

enum PlaceIndexerPath {
    PlacePath { index: ExpressionNodeId },
    IndexPath { place: Place },
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
        let ownership = context.requested_ownership();
        context.handle_node_as_place(frame, source, ownership)
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
                let place = item.expect_any_place();
                self.state = PlaceIndexerPath::IndexPath { place };
                // If we do my_obj["my_key"] = 1 then the "my_key" place is created,
                // so mutable reference indexing takes an owned index.
                context.handle_node_as_value(self, index, RequestedValueOwnership::Owned)
            }
            PlaceIndexerPath::IndexPath { place } => {
                let index = item.expect_owned_value();
                match place {
                    Place::MutableReference { mut_ref } => context.return_mutable(
                        mut_ref.resolve_indexed_with_autocreate(self.access, index)?,
                    ),
                    Place::SharedReference {
                        shared_ref,
                        reason_not_mutable,
                    } => context.return_shared(
                        shared_ref.resolve_indexed(self.access, &index)?,
                        reason_not_mutable,
                    ),
                }
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
        let requested_ownership = context.requested_ownership();
        context.handle_node_as_place(frame, source, requested_ownership)
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
        let place = item.expect_any_place();
        Ok(match place {
            Place::MutableReference { mut_ref } => {
                context.return_mutable(mut_ref.resolve_property(self.access)?)
            }
            Place::SharedReference {
                shared_ref,
                reason_not_mutable,
            } => context.return_shared(
                shared_ref.resolve_property(self.access)?,
                reason_not_mutable,
            ),
        })
    }
}
