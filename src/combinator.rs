use winnow::error::ParserError;
use winnow::stream::{AsChar, Stream, StreamIsPartial};
use winnow::token::take_till;
use winnow::trace::trace;
use winnow::{PResult, Parser};

#[inline]
pub fn not_whitespace<I, E: ParserError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    trace("not_whitespace", take_till(1.., AsChar::is_space)).parse_next(input)
}
