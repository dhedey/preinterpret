use crate::internal_prelude::*;

pub(crate) trait StopCondition: Clone {
    fn should_stop(input: ParseStream) -> bool;
}

#[derive(Clone)]
pub(crate) struct UntilEnd;
impl StopCondition for UntilEnd {
    fn should_stop(input: ParseStream) -> bool {
        input.is_empty()
    }
}

pub(crate) trait PeekableToken: Clone {
    fn peek(input: ParseStream) -> bool;
}

// This is going through such pain to ensure we stay in the public API of syn
// We'd like to be able to use `input.peek::<T>()` for some syn::token::Token
// but that's just not an API they support for some reason
macro_rules! impl_peekable_token {
    ($(Token![$token:tt]),* $(,)?) => {
        $(
            impl PeekableToken for Token![$token] {
                fn peek(input: ParseStream) -> bool {
                    input.peek(Token![$token])
                }
            }
        )*
    };
}

impl_peekable_token! {
    Token![=],
    Token![in],
}

#[derive(Clone)]
pub(crate) struct UntilToken<T: PeekableToken> {
    token: PhantomData<T>,
}
impl<T: PeekableToken> StopCondition for UntilToken<T> {
    fn should_stop(input: ParseStream) -> bool {
        input.is_empty() || T::peek(input)
    }
}

pub(crate) type DestructureRemaining = DestructureSegment<UntilEnd>;
pub(crate) type DestructureUntil<T> = DestructureSegment<UntilToken<T>>;

#[derive(Clone)]
pub(crate) struct DestructureSegment<C: StopCondition> {
    stop_condition: PhantomData<C>,
    inner: Vec<DestructureItem>,
}

impl<C: StopCondition> Parse for DestructureSegment<C> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut inner = vec![];
        while !C::should_stop(input) {
            inner.push(DestructureItem::parse_until::<C>(input)?);
        }
        Ok(Self {
            stop_condition: PhantomData,
            inner,
        })
    }
}

impl<C: StopCondition> HandleDestructure for DestructureSegment<C> {
    fn handle_destructure(
        &self,
        input: ParseStream,
        interpreter: &mut Interpreter,
    ) -> ExecutionResult<()> {
        for item in self.inner.iter() {
            item.handle_destructure(input, interpreter)?;
        }
        Ok(())
    }
}
