use super::*;

/// A [`ArgumentValue`] represents a value which has had its ownership concretely
/// resolved for use in a specific operation (e.g. method call, property access, etc).
///
/// It is typically paired with an [`ArgumentOwnership`] (maybe wrapped in a [`RequestedOwnership`])
/// which indicates what ownership type to resolve to.
pub(crate) enum ArgumentValue {
    Owned(OwnedValue),
    CopyOnWrite(CopyOnWriteValue),
    Mutable(MutableValue),
    Assignee(AssigneeValue),
    Shared(SharedValue),
}

impl ArgumentValue {
    pub(crate) fn expect_owned(self) -> OwnedValue {
        match self {
            ArgumentValue::Owned(value) => value,
            _ => panic!("expect_owned() called on a non-owned ArgumentValue"),
        }
    }

    pub(crate) fn expect_copy_on_write(self) -> CopyOnWriteValue {
        match self {
            ArgumentValue::CopyOnWrite(value) => value,
            _ => panic!("expect_copy_on_write() called on a non-copy-on-write ArgumentValue"),
        }
    }

    pub(crate) fn expect_mutable(self) -> MutableValue {
        match self {
            ArgumentValue::Mutable(value) => value,
            _ => panic!("expect_mutable() called on a non-mutable ArgumentValue"),
        }
    }

    pub(crate) fn expect_assignee(self) -> AssigneeValue {
        match self {
            ArgumentValue::Assignee(value) => value,
            _ => panic!("expect_assignee() called on a non-assignee ArgumentValue"),
        }
    }

    pub(crate) fn expect_shared(self) -> SharedValue {
        match self {
            ArgumentValue::Shared(value) => value,
            _ => panic!("expect_shared() called on a non-shared ArgumentValue"),
        }
    }
}

impl Spanned<ArgumentValue> {
    #[inline]
    pub(crate) fn expect_owned(self) -> Spanned<OwnedValue> {
        self.map(|value| value.expect_owned())
    }

    #[inline]
    pub(crate) fn expect_mutable(self) -> Spanned<MutableValue> {
        self.map(|value| value.expect_mutable())
    }

    #[inline]
    pub(crate) fn expect_assignee(self) -> Spanned<AssigneeValue> {
        self.map(|value| value.expect_assignee())
    }

    #[inline]
    pub(crate) fn expect_shared(self) -> Spanned<SharedValue> {
        self.map(|value| value.expect_shared())
    }
}

// Note: ArgumentValue no longer implements HasSpanRange or WithSpanRangeExt
// since the inner value types no longer carry spans internally.
// Spans should be tracked separately at a higher level if needed.

impl Spanned<&mut ArgumentValue> {
    /// SAFETY:
    /// * Must be paired with a call to `enable()` before any further use of the value.
    /// * Must not use the value while disabled.
    pub(crate) unsafe fn disable(&mut self) {
        match &mut self.0 {
            ArgumentValue::Owned(_) => {}
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write.spanned(self.1).disable(),
            ArgumentValue::Mutable(mutable) => mutable.spanned(self.1).disable(),
            ArgumentValue::Assignee(assignee) => (&mut assignee.0).spanned(self.1).disable(),
            ArgumentValue::Shared(shared) => shared.spanned(self.1).disable(),
        }
    }

    /// SAFETY:
    /// * Must only be used after a call to `disable()`.
    ///
    /// Returns an ownership error if re-enabling fails (e.g., due to conflicting borrows).
    pub(crate) unsafe fn enable(&mut self) -> ExecutionResult<()> {
        match &mut self.0 {
            ArgumentValue::Owned(_) => Ok(()),
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write.spanned(self.1).enable(),
            ArgumentValue::Mutable(mutable) => mutable.spanned(self.1).enable(),
            ArgumentValue::Assignee(assignee) => (&mut assignee.0).spanned(self.1).enable(),
            ArgumentValue::Shared(shared) => shared.spanned(self.1).enable(),
        }
    }
}

impl Deref for ArgumentValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsMut<Self> for ArgumentValue {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl AsRef<Value> for ArgumentValue {
    fn as_ref(&self) -> &Value {
        match self {
            ArgumentValue::Owned(owned) => owned,
            ArgumentValue::Mutable(mutable) => mutable,
            ArgumentValue::Assignee(assignee) => &assignee.0,
            ArgumentValue::Shared(shared) => shared,
            ArgumentValue::CopyOnWrite(copy_on_write) => copy_on_write,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestedOwnership {
    /// Receives any of Owned, SharedReference or MutableReference, depending on what
    /// is available.
    /// This can then be used to resolve the value kind, and use the correct one.
    LateBound,
    /// A concrete value of the correct type.
    Concrete(ArgumentOwnership),
}

impl RequestedOwnership {
    pub(crate) fn owned() -> Self {
        RequestedOwnership::Concrete(ArgumentOwnership::Owned)
    }

    pub(crate) fn shared() -> Self {
        RequestedOwnership::Concrete(ArgumentOwnership::Shared)
    }

    pub(crate) fn replace_owned_with_copy_on_write(self) -> Self {
        match self {
            RequestedOwnership::Concrete(ArgumentOwnership::Owned) => {
                RequestedOwnership::Concrete(ArgumentOwnership::CopyOnWrite)
            }
            _ => self,
        }
    }

    pub(crate) fn requests_auto_create(&self) -> bool {
        match self {
            RequestedOwnership::Concrete(ArgumentOwnership::Assignee { auto_create }) => {
                *auto_create
            }
            _ => false,
        }
    }

    pub(crate) fn map_none(self, span: SpanRange) -> ExecutionResult<Spanned<RequestedValue>> {
        self.map_from_owned(Spanned(().into_owned_value(), span))
    }

    pub(crate) fn map_from_late_bound(
        &self,
        Spanned(late_bound, span): Spanned<LateBoundValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => RequestedValue::LateBound(late_bound),
                RequestedOwnership::Concrete(_) => {
                    panic!("Returning a late-bound reference when concrete ownership was requested")
                }
            },
            span,
        ))
    }

