use crate::internal_prelude::*;

pub(crate) trait HandleDestructure {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()>;
}

#[derive(Clone)]
pub(crate) enum Pattern {
    Variable(VariablePattern),
    Array(ArrayPattern),
    Stream(ExplicitTransformStream),
    #[allow(unused)]
    Discarded(Token![_]),
}

impl Parse<Source> for Pattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Pattern::Variable(input.parse()?))
        } else if lookahead.peek(syn::token::Bracket) {
            Ok(Pattern::Array(input.parse()?))
        } else if lookahead.peek(Token![@]) {
            Ok(Pattern::Stream(input.parse()?))
        } else if lookahead.peek(Token![_]) {
            Ok(Pattern::Discarded(input.parse()?))
        } else if input.peek(Token![#]) {
            return input.parse_err("Use `var` instead of `#var` in a destructuring");
        } else {
            Err(lookahead.error().into())
        }
    }
}

impl HandleDestructure for Pattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        match self {
            Pattern::Variable(variable) => variable.handle_destructure(interpreter, value),
            Pattern::Array(array) => array.handle_destructure(interpreter, value),
            Pattern::Stream(stream) => stream.handle_destructure(interpreter, value),
            Pattern::Discarded(_) => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct ArrayPattern {
    #[allow(unused)]
    delim_span: DelimSpan,
    items: Punctuated<Pattern, Token![,]>,
}

impl Parse<Source> for ArrayPattern {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delim_span, inner) = input.parse_specific_group(Delimiter::Bracket)?;
        Ok(Self {
            delim_span,
            items: inner.parse_terminated()?,
        })
    }
}

impl HandleDestructure for ArrayPattern {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        let array = value.expect_array("The destructure source")?;
        if array.items.len() != self.items.len() {
            return array.execution_err(format!(
                "The array has {} items, but the destructuring expected {}",
                array.items.len(),
                self.items.len()
            ));
        }
        for (value, destructuring) in array.items.into_iter().zip(&self.items) {
            destructuring.handle_destructure(interpreter, value)?;
        }
        Ok(())
    }
}
