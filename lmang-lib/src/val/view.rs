use crate::error::RuntimeError;
use crate::val::Val;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub trait View {
    type Output;
    fn view<T>(
        val: &mut Val,
        f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError>;
}

#[derive(Default)]
pub struct Bottom;

impl View for Bottom {
    type Output = Val;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        f(val)
    }
}

#[derive(Default)]
pub struct Number;

impl View for Number {
    type Output = i32;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        f(val.as_number_mut()?)
    }
}

#[derive(Default)]
pub struct Char;

impl View for Char {
    type Output = char;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        f(val.as_char_mut()?)
    }
}

#[derive(Default)]
pub struct Bool;

impl View for Bool {
    type Output = bool;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        f(val.as_bool_mut()?)
    }
}

#[derive(Default)]
pub struct Break<V: View>(PhantomData<V>);

impl<V: View> View for Break<V> {
    type Output = V::Output;

    fn view<T>(
        val: &mut Val,
        f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        match val {
            Val::Break(v) => V::view(v.as_mut(), f),
            _ => Err(RuntimeError::CastError {
                from: val.variant_name().to_owned(),
                to: Val::Break(Box::new(Val::Unit)).variant_name().to_owned(),
            }),
        }
    }
}

#[derive(Default)]
pub struct Deque;

impl View for Deque {
    type Output = VecDeque<Val>;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        f(val.as_deque_mut()?)
    }
}

#[derive(Default)]
pub struct String;

impl View for String {
    type Output = std::string::String;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        Deque::view(val, move |dq| {
            let all_chars = dq.iter().all(|v| v.as_char().is_ok());
            if all_chars {
                let mut s = dq.iter().map(|v| *v.as_char().unwrap()).collect();
                f(&mut s)
            } else {
                Err(RuntimeError::CastError {
                    from: Val::Deque(Box::new(VecDeque::new()))
                        .variant_name()
                        .to_string(),
                    to: "string".to_string(),
                })
            }
        })
    }
}

#[derive(Default)]
pub struct AnyRef<V: View>(PhantomData<V>);

impl<V: View> View for AnyRef<V> {
    type Output = V::Output;

    fn view<T>(
        val: &mut Val,
        f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        val.apply_to_root_mut(|val_inner| <V as View>::view(val_inner, f))?
    }
}

#[derive(Default)]
pub struct Ref<V: View>(PhantomData<V>);

impl<V: View> View for Ref<V> {
    type Output = V::Output;

    fn view<T>(
        val: &mut Val,
        f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let refrc = val.as_val_ref_mut()?;
        let mut borrow = refrc.borrow_mut();

        <V as View>::view(&mut *borrow, f)
    }
}

#[cfg(feature = "web")]
#[derive(Default)]
pub struct Js;

#[cfg(feature = "web")]
impl View for Js {
    type Output = wasm_bindgen::JsValue;

    fn view<T>(
        val: &mut Val,
        mut f: impl FnMut(&mut Self::Output) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        match val {
            Val::JsValue(jv) => f(jv),
            _ => Err(RuntimeError::CastError {
                from: val.variant_name().to_owned(),
                to: Val::JsValue(wasm_bindgen::JsValue::NULL)
                    .variant_name()
                    .to_owned(),
            }),
        }
    }
}

pub fn view1<V1, F, T>(vals: &mut [Val], mut f: F) -> Result<(T, &mut [Val]), RuntimeError>
where
    V1: View,
    F: FnMut(&mut V1::Output) -> Result<T, RuntimeError>,
{
    let ([val1], tail) = take_n::<1>(vals)?;
    let res = V1::view(val1, |v1| f(v1))?;

    Ok((res, tail))
}

pub fn view2<V1, V2, F, T>(vals: &mut [Val], mut f: F) -> Result<(T, &mut [Val]), RuntimeError>
where
    V1: View,
    V2: View,
    F: FnMut(&mut V1::Output, &mut V2::Output) -> Result<T, RuntimeError>,
{
    let ([val1, val2], tail) = take_n::<2>(vals)?;
    let res = V1::view(val1, |v1| V2::view(val2, |v2| f(v1, v2)))?;

    Ok((res, tail))
}

