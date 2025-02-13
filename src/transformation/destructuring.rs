use crate::internal_prelude::*;

pub(crate) trait HandleDestructure {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()>;
}

#[derive(Clone)]
pub(crate) enum Destructuring {
    Variable(VariableDestructuring),
    Array(ArrayDestructuring),
    Stream(ExplicitTransformStream),
    #[allow(unused)]
    Discarded(Token![_]),
}

impl Parse<Source> for Destructuring {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Destructuring::Variable(input.parse()?))
        } else if lookahead.peek(syn::token::Bracket) {
            Ok(Destructuring::Array(input.parse()?))
        } else if lookahead.peek(Token![@]) {
            Ok(Destructuring::Stream(input.parse()?))
        } else if lookahead.peek(Token![_]) {
            Ok(Destructuring::Discarded(input.parse()?))
        } else if input.peek(Token![#]) {
            return input.parse_err("Use `var` instead of `#var` in a destructuring");
        } else {
            Err(lookahead.error().into())
        }
    }
}

impl HandleDestructure for Destructuring {
    fn handle_destructure(
        &self,
        interpreter: &mut Interpreter,
        value: ExpressionValue,
    ) -> ExecutionResult<()> {
        match self {
            Destructuring::Variable(variable) => variable.handle_destructure(interpreter, value),
            Destructuring::Array(array) => array.handle_destructure(interpreter, value),
            Destructuring::Stream(stream) => stream.handle_destructure(interpreter, value),
            Destructuring::Discarded(_) => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct ArrayDestructuring {
    #[allow(unused)]
    delim_span: DelimSpan,
    items: Punctuated<Destructuring, Token![,]>,
}

impl Parse<Source> for ArrayDestructuring {
    fn parse(input: ParseStream<Source>) -> ParseResult<Self> {
        let (delim_span, inner) = input.parse_specific_group(Delimiter::Bracket)?;
        Ok(Self {
            delim_span,
            items: inner.parse_terminated()?,
        })
    }
}

impl HandleDestructure for ArrayDestructuring {
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
