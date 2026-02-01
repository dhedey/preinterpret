use super::*;

pub(crate) struct ClosureExpression(Rc<ClosureDefinition>);

impl ParseSource for ClosureExpression {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self(Rc::new(input.parse()?)))
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        Rc::get_mut(&mut self.0)
            .expect("ClosureExpressionInner should be uniquely owned during the control flow pass")
            .control_flow_pass(context)
    }
}

impl ClosureExpression {
    pub(crate) fn evaluate_spanned(
        &self,
        interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> FunctionResult<Spanned<RequestedValue>> {
        let span_range = self.0.span_range;
        let value = ClosureValue {
            definition: Rc::clone(&self.0),
            closed_references: interpreter.resolve_closed_references(self.0.frame_id),
        };
        ownership.map_from_owned(value.into_any_value().spanned(span_range))
    }
}

#[derive(Clone)]
pub(crate) struct ClosureValue {
    definition: Rc<ClosureDefinition>,
    closed_references: Vec<(VariableDefinitionId, VariableContent)>,
    // TODO[functions]: Add closed_values from moves here
}

impl PartialEq for ClosureValue {
    fn eq(&self, other: &Self) -> bool {
        // A few options here:
        // 1. Always return false. This is simple, but non-reflexive and non-intuitive.
        //    e.g. NaN works like this
        // 2. Check just for closure equality
        // 3. Also check for equality of bound values
        // I think 2 makes the most sense for now
        Rc::ptr_eq(&self.definition, &other.definition)
    }
}

impl ClosureValue {
    /// Returns (argument_ownerships, required_argument_count)
    pub(crate) fn argument_ownerships(&self) -> (&[ArgumentOwnership], usize) {
        (
            &self.definition.argument_ownerships,
            self.definition.required_argument_count,
        )
    }

    pub(crate) fn invoke(
        self,
        arguments: Vec<Spanned<ArgumentValue>>,
        context: &mut FunctionCallContext,
    ) -> FunctionResult<Spanned<ReturnedValue>> {
        let definition = &*self.definition;

        context.interpreter.enter_function_boundary_scope(
            definition.scope_id,
            definition.frame_id,
            context.output_span_range,
        ).expect_no_interrupts()?;
        for (definition, content) in self.closed_references {
            context.interpreter.define_variable(definition, content);
        }

        for (pattern, Spanned(arg, arg_span)) in
            definition.argument_definitions.iter().zip(arguments)
        {
            match (pattern, arg) {
                (pattern, ArgumentValue::Owned(owned)) => {
                    pattern.handle_destructure(context.interpreter, owned).expect_no_interrupts()?;
                }
                (Pattern::Discarded(_), _) => {}
                (Pattern::Variable(variable), ArgumentValue::Shared(shared)) => {
                    context.interpreter.define_variable(
                        variable.definition.id,
                        VariableContent::Shared(shared.disable()),
                    );
                }
                (_, ArgumentValue::Shared(_)) => {
                    return arg_span.type_err(
                        "Destructuring patterns are not currently supported for & arguments",
                    );
                }
                (Pattern::Variable(variable), ArgumentValue::Mutable(mutable)) => {
                    context.interpreter.define_variable(
                        variable.definition.id,
                        VariableContent::Mutable(mutable.disable()),
                    );
                }
                (_, ArgumentValue::Mutable(_)) => {
                    return arg_span.type_err(
                        "Destructuring patterns are not currently supported for &mut arguments",
                    );
                }
                (_, _) => {
                    return arg_span.type_err(
                        "Only owned, shared and mutable arguments are currently supported for closures",
                    );
                }
            }
        }

        let Spanned(output, body_span) = definition.body.evaluate(
            context.interpreter,
            RequestedOwnership::Concrete(ArgumentOwnership::AsIs),
        ).expect_no_interrupts()?;

        let returned_value = match output {
            RequestedValue::Owned(any_value) => ReturnedValue::Owned(any_value),
            RequestedValue::Shared(any_value) => ReturnedValue::Shared(any_value),
            RequestedValue::Mutable(any_value) => ReturnedValue::Mutable(any_value),
            RequestedValue::CopyOnWrite(any_value) => ReturnedValue::CopyOnWrite(any_value),
            _ => {
                return body_span.type_err(
                    "Closure body must evaluate to an Owned, Shared, Mutable, or CopyOnWrite value",
                );
            }
        };

        context.interpreter.exit_scope(definition.scope_id);

        Ok(Spanned(returned_value, context.output_span_range))
    }
}

impl PartialEq for ClosureExpression {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ClosureExpression {}

pub(crate) struct ClosureDefinition {
    frame_id: FrameId,
    scope_id: ScopeId,
    required_argument_count: usize,
    argument_ownerships: Vec<ArgumentOwnership>,
    argument_definitions: Vec<Pattern>,
    body: Expression,
    span_range: SpanRange,
}

impl ParseSource for ClosureDefinition {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let start_span = input.span();
        let _left_bar = input.parse::<syn::Token![|]>()?;
        let punctuated_arguments = input
            .parse_punctuated_until::<FunctionArgument, syn::Token![,]>(|x| x.peek(Token![|]))?;
        let _right_bar = input.parse::<syn::Token![|]>()?;
        let body = input.parse()?;

