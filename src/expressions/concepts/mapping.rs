use super::*;

pub(crate) trait LeafMapper<F: IsHierarchicalForm> {
    type Output<'a, T: IsHierarchicalType>;

    fn to_parent_output<'a, T: IsChildType>(
        output: Self::Output<'a, T>,
    ) -> Self::Output<'a, T::ParentType>;

    fn map_leaf<'a, T: IsLeafType>(self, leaf: F::Leaf<'a, T>) -> Self::Output<'a, T>;
}

pub(crate) trait RefLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r F::Leaf<'a, T>,
    ) -> Self::Output<'r, 'a, T>;
}

pub(crate) trait MutLeafMapper<F: IsHierarchicalForm> {
    type Output<'r, 'a: 'r, T: IsHierarchicalType>;

    fn to_parent_output<'r, 'a: 'r, T: IsChildType>(
        output: Self::Output<'r, 'a, T>,
    ) -> Self::Output<'r, 'a, T::ParentType>;

    fn map_leaf<'r, 'a: 'r, T: IsLeafType>(
        self,
        leaf: &'r mut F::Leaf<'a, T>,
    ) -> Self::Output<'r, 'a, T>;
}

/// Macro for creating and immediately applying a leaf mapper inline.
///
/// This macro creates an anonymous mapper struct, implements the appropriate mapper trait
/// (LeafMapper, RefLeafMapper, or MutLeafMapper) based on the leaf pattern, and applies
/// it to the input content.
///
/// Specific output type forms are supported for parent propogation.
/// If you get a cryptic error in the to_parent method, you may need to add
/// an explicit implementation into [`__map_via_leaf_to_parent`]
///
/// # Syntax
///
/// ```ignore
/// map_via_leaf! {
///     input: [<none> | &'r | &'r mut] (Content<'a, InputType, InputForm>) = input_value,
///     state: StateType | let state_binding = state_init,  // Optional
///     fn map_leaf<F [<none> | : <bounds> | = <fixed type>], T>(leaf) -> (OutputType)
///     [where <...>]  // Optional
///     {
///         // mapper body
///     }
/// }
/// ```
macro_rules! map_via_leaf {
    (
        input: $(&$r:lifetime $($mut:ident)?)? (Content<$a:lifetime, $input_type:ty, $input_form:ty>) = $input_value:expr,
        $(state: $state_type:ty | let $state_pat:pat = $state_init:expr,)?
        fn map_leaf <$f:ident $(: $fb:ident $(+ $fbe:ident )*)? $(= $fixed_form:ty)?, $t:ident> ($leaf:ident) -> ($($output_type:tt)+)
        $(where $($where_clause:tt)*)?
        $body:block
    ) => {{
        struct __InlineMapper {
            state: $crate::if_exists!{ {$($state_type)?} {$($state_type)?} {()} },
        }

        $crate::__map_via_leaf_trait_impl!{
            $(@fixed_form $fixed_form |)? impl<$f $(: $fb $(+ $fbe)*)?> @mutability $(&$r $($mut)?)? for __InlineMapper $(where $($where_clause)*)?
            {
                type Output<$($r,)? $a $(: $r)?, $t: $crate::expressions::concepts::IsHierarchicalType> = $($output_type)+;

                fn to_parent_output<$($r,)? $a $(: $r)?, $t: $crate::expressions::concepts::IsChildType>(
                    output: Self::Output<$($r,)? $a, $t>,
                ) -> Self::Output<$($r,)? $a, $t::ParentType> {
                    $crate::__map_via_leaf_to_parent!(output -> $($output_type)+)
                }

                #[allow(unused_mut)]
                fn map_leaf<$($r,)? $a $(: $r)?, $t: $crate::expressions::concepts::IsLeafType>(
                    self,
                    $leaf: $crate::__map_via_leaf_leaf_type!($(@fixed_form $fixed_form |)? $(&$r $($mut)?)? | $f::Leaf<$a, $t>),
                ) -> Self::Output<$($r,)? 'a, T> {
                    $(let $state_pat = self.state;)?
                    $body
                }
            }
        };

        let __mapper = __InlineMapper {
            state: $crate::if_exists!{ {$($state_type)?} {$($state_init)?} {()} },
        };
        $crate::__map_via_leaf_output!(
            <$input_type, $input_form> @mutability $(&$r $($mut)?)? (
                __mapper,
                $input_value
            )
        )
    }};
}

