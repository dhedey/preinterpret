//! A preinterpret assignee frame is just used for the target of an assignment.
//!
//! They're similar to mutable references, but behave slightly differently:
//! * They can create entries in objects, e.g. `x["new_key"] = value`
//!
//! Realistically, these could just be moved to be value frames, by:
//! * Adding ResolvedValue::Assignee(MutableValue)
//! * Merging these assignee frames with the value frames,
//!   and distinguishing whether to use "auto_create" or not based on whether the
//!   request is mutable or assignee.
#![allow(unused)] // TODO[unused-clearup]
use super::*;

/// Handlers which return an Assignee
pub(super) enum AnyAssigneeFrame {
    Grouped(GroupedAssignee),
    Indexed(IndexedAssignee),
    PropertyAccessed(PropertyAccessedAssignee),
    ValueBased(ValueBasedAssignee),
}

impl AnyAssigneeFrame {
    pub(super) fn handle_item(
        self,
        context: Context<AssigneeType>,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        match self {
            Self::Grouped(frame) => frame.handle_item(context, item),
            Self::Indexed(frame) => frame.handle_item(context, item),
            Self::PropertyAccessed(frame) => frame.handle_item(context, item),
            Self::ValueBased(frame) => frame.handle_item(context, item),
        }
    }
}

struct PrivateUnit;

pub(super) struct GroupedAssignee(PrivateUnit);

impl GroupedAssignee {
    pub(super) fn start(context: AssigneeContext, inner: ExpressionNodeId) -> NextAction {
        let frame = Self(PrivateUnit);
        context.handle_node_as_assignee(frame, inner)
    }
}

impl EvaluationFrame for GroupedAssignee {
    type ReturnType = AssigneeType;

    fn into_any(self) -> AnyAssigneeFrame {
        AnyAssigneeFrame::Grouped(self)
    }

    fn handle_item(
        self,
        context: AssigneeContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(context.return_assignee(item.expect_assignee()))
    }
}

pub(super) struct IndexedAssignee {
    access: IndexAccess,
    state: IndexedAssigneePath,
}

enum IndexedAssigneePath {
    IndexedPath { index: ExpressionNodeId },
    IndexPath { place: MutableValue },
}

impl IndexedAssignee {
    pub(super) fn start(
        context: AssigneeContext,
        source: ExpressionNodeId,
        access: IndexAccess,
        index: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            access,
            state: IndexedAssigneePath::IndexedPath { index },
        };
        context.handle_node_as_assignee(frame, source)
    }
}

impl EvaluationFrame for IndexedAssignee {
    type ReturnType = AssigneeType;

    fn into_any(self) -> AnyAssigneeFrame {
        AnyAssigneeFrame::Indexed(self)
    }

    fn handle_item(
        mut self,
        context: AssigneeContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            IndexedAssigneePath::IndexedPath { index } => {
                let place = item.expect_assignee();
                self.state = IndexedAssigneePath::IndexPath { place };
                // If we do my_obj["my_key"] = 1 then the "my_key" place is created,
                // so mutable reference indexing takes an owned index...
                // But we auto-clone the key in that case, so we can still pass a shared ref
                context.handle_node_as_shared(self, index)
            }
            IndexedAssigneePath::IndexPath { place } => {
                let index = item.expect_shared();
                let output = place.resolve_indexed(self.access, index.as_spanned(), true)?;
                context.return_assignee(output)
            }
        })
    }
}

pub(super) struct PropertyAccessedAssignee {
    access: PropertyAccess,
}

impl PropertyAccessedAssignee {
    pub(super) fn start(
        context: AssigneeContext,
        source: ExpressionNodeId,
        access: PropertyAccess,
    ) -> NextAction {
        let frame = Self { access };
        context.handle_node_as_assignee(frame, source)
    }
}

impl EvaluationFrame for PropertyAccessedAssignee {
    type ReturnType = AssigneeType;

    fn into_any(self) -> AnyAssigneeFrame {
        AnyAssigneeFrame::PropertyAccessed(self)
    }

    fn handle_item(
        self,
        context: AssigneeContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        let place = item.expect_assignee();
        let output = place.resolve_property(&self.access, true)?;
        Ok(context.return_assignee(output))
    }
}

pub(super) struct ValueBasedAssignee(PrivateUnit);

impl ValueBasedAssignee {
    pub(super) fn start(context: AssigneeContext, inner: ExpressionNodeId) -> NextAction {
        let frame = Self(PrivateUnit);
        context.handle_node_as_assignee_value(frame, inner)
    }
}

impl EvaluationFrame for ValueBasedAssignee {
    type ReturnType = AssigneeType;

    fn into_any(self) -> AnyAssigneeFrame {
        AnyAssigneeFrame::ValueBased(self)
    }

    fn handle_item(
        self,
        context: AssigneeContext,
        item: EvaluationItem,
    ) -> ExecutionResult<NextAction> {
        Ok(context.return_assignee(item.expect_assignee_value()))
    }
}