        // Really we want to use the span of the last token in the body expression.
        // But painfully in syn, Cursor::prev_span is not public.
        //
        // We could do a few alternatives:
        // - Do some unsafe transmuting
        // - Fork syn to expose prev_span (ouch - compile times)
        // - Expose an expensive walker on Expression that finds the last span
        // - Pass the current end span through the ExpressionParser during parsing
        //
        // For now we'll use the off-by-one current span of the input.
        let end_span = input.cursor().span();

        let mut argument_ownerships = Vec::with_capacity(punctuated_arguments.len());
        let mut argument_definitions = Vec::with_capacity(punctuated_arguments.len());
        for arg in punctuated_arguments.into_iter() {
            let ownership = if let Some(annotation) = &arg.annotation {
                match &annotation.argument_specifier {
                    ArgumentSpecifier::ByValue { .. } => ArgumentOwnership::Owned,
                    ArgumentSpecifier::BySharedRef { .. } => ArgumentOwnership::Shared,
                    ArgumentSpecifier::ByMutableRef { .. } => ArgumentOwnership::Mutable,
                }
            } else {
                // Default to by-value
                ArgumentOwnership::Owned
            };
            let pattern = arg.pattern;
            match (&ownership, &pattern) {
                (ArgumentOwnership::Owned, _) => {}
                (_, Pattern::Variable(_)) => {}
                _ => {
                    return pattern.parse_err(
                        "Destructuring patterns are not currently supported for & and &mut arguments",
                    );
                }
            }
            argument_ownerships.push(ownership);
            argument_definitions.push(pattern);
        }

        Ok(Self {
            frame_id: FrameId::new_placeholder(),
            scope_id: ScopeId::new_placeholder(),
            required_argument_count: argument_ownerships.len(),
            argument_ownerships,
            argument_definitions,
            body,
            span_range: SpanRange::new_between(start_span, end_span),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_frame(&mut self.frame_id);
        context.register_scope(&mut self.scope_id);
        context.enter_closure_frame(self.frame_id, self.scope_id);
        for argument in &mut self.argument_definitions {
            argument.control_flow_pass(context)?;
        }
        self.body.control_flow_pass(context)?;
        context.exit_frame(self.frame_id, self.scope_id);
        Ok(())
    }
}

struct FunctionArgument {
    pattern: Pattern,
    annotation: Option<FunctionArgumentAnnotation>,
}

impl ParseSource for FunctionArgument {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            pattern: input.parse()?,
            annotation: if input.peek(syn::Token![:]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        self.pattern.control_flow_pass(context)
    }
}

struct FunctionArgumentAnnotation {
    _colon: Unused<syn::Token![:]>,
    argument_specifier: ArgumentSpecifier,
}

// They're clearer with a common prefix By
#[allow(clippy::enum_variant_names)]
enum ArgumentSpecifier {
    ByValue {
        _argument_type: ArgumentType,
    },
    BySharedRef {
        _ampersand: Unused<syn::Token![&]>,
        _argument_type: ArgumentType,
    },
    ByMutableRef {
        _ampersand: Unused<syn::Token![&]>,
        _mut_token: Unused<syn::Token![mut]>,
        _argument_type: ArgumentType,
    },
}

impl ParseSource for FunctionArgumentAnnotation {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let _colon = input.parse()?;
        let _argument_specifier = if input.peek(syn::Token![&]) {
            let _ampersand = input.parse()?;
            if input.peek(syn::Token![mut]) {
                let _mut_token = input.parse()?;
                let _argument_type = input.parse()?;
                ArgumentSpecifier::ByMutableRef {
                    _ampersand,
                    _mut_token,
                    _argument_type,
                }
            } else {
                let _argument_type = input.parse()?;
                ArgumentSpecifier::BySharedRef {
                    _ampersand,
                    _argument_type,
                }
            }
        } else {
            let _argument_type = input.parse()?;
            ArgumentSpecifier::ByValue { _argument_type }
        };
        Ok(Self {
            _colon,
            argument_specifier: _argument_specifier,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}

struct ArgumentType {
    _type_ident: TypeIdent,
}

impl ParseSource for ArgumentType {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        if !input.peek_ident_matching("any") {
            return input.parse_err(
                "Only the type `any` is currently supported as an argument type annotation",
            );
        }
        Ok(Self {
            _type_ident: input.parse()?,
        })
    }

    fn control_flow_pass(&mut self, _context: FlowCapturer) -> ParseResult<()> {
        Ok(())
    }
}
