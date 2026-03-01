use super::*;

pub(super) enum VariableState {
    Uninitialized,
    Value(VariableContent),
    Finished,
}

/// Cheaply clonable, inactive content of a variable.
/// By inactive, we mean that the shared/mutable aliasing invariants are not required to hold.
/// Before use, they have to be activated. This allows more flexibility over how references
/// can be used and held, without compromising safety.
#[derive(Clone)]
pub(crate) enum VariableContent {
    // Owned, but possibly with pre-existing references
    Referenceable(Referenceable),
    Shared(InactiveShared<AnyValue>),
    Mutable(InactiveMutable<AnyValue>),
}

impl VariableContent {
    fn into_owned_as_only_owner(self) -> Result<Owned<AnyValue>, VariableContent> {
        match self {
            VariableContent::Referenceable(data) => match data.try_into_inner() {
                Ok(owned) => Ok(owned),
                Err(original) => Err(VariableContent::Referenceable(original)),
            },
            other => Err(other),
        }
    }
}

const UNINITIALIZED_ERR: &str = "Cannot resolve uninitialized variable. This shouldn't be possible, because all variables are set on first use.";
const FINISHED_ERR: &str = "Cannot resolve finished variable. This shouldn't be possible, because is_final should be marked correctly. If you see this error, please report a bug to preinterpret on GitHub with a reproduction case.";

impl VariableState {
    pub(crate) fn define(&mut self, value: VariableContent) {
        match self {
            content @ VariableState::Uninitialized => {
                *content = VariableState::Value(value);
            }
            VariableState::Value(_) => panic!("Cannot define existing variable"),
            VariableState::Finished => panic!("Cannot define finished variable"),
        }
    }