    pub(crate) fn map_from_argument(
        &self,
        Spanned(value, span): Spanned<ArgumentValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        match value {
            ArgumentValue::Owned(owned) => self.map_from_owned(Spanned(owned, span)),
            ArgumentValue::Mutable(mutable) => self.map_from_mutable(Spanned(mutable, span)),
            ArgumentValue::Assignee(assignee) => self.map_from_assignee(Spanned(assignee, span)),
            ArgumentValue::Shared(shared) => self.map_from_shared(Spanned(shared, span)),
            ArgumentValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(Spanned(copy_on_write, span))
            }
        }
    }

    pub(crate) fn map_from_returned(
        &self,
        Spanned(value, span): Spanned<ReturnedValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        match value {
            ReturnedValue::Owned(owned) => self.map_from_owned(Spanned(owned, span)),
            ReturnedValue::Mutable(mutable) => self.map_from_mutable(Spanned(mutable, span)),
            ReturnedValue::Shared(shared) => self.map_from_shared(Spanned(shared, span)),
            ReturnedValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(Spanned(copy_on_write, span))
            }
        }
    }

    /// This ensures the requested value's type aligns with the requested ownership.
    pub(crate) fn map_from_requested(
        &self,
        Spanned(requested, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        match requested {
            RequestedValue::Owned(owned) => self.map_from_owned(Spanned(owned, span)),
            RequestedValue::Shared(shared) => self.map_from_shared(Spanned(shared, span)),
            RequestedValue::Mutable(mutable) => self.map_from_mutable(Spanned(mutable, span)),
            RequestedValue::Assignee(assignee) => self.map_from_assignee(Spanned(assignee, span)),
            RequestedValue::LateBound(late_bound_value) => {
                self.map_from_late_bound(Spanned(late_bound_value, span))
            }
            RequestedValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(Spanned(copy_on_write, span))
            }
            RequestedValue::AssignmentCompletion { .. } => {
                panic!("Returning a non-value item from a value context")
            }
        }
    }

    pub(crate) fn map_from_owned(
        &self,
        Spanned(value, span): Spanned<OwnedValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => {
                    RequestedValue::LateBound(LateBoundValue::Owned(LateBoundOwnedValue {
                        owned: value,
                        is_from_last_use: false,
                    }))
                }
                RequestedOwnership::Concrete(requested) => {
                    Self::item_from_argument(requested.map_from_owned(Spanned(value, span))?)
                }
            },
            span,
        ))
    }

    pub(crate) fn map_from_copy_on_write(
        &self,
        Spanned(cow, span): Spanned<CopyOnWriteValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => {
                    RequestedValue::LateBound(LateBoundValue::CopyOnWrite(cow))
                }
                RequestedOwnership::Concrete(requested) => {
                    Self::item_from_argument(requested.map_from_copy_on_write(Spanned(cow, span))?)
                }
            },
            span,
        ))
    }

    pub(crate) fn map_from_mutable(
        &self,
        Spanned(mutable, span): Spanned<MutableValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => {
                    RequestedValue::LateBound(LateBoundValue::Mutable(mutable))
                }
                RequestedOwnership::Concrete(requested) => {
                    Self::item_from_argument(requested.map_from_mutable(Spanned(mutable, span))?)
                }
            },
            span,
        ))
    }

    pub(crate) fn map_from_assignee(
        &self,
        Spanned(assignee, span): Spanned<AssigneeValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => {
                    RequestedValue::LateBound(LateBoundValue::Mutable(assignee.0))
                }
                RequestedOwnership::Concrete(requested) => {
                    Self::item_from_argument(requested.map_from_assignee(Spanned(assignee, span))?)
                }
            },
            span,
        ))
    }

    pub(crate) fn map_from_shared(
        &self,
        Spanned(shared, span): Spanned<SharedValue>,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        Ok(Spanned(
            match self {
                RequestedOwnership::LateBound => RequestedValue::LateBound(
                    LateBoundValue::CopyOnWrite(CopyOnWrite::shared_in_place_of_shared(shared)),
                ),
                RequestedOwnership::Concrete(requested) => {
                    Self::item_from_argument(requested.map_from_shared(Spanned(shared, span))?)
                }
            },
            span,
        ))
    }

    fn item_from_argument(value: ArgumentValue) -> RequestedValue {
        match value {
            ArgumentValue::Owned(owned) => RequestedValue::Owned(owned),
            ArgumentValue::Mutable(mutable) => RequestedValue::Mutable(mutable),
            ArgumentValue::Assignee(assignee) => RequestedValue::Assignee(assignee),
            ArgumentValue::Shared(shared) => RequestedValue::Shared(shared),
            ArgumentValue::CopyOnWrite(copy_on_write) => RequestedValue::CopyOnWrite(copy_on_write),
        }
    }
}