#[doc(hidden)]
macro_rules! __map_via_leaf_trait_impl {
    (impl<$f:ident $(: $fb:ident $(+ $fbe:ident )*)?> @mutability &$r:lifetime mut for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl <$f $(: $fb $(+ $fbe)*)?> MutLeafMapper<$f> for $mapper $(where $($where_clause)*)? { $($body)* } };
    (impl<$f:ident $(: $fb:ident $(+ $fbe:ident )*)?> @mutability &$r:lifetime for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl <$f $(: $fb $(+ $fbe)*)?> RefLeafMapper<$f> for $mapper $(where $($where_clause)*)? { $($body)* } };
    (impl<$f:ident $(: $fb:ident $(+ $fbe:ident )*)?> @mutability for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl <$f $(: $fb $(+ $fbe)*)?> LeafMapper<$f> for $mapper $(where $($where_clause)*)? { $($body)* } };
    (@fixed_form $fixed_form:ty | impl<$f:ident> @mutability &$r:lifetime mut for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl MutLeafMapper<$fixed_form> for $mapper $(where $($where_clause)*)? { $($body)* } };
    (@fixed_form $fixed_form:ty | impl<$f:ident> @mutability &$r:lifetime for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl RefLeafMapper<$fixed_form> for $mapper $(where $($where_clause)*)? { $($body)* } };
    (@fixed_form $fixed_form:ty | impl<$f:ident> @mutability for $mapper:ident $(where $($where_clause:tt)*)? { $($body:tt)* } ) => { impl LeafMapper<$fixed_form> for $mapper $(where $($where_clause)*)? { $($body)* } };
}

#[doc(hidden)]
macro_rules! __map_via_leaf_output {
    (<$input_type:ty, $input_form:ty> @mutability &$r:lifetime mut ($mapper:expr, $input:expr) ) => {
        <$input_type>::map_mut_with::<$input_form, _>($mapper, $input)
    };
    (<$input_type:ty, $input_form:ty> @mutability &$r:lifetime ($mapper:expr, $input:expr) ) => {
        <$input_type>::map_ref_with::<$input_form, _>($mapper, $input)
    };
    (<$input_type:ty, $input_form:ty> @mutability ($mapper:expr, $input:expr) ) => {
        <$input_type>::map_with::<$input_form, _>($mapper, $input)
    };
}

#[doc(hidden)]
macro_rules! __map_via_leaf_leaf_type {
    (@fixed_form $fixed_form:ty | $(&$r:lifetime $($mut:ident)?)? | $f:ident::Leaf<$a:lifetime, $t:ident>) => {
        $(&$r $($mut)?)? <$fixed_form as IsHierarchicalForm>::Leaf<$a, $t>
    };
    ($(&$r:lifetime $($mut:ident)?)? | $f:ident::Leaf<$a:lifetime, $t:ident>) => {
        $(&$r $($mut)?)? <$f>::Leaf<$a, $t>
    };
}

#[doc(hidden)]
macro_rules! __map_via_leaf_to_parent {
    ($output:ident -> Content<$lt:lifetime, $t:ident, $form:ty>) => {
        $t::into_parent($output)
    };
    ($output:ident -> ExecutionResult<Content<$lt:lifetime, $t:ident, $form:ty>>) => {
        Ok($t::into_parent($output?))
    };
    ($output:ident -> Result<Content<$lt:lifetime, $t:ident, $form:ty>, $err:ty>) => {
        Ok($t::into_parent($output?))
    };
    ($output:ident -> AnyLevelCopyOnWrite<$lt:lifetime, $t:ident>) => {
        match $output {
            AnyLevelCopyOnWrite::Owned(owned) => AnyLevelCopyOnWrite::Owned($t::into_parent(owned)),
            AnyLevelCopyOnWrite::SharedWithInfallibleCloning(shared) => {
                AnyLevelCopyOnWrite::SharedWithInfallibleCloning($t::into_parent(shared))
            }
            AnyLevelCopyOnWrite::SharedWithTransparentCloning(shared) => {
                AnyLevelCopyOnWrite::SharedWithTransparentCloning($t::into_parent(shared))
            }
        }
    };
    ($output:ident -> $ty:ty) => {
        $output
    };
}

pub(crate) use {
    __map_via_leaf_leaf_type, __map_via_leaf_output, __map_via_leaf_to_parent,
    __map_via_leaf_trait_impl, map_via_leaf,
};

#[cfg(test)]
mod test_macro {
    use super::*;

    #[test]
    fn test_map_via_leaf_owned() {
        let mut x = 4;
        let owned = map_via_leaf! {
            input: &'r mut (Content<'a, U8Type, BeOwned>) = &mut x,
            fn map_leaf<F: LeafAsRefForm, T>(leaf) -> (Content<'static, T, BeOwned>) {
                F::leaf_clone_to_owned_infallible(leaf)
            }
        };
        assert_eq!(owned, 4);
    }
}