    pub(crate) fn resolve_content(
        &mut self,
        is_final: bool,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> VariableContent {
        if is_final && blocked_from_mutation.is_none() {
            let content = std::mem::replace(self, VariableState::Finished);
            match content {
                VariableState::Uninitialized => panic!("{}", UNINITIALIZED_ERR),
                VariableState::Value(content) => content,
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        } else {
            match self {
                VariableState::Uninitialized => panic!("{}", UNINITIALIZED_ERR),
                VariableState::Value(content) => content.clone(),
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        }
    }

    pub(crate) fn resolve(
        &mut self,
        variable_span: Span,
        is_final: bool,
        ownership: RequestedOwnership,
        blocked_from_mutation: Option<MutationBlockReason>,
    ) -> FunctionResult<Spanned<AnyValueLateBound>> {
        let span_range = variable_span.span_range();

        // If blocked from mutation, we technically could allow is_final to work and
        // return a fully owned value without observable mutation,
        // but it's likely confusingly inconsistent, so it's better to just block it entirely.
        let content = if is_final && blocked_from_mutation.is_none() {
            let content = std::mem::replace(self, VariableState::Finished);
            match content {
                VariableState::Uninitialized => panic!("{}", UNINITIALIZED_ERR),
                VariableState::Value(content) => match content.into_owned_as_only_owner() {
                    Ok(owned) => {
                        if matches!(
                            ownership,
                            RequestedOwnership::Concrete(ArgumentOwnership::Assignee { .. })
                        ) {
                            return variable_span.control_flow_err("The final usage of a variable cannot be assigned to. You can use `let _ = ..` to discard a value.");
                        }
                        return Ok(Spanned(
                            LateBound::Owned(LateBoundOwned {
                                owned,
                                is_from_last_use: true,
                            }),
                            span_range,
                        ));
                    }
                    // It's currently referenced elsewhere, proceed with normal late-bound resolution.
                    // e.g.
                    // * `let x = %[]; x.assert_eq(x, %[]);` - the final `x` resolves to a shared reference
                    // * `let x = %[]; x.assert_eq(x + %[], %[]);` - errors because the final `x` is shared but it needs to be owned
                    // Or, it could be captured somewhere else, e.g. in a closure:
                    // * `let x = %[]; let f = || x; let y = x;` - the final `x` resolves to a shared reference
                    Err(content) => content,
                },
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        } else {
            match self {
                VariableState::Uninitialized => panic!("{}", UNINITIALIZED_ERR),
                VariableState::Value(content) => content.clone(),
                VariableState::Finished => panic!("{}", FINISHED_ERR),
            }
        };
        let binding = VariableBinding {
            content,
            variable_span,
        };
        let resolved = match ownership {
            RequestedOwnership::LateBound => binding.into_late_bound(),
            RequestedOwnership::Concrete(ownership) => match ownership {
                ArgumentOwnership::Owned => binding.into_transparently_cloned().map(|owned| {
                    LateBound::Owned(LateBoundOwned {
                        owned,
                        is_from_last_use: false,
                    })
                }),
                ArgumentOwnership::Shared => binding
                    .into_shared()
                    .map(CopyOnWrite::shared_in_place_of_shared)
                    .map(AnyValueLateBound::CopyOnWrite),
                ArgumentOwnership::Assignee { .. } => {
                    binding.into_mut().map(AnyValueLateBound::Mutable)
                }
                ArgumentOwnership::Mutable => binding.into_mut().map(AnyValueLateBound::Mutable),
                ArgumentOwnership::CopyOnWrite | ArgumentOwnership::AsIs => binding
                    .into_shared()
                    .map(CopyOnWrite::shared_in_place_of_shared)
                    .map(AnyValueLateBound::CopyOnWrite),
            },
        };
        let late_bound = if let Some(mutation_block_reason) = blocked_from_mutation {
            match resolved {
                Ok(AnyValueLateBound::Mutable(mutable)) => {
                    let reason_not_mutable = variable_span
                        .syn_error(mutation_block_reason.error_message("mutate this variable"));
                    Ok(LateBound::new_shared(
                        mutable.into_shared(),
                        reason_not_mutable,
                    ))
                }
                x => x,
            }
        } else {
            resolved
        }?;
        Ok(Spanned(late_bound, span_range))
    }
}

#[derive(Clone)]
struct VariableBinding {
    content: VariableContent,
    variable_span: Span,
}

impl VariableBinding {
    /// Gets the cloned expression value
    /// This only works if the value can be transparently cloned
    pub(crate) fn into_transparently_cloned(self) -> FunctionResult<AnyValue> {
        let span_range = self.variable_span.span_range();
        let shared = self.into_shared()?;
        let value = shared.as_ref().try_transparent_clone(span_range)?;
        Ok(value)
    }

    fn into_mut(self) -> FunctionResult<AnyValueMutable> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => referenceable.new_active_mutable(span),
            VariableContent::Mutable(inactive) => inactive.activate(span),
            VariableContent::Shared(_) => self
                .variable_span
                .ownership_err(SHARED_TO_MUTABLE_ERROR_MESSAGE),
        }
    }

    fn into_shared(self) -> FunctionResult<AnyValueShared> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => referenceable.new_active_shared(span),
            VariableContent::Mutable(inactive) => inactive.into_shared().activate(span),
            VariableContent::Shared(inactive) => inactive.activate(span),
        }
    }

    fn into_late_bound(self) -> FunctionResult<AnyValueLateBound> {
        let span = self.variable_span.span_range();
        match self.content {
            VariableContent::Referenceable(referenceable) => {
                match referenceable.new_active_mutable(span) {
                    Ok(mutable) => Ok(AnyValueLateBound::Mutable(mutable)),
                    Err(err) => {
                        // Mutable failed, try shared
                        let shared = referenceable.new_active_shared(span)?;
                        Ok(LateBound::new_shared(shared, err.expect_ownership_error()?))
                    }
                }
            }
            VariableContent::Mutable(inactive) => Ok(LateBound::Mutable(inactive.activate(span)?)),
            VariableContent::Shared(inactive) => Ok(LateBound::new_shared(
                inactive.activate(span)?,
                self.variable_span
                    .syn_error(SHARED_TO_MUTABLE_ERROR_MESSAGE),
            )),
        }
    }
}

static SHARED_TO_MUTABLE_ERROR_MESSAGE: &str = "Cannot mutate a non-mutable reference";