impl From<ArgumentOwnership> for RequestedOwnership {
    fn from(ownership: ArgumentOwnership) -> Self {
        RequestedOwnership::Concrete(ownership)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The ownership that a method might concretely request
pub(crate) enum ArgumentOwnership {
    Owned,
    Shared,
    Mutable,
    /// Approximately equivalent to Mutable, but explicitly for use as an assignee.
    /// (e.g. `x[0] = 1` or `obj.field = 2` or `x.swap(y)`)
    /// In rust, an assignee is a special type of place, and if you want to assign
    /// to a mutable reference, you have to explicitly dereference it with *.
    /// (Under Niko's Overwrite proposal, this would require that the value is also
    /// Overwrite).
    ///
    /// The distinction from mutable allows for different handling in ownership conversion.
    /// For example, whilst an owned value can be freely converted to a mutable reference
    /// for use in a method, it cannot be freely converted to an assignee.
    /// This prevents (1 = 2) = 3 style issues, where it would be insane to allow assignment
    /// to a floating owned value.
    ///
    /// The auto_create flag indicates whether the assignee should create missing entries,
    /// for example, it enables `obj.new_field = value` to work by auto-creating `new_field` in `obj`.
    Assignee {
        auto_create: bool,
    },
    /// Approximately equivalent to Owned, but more flexible to avoid cloning large values unnecessarily
    /// e.g. array indexing operations should take CopyOnWrite instead of Owned
    /// If a method needs to create an owned value, that method can transparently or infallibly copy it,
    /// as per the method's own requirements/expectations.
    CopyOnWrite,
    /// A niche resolved value which passes through the value uncoerced, for
    /// handling in the method itself - notably this is used in the `.as_mut()` method.
    ///
    /// Currently this is handled as a ArgumentValue, but it might be better to handle
    /// it as LateBound (so that we don't drop the shared-conversion error reason).
    AsIs,
}

impl ArgumentOwnership {
    pub(crate) fn map_from_late_bound(
        &self,
        Spanned(late_bound, span): Spanned<LateBoundValue>,
    ) -> ExecutionResult<ArgumentValue> {
        match late_bound {
            LateBoundValue::Owned(owned) => self.map_from_owned_with_is_last_use(
                Spanned(owned.owned, span),
                owned.is_from_last_use,
            ),
            LateBoundValue::CopyOnWrite(copy_on_write) => {
                self.map_from_copy_on_write(Spanned(copy_on_write, span))
            }
            LateBoundValue::Mutable(mutable) => {
                self.map_from_mutable_inner(Spanned(mutable, span), true)
            }
            LateBoundValue::Shared(late_bound_shared) => self
                .map_from_shared_with_error_reason(Spanned(late_bound_shared.shared, span), |_| {
                    ExecutionInterrupt::ownership_error(late_bound_shared.reason_not_mutable)
                }),
        }
    }

    pub(crate) fn map_from_copy_on_write(
        &self,
        Spanned(copy_on_write, span): Spanned<CopyOnWriteValue>,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => Ok(ArgumentValue::Owned(
                copy_on_write.into_owned_transparently(span)?,
            )),
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(copy_on_write.into_shared())),
            ArgumentOwnership::Mutable => {
                if copy_on_write.acts_as_shared_reference() {
                    span.ownership_err("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone()` to get a mutable reference to a cloned value.")
                } else {
                    Ok(ArgumentValue::Mutable(Mutable::new_from_owned(
                        copy_on_write.into_owned_transparently(span)?.into_inner(),
                    )))
                }
            }
            ArgumentOwnership::Assignee { .. } => {
                if copy_on_write.acts_as_shared_reference() {
                    span.ownership_err("A shared reference cannot be assigned to.")
                } else {
                    span.ownership_err("An owned value cannot be assigned to.")
                }
            }
            ArgumentOwnership::CopyOnWrite | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::CopyOnWrite(copy_on_write))
            }
        }
    }

    pub(crate) fn map_from_shared(
        &self,
        shared: Spanned<SharedValue>,
    ) -> ExecutionResult<ArgumentValue> {
        self.map_from_shared_with_error_reason(
            shared,
            |span| span.ownership_error("A mutable reference is required, but a shared reference was received, this indicates a possible bug as the updated value won't be accessible. To proceed regardless, use `.clone().as_mut()` to get a mutable reference."),
        )
    }

    fn map_from_shared_with_error_reason(
        &self,
        Spanned(shared, span): Spanned<SharedValue>,
        mutable_error: impl FnOnce(SpanRange) -> ExecutionInterrupt,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => Ok(ArgumentValue::Owned(
                Spanned(shared, span).transparent_clone()?,
            )),
            ArgumentOwnership::CopyOnWrite => Ok(ArgumentValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(shared),
            )),
            ArgumentOwnership::Assignee { .. } => Err(mutable_error(span)),
            ArgumentOwnership::Mutable => Err(mutable_error(span)),
            ArgumentOwnership::Shared | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::Shared(shared))
            }
        }
    }

    pub(crate) fn map_from_mutable(
        &self,
        spanned_mutable: Spanned<MutableValue>,
    ) -> ExecutionResult<ArgumentValue> {
        self.map_from_mutable_inner(spanned_mutable, false)
    }

    pub(crate) fn map_from_assignee(
        &self,
        Spanned(assignee, span): Spanned<AssigneeValue>,
    ) -> ExecutionResult<ArgumentValue> {
        self.map_from_mutable_inner(Spanned(assignee.0, span), false)
    }

    fn map_from_mutable_inner(
        &self,
        Spanned(mutable, span): Spanned<MutableValue>,
        is_late_bound: bool,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned => {
                if is_late_bound {
                    Ok(ArgumentValue::Owned(
                        Spanned(mutable, span).transparent_clone()?,
                    ))
                } else {
                    span.ownership_err("An owned value is required, but a mutable reference was received. This indicates a possible bug. If this was intended, use `.clone()` to get an owned value.")
                }
            }
            ArgumentOwnership::CopyOnWrite => Ok(ArgumentValue::CopyOnWrite(
                CopyOnWrite::shared_in_place_of_shared(mutable.into_shared()),
            )),
            ArgumentOwnership::Mutable | ArgumentOwnership::AsIs => {
                Ok(ArgumentValue::Mutable(mutable))
            }
            ArgumentOwnership::Assignee { .. } => Ok(ArgumentValue::Assignee(Assignee(mutable))),
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(mutable.into_shared())),
        }
    }

    pub(crate) fn map_from_owned(
        &self,
        owned: Spanned<OwnedValue>,
    ) -> ExecutionResult<ArgumentValue> {
        self.map_from_owned_with_is_last_use(owned, false)
    }

    fn map_from_owned_with_is_last_use(
        &self,
        Spanned(owned, span): Spanned<OwnedValue>,
        is_from_last_use: bool,
    ) -> ExecutionResult<ArgumentValue> {
        match self {
            ArgumentOwnership::Owned | ArgumentOwnership::AsIs => Ok(ArgumentValue::Owned(owned)),
            ArgumentOwnership::CopyOnWrite => {
                Ok(ArgumentValue::CopyOnWrite(CopyOnWrite::owned(owned)))
            }
            ArgumentOwnership::Mutable => Ok(ArgumentValue::Mutable(Mutable::new_from_owned(
                owned.into_inner(),
            ))),
            ArgumentOwnership::Assignee { .. } => {
                if is_from_last_use {
                    span.ownership_err("The final usage of a variable cannot be assigned to. You can use `let _ = ..` to discard a value.")
                } else {
                    span.ownership_err("An owned value cannot be assigned to.")
                }
            }
            ArgumentOwnership::Shared => Ok(ArgumentValue::Shared(Shared::new_from_owned(
                owned.into_inner(),
            ))),
        }
    }
}