pub fn foreach<ViewIter, ViewInner, F, T>(
    vals: &mut [Val],
    mut f: F,
) -> Result<(Vec<T>, &mut [Val]), RuntimeError>
where
    ViewIter: View,
    ViewInner: View,
    for<'x> &'x mut ViewIter::Output: IntoIterator<Item = &'x mut Val>,
    F: FnMut(&mut ViewInner::Output) -> Result<T, RuntimeError>,
{
    let mut results = Vec::new();

    let (_, tail) = view1::<ViewIter, _, _>(vals, |dq| {
        for val in dq.into_iter() {
            <ViewInner as View>::view(val, |v| {
                results.push(f(v)?);
                Ok(())
            })?;
        }

        Ok(())
    })?;

    Ok((results, tail))
}

pub fn take_n<const N: usize>(
    vals: &mut [Val],
) -> Result<(&mut [Val; N], &mut [Val]), RuntimeError> {
    if vals.len() < N {
        Err(RuntimeError::WrongArgsN)
    } else {
        let (requested, tail) = vals.split_at_mut(N);
        let requested = requested.try_into().unwrap();

        Ok((requested, tail))
    }
}

pub fn test_consumed(tail: &[Val]) -> Result<(), RuntimeError> {
    if tail.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::WrongArgsN)
    }
}

pub trait DequeExt {
    fn try_get(&mut self, idx: i32) -> Result<&mut Val, RuntimeError>;
    fn try_remove(&mut self, idx: i32) -> Result<Val, RuntimeError>;
}

impl DequeExt for VecDeque<Val> {
    fn try_get(&mut self, idx: i32) -> Result<&mut Val, RuntimeError> {
        let len = self.len();
        let idx_rel = if idx >= 0 {
            idx as usize
        } else {
            use std::ops::Sub;
            len.sub((-idx) as usize)
        };

        self.get_mut(idx_rel)
            .ok_or_else(|| RuntimeError::OutOfBounds {
                idx: idx_rel as i32,
                len,
            })
    }

    fn try_remove(&mut self, idx: i32) -> Result<Val, RuntimeError> {
        let len = self.len();
        let idx_rel = if idx >= 0 {
            idx as usize
        } else {
            use std::ops::Sub;
            len.sub((-idx) as usize)
        };

        self.remove(idx_rel)
            .ok_or_else(|| RuntimeError::OutOfBounds {
                idx: idx_rel as i32,
                len,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn check_ref_ref_int() {
        let ref_ref_int_constructor = |i| {
            Val::Ref(Rc::new(RefCell::new(Val::Ref(Rc::new(RefCell::new(
                Val::Number(i),
            ))))))
        };
        let mut val = ref_ref_int_constructor(42);
        let res = Ref::<Ref<Number>>::view(&mut val, |i| {
            *i = 1;
            Ok(())
        });

        assert_eq!(res, Ok(()));
        assert_eq!(val, ref_ref_int_constructor(1));
    }

    #[test]
    fn check_int_x2_nested() {
        let mut val1 = Val::Number(1);
        let mut val2 = Val::Number(2);

        let res = Number::view(&mut val1, |v1| {
            Number::view(&mut val2, |v2| {
                *v1 += *v2;

                Ok(())
            })
        });

        assert_eq!(res, Ok(()));
        assert_eq!(val1, Val::Number(3));
    }

    #[test]
    fn test_int_x2_flat() {
        let mut vals = [Val::Number(2), Val::Number(12)];

        let res = view2::<Number, Number, _, _>(&mut vals, |v1, v2| {
            *v2 /= *v1;
            Ok(())
        });

        assert_eq!(res, Ok(((), [].as_mut_slice())));
        assert_eq!(vals[1], Val::Number(6));
    }
}
