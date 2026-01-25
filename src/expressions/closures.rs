use super::*;

#[derive(Clone)]
pub(crate) struct ClosureExpression(Rc<ClosureExpressionInner>);

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
        _interpreter: &mut Interpreter,
        ownership: RequestedOwnership,
    ) -> ExecutionResult<Spanned<RequestedValue>> {
        let span_range = self.0.span_range;
        ownership.map_from_owned(self.clone().into_any_value().spanned(span_range))
    }
}

impl PartialEq for ClosureExpression {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ClosureExpression {}

pub(crate) struct ClosureExpressionInner {
    frame_id: FrameId,
    scope_id: ScopeId,
    _left_bar: Unused<syn::Token![|]>,
    arguments: Punctuated<FunctionArgument, syn::Token![,]>,
    _right_bar: Unused<syn::Token![|]>,
    body: Expression,
    span_range: SpanRange,
}

impl ParseSource for ClosureExpressionInner {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        let left_bar: syn::Token![|] = input.parse()?;
        let start_span = left_bar.span;
        let arguments = input.parse_terminated()?;
        let _right_bar = input.parse()?;
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

        Ok(Self {
            frame_id: FrameId::new_placeholder(),
            scope_id: ScopeId::new_placeholder(),
            _left_bar: Unused::new(left_bar),
            arguments,
            _right_bar,
            body,
            span_range: SpanRange::new_between(start_span, end_span),
        })
    }

    fn control_flow_pass(&mut self, context: FlowCapturer) -> ParseResult<()> {
        context.register_frame(&mut self.frame_id);
        context.register_scope(&mut self.scope_id);
        context.enter_frame(self.frame_id, self.scope_id);
        for argument in &mut self.arguments {
            argument.pattern.control_flow_pass(context)?;
        }
        self.body.control_flow_pass(context)?;
        context.exit_frame(self.frame_id, self.scope_id);
        Ok(())
    }
}

struct FunctionArgument {
    pattern: Pattern,
    _annotation: Option<FunctionArgumentAnnotation>,
}

impl ParseSource for FunctionArgument {
    fn parse(input: SourceParser) -> ParseResult<Self> {
        Ok(Self {
            pattern: input.parse()?,
            _annotation: if input.peek(syn::Token![:]) {
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
    _argument_specifier: ArgumentSpecifier,
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
            _argument_specifier,
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