/// Handlers which return a Value
pub(super) enum AnyValueFrame {
    Group(GroupBuilder),
    Array(ArrayBuilder),
    Object(Box<ObjectBuilder>),
    UnaryOperation(UnaryOperationBuilder),
    BinaryOperation(BinaryOperationBuilder),
    PropertyAccess(ValuePropertyAccessBuilder),
    IndexAccess(ValueIndexAccessBuilder),
    Range(RangeBuilder),
    Assignment(AssignmentBuilder),
    MethodCall(MethodCallBuilder),
}

impl AnyValueFrame {
    pub(super) fn handle_next(
        self,
        context: Context<ValueType>,
        value: Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        match self {
            AnyValueFrame::Group(frame) => frame.handle_next(context, value),
            AnyValueFrame::Array(frame) => frame.handle_next(context, value),
            AnyValueFrame::Object(frame) => frame.handle_next(context, value),
            AnyValueFrame::UnaryOperation(frame) => frame.handle_next(context, value),
            AnyValueFrame::BinaryOperation(frame) => frame.handle_next(context, value),
            AnyValueFrame::PropertyAccess(frame) => frame.handle_next(context, value),
            AnyValueFrame::IndexAccess(frame) => frame.handle_next(context, value),
            AnyValueFrame::Range(frame) => frame.handle_next(context, value),
            AnyValueFrame::Assignment(frame) => frame.handle_next(context, value),
            AnyValueFrame::MethodCall(frame) => frame.handle_next(context, value),
        }
    }
}

pub(super) struct GroupBuilder {
    span: Span,
}

impl GroupBuilder {
    pub(super) fn start(
        context: ValueContext,
        delim_span: &DelimSpan,
        inner: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            span: delim_span.join(),
        };
        context.request_owned(frame, inner)
    }
}

impl EvaluationFrame for GroupBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Group(self)
    }

    fn handle_next(
        self,
        context: ValueContext,
        Spanned(value, _span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let inner = value.expect_owned();
        // Use the grouped expression's span, not the inner expression's span
        context.return_value(Spanned(inner, self.span.span_range()))
    }
}

pub(super) struct ArrayBuilder {
    span: Span,
    unevaluated_items: Vec<ExpressionNodeId>,
    evaluated_items: Vec<Value>,
}

