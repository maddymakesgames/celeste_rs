#![allow(unused)]
use anyhow::{anyhow, Error};
use std::{
    fmt,
    iter::{FusedIterator, Map},
};

pub fn anyhow_ok_or<T>(err: &'static str) -> impl FnMut(Option<T>) -> Result<T, Error> {
    move |x| x.anyhow(err)
}

pub(crate) trait AnyhowOption<T> {
    fn anyhow(self, str: &'static str) -> Result<T, Error>;
}

impl<T> AnyhowOption<T> for Option<T> {
    fn anyhow(self, str: &'static str) -> Result<T, Error> {
        self.ok_or(anyhow!(str))
    }
}

pub(crate) trait AnyhowIter<T>: Iterator<Item = Option<T>> {
    fn anyhow(self, str: &'static str) -> Map<Self, impl FnMut(Option<T>) -> Result<T, Error>>
    where Self: Sized {
        self.map(anyhow_ok_or(str))
    }
}

impl<T, B> AnyhowIter<B> for T where T: Iterator<Item = Option<B>> {}

pub(crate) trait OptionOkOrIter<T, E: Clone>: Iterator<Item = Option<T>> {
    fn ok_or(self, val: E) -> OkOrIter<Self, E>
    where Self: Sized {
        OkOrIter {
            iter: self,
            err: val,
        }
    }
}

impl<T, I, E: Clone> OptionOkOrIter<I, E> for T where T: Iterator<Item = Option<I>> + ?Sized {}

pub(crate) trait ResultMapIter<T, E>: Iterator<Item = Result<T, E>> {
    fn map_result<U, F: FnMut(T) -> U>(self, f: F) -> ResultMap<Self, F>
    where Self: Sized {
        ResultMap { iter: self, f }
    }
}

impl<T, R, E> ResultMapIter<R, E> for T where T: Iterator<Item = Result<R, E>> + ?Sized {}

#[derive(Clone)]
pub(crate) struct ResultMap<I, F> {
    // Used for `SplitWhitespace` and `SplitAsciiWhitespace` `as_str` methods
    iter: I,
    f: F,
}

impl<I: fmt::Debug, F> fmt::Debug for ResultMap<I, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResultMap")
            .field("iter", &self.iter)
            .finish()
    }
}

fn map_fold<T, E, B, Acc>(
    mut f: impl FnMut(T) -> B,
    mut g: impl FnMut(Acc, Result<B, E>) -> Acc,
) -> impl FnMut(Acc, Result<T, E>) -> Acc {
    move |acc, elt| {
        g(acc, match elt {
            Ok(t) => Ok(f(t)),
            Err(e) => Err(e),
        })
    }
}

impl<B, T, E, I: Iterator<Item = Result<T, E>>, F> Iterator for ResultMap<I, F>
where F: FnMut(T) -> B
{
    type Item = Result<B, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|r| r.map(&mut self.f))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<Acc, G>(self, init: Acc, g: G) -> Acc
    where G: FnMut(Acc, Self::Item) -> Acc {
        self.iter.fold(init, map_fold(self.f, g))
    }
}

impl<B, T, E, I: DoubleEndedIterator + Iterator<Item = Result<T, E>>, F> DoubleEndedIterator
    for ResultMap<I, F>
where F: FnMut(T) -> B
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|r| r.map(&mut self.f))
    }

    fn rfold<Acc, G>(self, init: Acc, g: G) -> Acc
    where G: FnMut(Acc, Self::Item) -> Acc {
        self.iter.rfold(init, map_fold(self.f, g))
    }
}

impl<B, T, E, I: ExactSizeIterator + Iterator<Item = Result<T, E>>, F> ExactSizeIterator
    for ResultMap<I, F>
where F: FnMut(T) -> B
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<B, T, E, I: FusedIterator + Iterator<Item = Result<T, E>>, F> FusedIterator for ResultMap<I, F> where F: FnMut(T) -> B
{}


#[derive(Clone)]
pub(crate) struct OkOrIter<I, E> {
    iter: I,
    err: E,
}

impl<I: fmt::Debug, F> fmt::Debug for OkOrIter<I, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OkOrIter")
            .field("iter", &self.iter)
            .finish()
    }
}

impl<T, E: Clone, I: Iterator<Item = Option<T>>> Iterator for OkOrIter<I, E> {
    type Item = Result<T, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|r| r.ok_or(self.err.clone()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, E: Clone, I: DoubleEndedIterator + Iterator<Item = Option<T>>> DoubleEndedIterator
    for OkOrIter<I, E>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|r| r.ok_or(self.err.clone()))
    }
}

impl<T, E: Clone, I: ExactSizeIterator + Iterator<Item = Option<T>>> ExactSizeIterator
    for OkOrIter<I, E>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, E: Clone, I: FusedIterator + Iterator<Item = Option<T>>> FusedIterator for OkOrIter<I, E> {}