impl ArrayBuilder {
    pub(super) fn start(
        context: ValueContext,
        brackets: &Brackets,
        items: &[ExpressionNodeId],
    ) -> ExecutionResult<NextAction> {
        Self {
            span: brackets.join(),
            unevaluated_items: items.to_vec(),
            evaluated_items: Vec::with_capacity(items.len()),
        }
        .next(context)
    }

    pub(super) fn next(self, context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_items
                .get(self.evaluated_items.len())
                .cloned()
            {
                Some(next) => context.request_owned(self, next),
                None => {
                    context.return_value(Spanned(self.evaluated_items, self.span.span_range()))?
                }
            },
        )
    }
}

impl EvaluationFrame for ArrayBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Array(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, _span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let value = value.expect_owned();
        self.evaluated_items.push(value.into_inner());
        self.next(context)
    }
}

pub(super) struct ObjectBuilder {
    span: Span,
    pending: Option<PendingEntryPath>,
    unevaluated_entries: Vec<(ObjectKey, ExpressionNodeId)>,
    evaluated_entries: BTreeMap<String, ObjectEntry>,
}

enum PendingEntryPath {
    OnIndexKeyBranch {
        access: IndexAccess,
        value_node: ExpressionNodeId,
    },
    OnValueBranch {
        key: String,
        key_span: Span,
    },
}

impl ObjectBuilder {
    pub(super) fn start(
        context: ValueContext,
        braces: &Braces,
        entries: &[(ObjectKey, ExpressionNodeId)],
    ) -> ExecutionResult<NextAction> {
        Box::new(Self {
            span: braces.join(),
            unevaluated_entries: entries.to_vec(),
            evaluated_entries: BTreeMap::new(),
            pending: None,
        })
        .next(context)
    }

    fn next(mut self: Box<Self>, context: ValueContext) -> ExecutionResult<NextAction> {
        Ok(
            match self
                .unevaluated_entries
                .get(self.evaluated_entries.len())
                .cloned()
            {
                Some((ObjectKey::Identifier(ident), value_node)) => {
                    let key = ident.to_string();
                    if self.evaluated_entries.contains_key(&key) {
                        return ident.syntax_err(format!("The key {} has already been set", key));
                    }
                    self.pending = Some(PendingEntryPath::OnValueBranch {
                        key,
                        key_span: ident.span(),
                    });
                    context.request_owned(self, value_node)
                }
                Some((ObjectKey::Indexed { access, index }, value_node)) => {
                    self.pending = Some(PendingEntryPath::OnIndexKeyBranch { access, value_node });
                    context.request_owned(self, index)
                }
                None => {
                    context.return_value(Spanned(self.evaluated_entries, self.span.span_range()))?
                }
            },
        )
    }
}

impl EvaluationFrame for Box<ObjectBuilder> {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Object(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let pending = self.pending.take();
        Ok(match pending {
            Some(PendingEntryPath::OnIndexKeyBranch { access, value_node }) => {
                let value = value.expect_owned();
                let key: String = Spanned(value, span).resolve_as("An object key")?;
                if self.evaluated_entries.contains_key(&key) {
                    return access.syntax_err(format!("The key {} has already been set", key));
                }
                self.pending = Some(PendingEntryPath::OnValueBranch {
                    key,
                    key_span: access.span(),
                });
                context.request_owned(self, value_node)
            }
            Some(PendingEntryPath::OnValueBranch { key, key_span }) => {
                let value = value.expect_owned().into_inner();
                let entry = ObjectEntry { key_span, value };
                self.evaluated_entries.insert(key, entry);
                self.next(context)?
            }
            None => {
                unreachable!("Should not receive a value without a pending handler set")
            }
        })
    }
}

pub(super) struct UnaryOperationBuilder {
    operation: UnaryOperation,
}

impl UnaryOperationBuilder {
    pub(super) fn start(
        context: ValueContext,
        operation: UnaryOperation,
        input: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self { operation };
        // Use late-bound evaluation to allow method resolution to determine ownership requirements
        context.request_late_bound(frame, input)
    }
}

impl EvaluationFrame for UnaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::UnaryOperation(self)
    }

    fn handle_next(
        self,
        context: ValueContext,
        Spanned(value, operand_span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let operand = value.expect_late_bound();
        let operand_kind = operand.kind();

        // Try method resolution first
        if let Some(interface) = operand_kind.resolve_unary_operation(&self.operation) {
            let resolved_value =
                Spanned(operand, operand_span).resolve(interface.argument_ownership())?;
            let result =
                interface.execute(Spanned(resolved_value, operand_span), &self.operation)?;
            return context.return_returned_value(result);
        }
        self.operation.type_err(format!(
            "The {} operator is not supported for {} values",
            self.operation.symbolic_description(),
            operand.value_type(),
        ))
    }
}

pub(super) struct BinaryOperationBuilder {
    operation: BinaryOperation,
    state: BinaryPath,
}

enum BinaryPath {
    OnLeftBranch {
        right: ExpressionNodeId,
    },
    OnRightBranch {
        left: Spanned<ArgumentValue>,
        interface: BinaryOperationInterface,
    },
}

impl BinaryOperationBuilder {
    pub(super) fn start(
        context: ValueContext,
        operation: BinaryOperation,
        left: ExpressionNodeId,
        right: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            operation,
            state: BinaryPath::OnLeftBranch { right },
        };
        // Use late-bound evaluation to allow method resolution to determine ownership requirements
        context.request_late_bound(frame, left)
    }
}

impl EvaluationFrame for BinaryOperationBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::BinaryOperation(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            BinaryPath::OnLeftBranch { right } => {
                let left_late_bound = value.expect_late_bound();
                let left_span = span;

                // Check for lazy evaluation first (short-circuit operators)
                // Use operator span for type errors since the error is about the operation's requirements
                let left_value = left_late_bound
                    .as_ref()
                    .spanned(self.operation.span_range());
                if let Some(result) = self.operation.lazy_evaluate(left_value)? {
                    // For short-circuit, the result span is just the left operand's span
                    // (the right operand was never evaluated)
                    context
                        .return_returned_value(Spanned(ReturnedValue::Owned(result), left_span))?
                } else {
                    // Try method resolution based on left operand's kind and resolve left operand immediately
                    let interface = left_late_bound
                        .as_ref()
                        .kind()
                        .resolve_binary_operation(&self.operation);

                    match interface {
                        Some(interface) => {
                            let rhs_ownership = interface.rhs_ownership();
                            let left = interface
                                .lhs_ownership()
                                .map_from_late_bound(Spanned(left_late_bound, left_span))?;
                            let mut left = Spanned(left, left_span);

                            unsafe {
                                // SAFETY: We re-enable it below and don't use it while disabled
                                left.to_mut().disable();
                            }

                            self.state = BinaryPath::OnRightBranch { left, interface };
                            context.request_argument_value(self, right, rhs_ownership)
                        }
                        None => {
                            return self.operation.type_err(format!(
                                "The {} operator is not supported for {} operand",
                                self.operation.symbolic_description(),
                                left_late_bound.articled_value_type(),
                            ));
                        }
                    }
                }
            }
            BinaryPath::OnRightBranch {
                mut left,
                interface,
            } => {
                let right = value.expect_argument_value();
                let mut right = Spanned(right, span);

                // NOTE:
                // - This disable/enable flow allows us to do x += x without issues
                // - Read https://rust-lang.github.io/rfcs/2025-nested-method-calls.html for more details
                // - We enable left-to-right for more intuitive error messages:
                //   If left and right clash, then the error message should be on the right, not the left
                unsafe {
                    // SAFETY: We disabled left above
                    right.to_mut().disable();
                    // SAFETY: enable() may fail if left and right reference the same variable
                    left.to_mut().enable()?;
                    right.to_mut().enable()?;
                }
                let result = interface.execute(left, right, &self.operation)?;
                return context.return_returned_value(result);
            }
        })
    }
}

pub(super) struct ValuePropertyAccessBuilder {
    access: PropertyAccess,
}

impl ValuePropertyAccessBuilder {
    pub(super) fn start(
        context: ValueContext,
        access: PropertyAccess,
        node: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self { access };
        // If we need an owned value, we can try resolving a reference and
        // clone the outputted value if needed - which can be much cheaper.
        // e.g. `let x = arr[0]` only copies `arr[0]` instead of the whole array.
        let ownership_request = context
            .requested_ownership()
            .replace_owned_with_copy_on_write();
        context.request_any_value(frame, node, ownership_request)
    }
}

impl EvaluationFrame for ValuePropertyAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::PropertyAccess(self)
    }

    fn handle_next(
        self,
        context: ValueContext,
        Spanned(value, source_span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        let auto_create = context.requested_ownership().requests_auto_create();
        let mapped = value.expect_any_value_and_map(
            |shared| shared.resolve_property(&self.access),
            |mutable| mutable.resolve_property(&self.access, auto_create),
            |owned| owned.resolve_property(&self.access),
        )?;
        // The result span covers source through property
        let result_span = SpanRange::new_between(source_span, self.access.span_range());
        context.return_not_necessarily_matching_requested(Spanned(mapped, result_span))
    }
}

pub(super) struct ValueIndexAccessBuilder {
    access: IndexAccess,
    state: IndexPath,
}

enum IndexPath {
    OnSourceBranch { index: ExpressionNodeId },
    OnIndexBranch { source: Spanned<RequestedValue> },
}

impl ValueIndexAccessBuilder {
    pub(super) fn start(
        context: ValueContext,
        source: ExpressionNodeId,
        access: IndexAccess,
        index: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            access,
            state: IndexPath::OnSourceBranch { index },
        };
        // If we need an owned value, we can try resolving a reference and
        // clone the outputted value if needed - which can be much cheaper.
        // e.g. `let x = obj.xyz` only copies `obj.xyz` instead of the whole object.
        let ownership_request = context
            .requested_ownership()
            .replace_owned_with_copy_on_write();
        context.request_any_value(frame, source, ownership_request)
    }
}

impl EvaluationFrame for ValueIndexAccessBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::IndexAccess(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            IndexPath::OnSourceBranch { index } => {
                self.state = IndexPath::OnIndexBranch {
                    source: Spanned(value, span),
                };
                // This is a value, so we are _accessing it_ and can't create values
                // (that's only possible in a place!) - therefore we don't need an owned key,
                // and can use &index for reading values from our array
                context.request_shared(self, index)
            }
            IndexPath::OnIndexBranch {
                source: Spanned(source, source_span),
            } => {
                let index = value.expect_shared();
                let index = index.as_ref().spanned(span);

                let auto_create = context.requested_ownership().requests_auto_create();
                let result = source.expect_any_value_and_map(
                    |shared| shared.resolve_indexed(self.access, index),
                    |mutable| mutable.resolve_indexed(self.access, index, auto_create),
                    |owned| owned.resolve_indexed(self.access, index),
                )?;
                let result_span = SpanRange::new_between(source_span, self.access.span_range());
                context.return_not_necessarily_matching_requested(Spanned(result, result_span))?
            }
        })
    }
}

pub(super) struct RangeBuilder {
    range_limits: syn::RangeLimits,
    state: RangePath,
}

enum RangePath {
    OnLeftBranch { right: Option<ExpressionNodeId> },
    OnRightBranch { left: Option<Value> },
}

impl RangeBuilder {
    pub(super) fn start(
        context: ValueContext,
        left: &Option<ExpressionNodeId>,
        range_limits: &syn::RangeLimits,
        right: &Option<ExpressionNodeId>,
    ) -> ExecutionResult<NextAction> {
        Ok(match (left, right) {
            (None, None) => match range_limits {
                syn::RangeLimits::HalfOpen(token) => {
                    let inner = RangeValueInner::RangeFull { token: *token };
                    context.return_value(Spanned(inner, token.span_range()))?
                }
                syn::RangeLimits::Closed(_) => {
                    unreachable!(
                        "A closed range should have been given a right in continue_range(..)"
                    )
                }
            },
            (None, Some(right)) => context.request_owned(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnRightBranch { left: None },
                },
                *right,
            ),
            (Some(left), right) => context.request_owned(
                Self {
                    range_limits: *range_limits,
                    state: RangePath::OnLeftBranch { right: *right },
                },
                *left,
            ),
        })
    }
}

impl EvaluationFrame for RangeBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Range(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, _span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        // TODO[range-refactor]: Change to not always clone the value
        let value = value.expect_owned().into_inner();
        Ok(match (self.state, self.range_limits) {
            (RangePath::OnLeftBranch { right: Some(right) }, _) => {
                self.state = RangePath::OnRightBranch { left: Some(value) };
                context.request_owned(self, right)
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::RangeFrom {
                    start_inclusive: value,
                    token,
                };
                context.return_value(Spanned(inner, token.span_range()))?
            }
            (RangePath::OnLeftBranch { right: None }, syn::RangeLimits::Closed(_)) => {
                unreachable!("A closed range should have been given a right in continue_range(..)")
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::Range {
                    start_inclusive: left,
                    token,
                    end_exclusive: value,
                };
                context.return_value(Spanned(inner, token.span_range()))?
            }
            (RangePath::OnRightBranch { left: Some(left) }, syn::RangeLimits::Closed(token)) => {
                let inner = RangeValueInner::RangeInclusive {
                    start_inclusive: left,
                    token,
                    end_inclusive: value,
                };
                context.return_value(Spanned(inner, token.span_range()))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::HalfOpen(token)) => {
                let inner = RangeValueInner::RangeTo {
                    token,
                    end_exclusive: value,
                };
                context.return_value(Spanned(inner, token.span_range()))?
            }
            (RangePath::OnRightBranch { left: None }, syn::RangeLimits::Closed(token)) => {
                let inner = RangeValueInner::RangeToInclusive {
                    token,
                    end_inclusive: value,
                };
                context.return_value(Spanned(inner, token.span_range()))?
            }
        })
    }
}

pub(super) struct AssignmentBuilder {
    #[allow(unused)]
    equals_token: Token![=],
    state: AssignmentPath,
}

enum AssignmentPath {
    OnValueBranch { assignee: ExpressionNodeId },
    OnAwaitingAssignment,
}

impl AssignmentBuilder {
    pub(super) fn start(
        context: ValueContext,
        assignee: ExpressionNodeId,
        equals_token: Token![=],
        value: ExpressionNodeId,
    ) -> NextAction {
        let frame = Self {
            equals_token,
            state: AssignmentPath::OnValueBranch { assignee },
        };
        context.request_owned(frame, value)
    }
}

impl EvaluationFrame for AssignmentBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::Assignment(self)
    }

    fn handle_next(
        mut self,
        context: ValueContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        Ok(match self.state {
            AssignmentPath::OnValueBranch { assignee } => {
                let value = value.expect_owned().into_inner();
                self.state = AssignmentPath::OnAwaitingAssignment;
                context.request_assignment(self, assignee, value)
            }
            AssignmentPath::OnAwaitingAssignment => {
                let AssignmentCompletion = value.expect_assignment_completion();
                context.return_value(Spanned((), span))?
            }
        })
    }
}

pub(super) struct MethodCallBuilder {
    method: MethodAccess,
    unevaluated_parameters_stack: Vec<(ExpressionNodeId, ArgumentOwnership)>,
    state: MethodCallPath,
}

enum MethodCallPath {
    CallerPath,
    ArgumentsPath {
        method: MethodInterface,
        disabled_evaluated_arguments_including_caller: Vec<Spanned<ArgumentValue>>,
    },
}

impl MethodCallBuilder {
    pub(super) fn start(
        context: ValueContext,
        caller: ExpressionNodeId,
        method: MethodAccess,
        parameters: &[ExpressionNodeId],
    ) -> NextAction {
        let frame = Self {
            method,
            unevaluated_parameters_stack: parameters
                .iter()
                .rev()
                .map(|x| {
                    // This is just a placeholder - we'll fix it up shortly
                    (*x, ArgumentOwnership::Owned)
                })
                .collect(),
            state: MethodCallPath::CallerPath,
        };
        context.request_late_bound(frame, caller)
    }
}

impl EvaluationFrame for MethodCallBuilder {
    type ReturnType = ValueType;

    fn into_any(self) -> AnyValueFrame {
        AnyValueFrame::MethodCall(self)
    }

    fn handle_next(
        mut self,
        mut context: ValueContext,
        Spanned(value, span): Spanned<RequestedValue>,
    ) -> ExecutionResult<NextAction> {
        // Handle expected item based on current state
        match self.state {
            MethodCallPath::CallerPath => {
                let caller_span = span;
                let caller = value.expect_late_bound();
                let method = caller
                    .as_ref()
                    .kind()
                    .resolve_method(self.method.method.to_string().as_str());
                let method = match method {
                    Some(m) => m,
                    None => {
                        return self.method.method.type_err(format!(
                            "The method {} does not exist on {}",
                            self.method.method,
                            caller.as_ref().articled_value_type(),
                        ))
                    }
                };
                let non_caller_arguments = self.unevaluated_parameters_stack.len();
                let (argument_ownerships, min_arguments) = method.argument_ownerships();
                let max_arguments = argument_ownerships.len();
                assert!(
                    max_arguments >= 1 && min_arguments >= 1,
                    "Method calls must have at least one argument (the caller)"
                );
                let non_caller_min_arguments = min_arguments - 1;
                let non_caller_max_arguments = max_arguments - 1;

                if non_caller_arguments < non_caller_min_arguments
                    || non_caller_arguments > non_caller_max_arguments
                {
                    return self.method.method.type_err(format!(
                        "The method {} expects {} non-self argument/s, but {} were provided",
                        self.method.method,
                        if non_caller_min_arguments == non_caller_max_arguments {
                            (non_caller_min_arguments).to_string()
                        } else {
                            format!(
                                "{} to {}",
                                (non_caller_min_arguments),
                                (non_caller_max_arguments)
                            )
                        },
                        non_caller_arguments,
                    ));
                }
                let caller =
                    argument_ownerships[0].map_from_late_bound(Spanned(caller, caller_span))?;
                let mut caller = Spanned(caller, caller_span);

                // We skip 1 to ignore the caller
                let non_self_argument_ownerships: iter::Skip<
                    std::slice::Iter<'_, ArgumentOwnership>,
                > = argument_ownerships.iter().skip(1);
                for ((_, requested_ownership), ownership) in self
                    .unevaluated_parameters_stack
                    .iter_mut()
                    .rev() // Swap the stack back to the normal order
                    .zip(non_self_argument_ownerships)
                {
                    *requested_ownership = *ownership;
                }

                self.state = MethodCallPath::ArgumentsPath {
                    disabled_evaluated_arguments_including_caller: {
                        let mut params =
                            Vec::with_capacity(1 + self.unevaluated_parameters_stack.len());
                        unsafe {
                            // SAFETY: We enable it again before use
                            caller.to_mut().disable();
                        }
                        params.push(caller);
                        params
                    },
                    method,
                };
            }
            MethodCallPath::ArgumentsPath {
                ref mut disabled_evaluated_arguments_including_caller,
                ..
            } => {
                let argument = value.expect_argument_value();
                let mut argument = Spanned(argument, span);
                unsafe {
                    // SAFETY: We enable it again before use
                    argument.to_mut().disable();
                }
                disabled_evaluated_arguments_including_caller.push(argument);
            }
        };
        // Now plan the next action
        Ok(match self.unevaluated_parameters_stack.pop() {
            Some((parameter, ownership)) => {
                context.request_any_value(self, parameter, RequestedOwnership::Concrete(ownership))
            }
            None => {
                let (arguments, method) = match self.state {
                    MethodCallPath::CallerPath => unreachable!("Already updated above"),
                    MethodCallPath::ArgumentsPath {
                        disabled_evaluated_arguments_including_caller: mut arguments,
                        method,
                    } => {
                        // NOTE:
                        // - This disable/enable flow allows us to do things like vec.push(vec.len())
                        // - Read https://rust-lang.github.io/rfcs/2025-nested-method-calls.html for more details
                        // - We enable left-to-right for intuitive error messages: later borrows will report errors
                        unsafe {
                            for argument in &mut arguments {
                                // SAFETY: We disabled them above
                                // NOTE: enable() may fail if arguments conflict (e.g., same variable)
                                argument.to_mut().enable()?;
                            }
                        }
                        (arguments, method)
                    }
                };
                let mut call_context = MethodCallContext {
                    output_span_range: self.method.span_range(),
                    interpreter: context.interpreter(),
                };
                let output = method.execute(arguments, &mut call_context)?;
                context.return_returned_value(output)?
            }
        })
    }
}
