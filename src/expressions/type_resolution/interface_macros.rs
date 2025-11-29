use super::*;

macro_rules! ignore_all {
    ($($_:tt)*) => {};
}

macro_rules! if_empty {
    ([] [$($output:tt)*]) => {
        $($output)*
    };
    ([$($input:tt)+] [$($output:tt)*]) => {
        $($input)*
    };
}

macro_rules! handle_first_arg_type {
    // By value
    ($($arg_part:ident)+ : $type:ty, $($rest:tt)*) => {
        $type
    };
}

macro_rules! create_method_interface {
    ($method_name:path[$(,)?]) => {
        MethodInterface::Arity0 {
            method: |context| apply_fn0($method_name, context),
            argument_ownership: [],
        }
    };
    ($method_name:path[$($arg_part:ident)+ : $ty:ty $(,)?]) => {
        MethodInterface::Arity1 {
            method: |context, a| apply_fn1($method_name, a, context),
            argument_ownership: [<$ty as FromResolved>::OWNERSHIP],
        }
    };
    ($method_name:path[
        $($arg_part1:ident)+ : $ty1:ty,
        $($arg_part2:ident)+ : Option<$ty2:ty> $(,)?
    ]) => {
        MethodInterface::Arity1PlusOptional1 {
            method: |context, a, b| apply_fn1_optional1($method_name, a, b, context),
            argument_ownership: [
                <$ty1 as FromResolved>::OWNERSHIP,
                <$ty2 as FromResolved>::OWNERSHIP,
            ],
        }
    };
    ($method_name:path[
        $($arg_part1:ident)+ : $ty1:ty,
        $($arg_part2:ident)+ : $ty2:ty $(,)?
    ]) => {
        MethodInterface::Arity2 {
            method: |context, a, b| apply_fn2($method_name, a, b, context),
            argument_ownership: [
                <$ty1 as FromResolved>::OWNERSHIP,
                <$ty2 as FromResolved>::OWNERSHIP,
            ],
        }
    };
    ($method_name:path[
        $($arg_part1:ident)+ : $ty1:ty,
        $($arg_part2:ident)+ : $ty2:ty,
        $($arg_part3:ident)+ : Option<$ty3:ty> $(,)?
    ]) => {
        MethodInterface::Arity2PlusOptional1 {
            method: |context, a, b, c| apply_fn2_optional1($method_name, a, b, c, context),
            argument_ownership: [
                <$ty1 as FromResolved>::OWNERSHIP,
                <$ty2 as FromResolved>::OWNERSHIP,
                <$ty3 as FromResolved>::OWNERSHIP,
            ],
        }
    };
    ($method_name:path[
        $($arg_part1:ident)+ : $ty1:ty,
        $($arg_part2:ident)+ : $ty2:ty,
        $($arg_part3:ident)+ : $ty3:ty $(,)?
    ]) => {
        MethodInterface::Arity3 {
            method: |context, a, b, c| apply_fn3($method_name, a, b, c, context),
            argument_ownership: [
                <$ty1 as FromResolved>::OWNERSHIP,
                <$ty2 as FromResolved>::OWNERSHIP,
                <$ty3 as FromResolved>::OWNERSHIP,
            ],
        }
    };
    ($method_name:path[
        $($arg_part1:ident)+ : $ty1:ty,
        $($arg_part2:ident)+ : $ty2:ty,
        $($arg_part3:ident)+ : $ty3:ty,
        $($arg_part4:ident)+ : Option<$ty4:ty> $(,)?
    ]) => {
        MethodInterface::Arity3PlusOptional1 {
            method: |context, a, b, c, d| apply_fn3_optional1($method_name, a, b, c, d, context),
            argument_ownership: [
                <$ty1 as FromResolved>::OWNERSHIP,
                <$ty2 as FromResolved>::OWNERSHIP,
                <$ty3 as FromResolved>::OWNERSHIP,
                <$ty4 as FromResolved>::OWNERSHIP,
            ],
        }
    };
}

// NOTE: We use function pointers here rather than generics to avoid monomorphization bloat.
// This means that we only need to compile the mapping glue combination once for each (A, B) -> C combination

#[allow(unused)]
pub(crate) fn apply_fn0<R>(
    f: fn(&mut MethodCallContext) -> R,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(context).to_resolved_value(output_span_range)
}

pub(crate) fn apply_fn1<A, R>(
    f: fn(&mut MethodCallContext, A) -> R,
    a: ResolvedValue,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(context, A::from_resolved(a)?).to_resolved_value(output_span_range)
}

#[allow(unused)]
pub(crate) fn apply_fn1_optional1<A, B, C>(
    f: fn(&mut MethodCallContext, A, Option<B>) -> C,
    a: ResolvedValue,
    b: Option<ResolvedValue>,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    C: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(
        context,
        A::from_resolved(a)?,
        b.map(|b| B::from_resolved(b)).transpose()?,
    )
    .to_resolved_value(output_span_range)
}

pub(crate) fn apply_fn2<A, B, C>(
    f: fn(&mut MethodCallContext, A, B) -> C,
    a: ResolvedValue,
    b: ResolvedValue,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    C: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(context, A::from_resolved(a)?, B::from_resolved(b)?).to_resolved_value(output_span_range)
}

pub(crate) fn apply_fn2_optional1<A, B, C, D>(
    f: fn(&mut MethodCallContext, A, B, Option<C>) -> D,
    a: ResolvedValue,
    b: ResolvedValue,
    c: Option<ResolvedValue>,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    C: FromResolved,
    D: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(
        context,
        A::from_resolved(a)?,
        B::from_resolved(b)?,
        c.map(|c| C::from_resolved(c)).transpose()?,
    )
    .to_resolved_value(output_span_range)
}

#[allow(unused)]
pub(crate) fn apply_fn3<A, B, C, R>(
    f: fn(&mut MethodCallContext, A, B, C) -> R,
    a: ResolvedValue,
    b: ResolvedValue,
    c: ResolvedValue,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    C: FromResolved,
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(
        context,
        A::from_resolved(a)?,
        B::from_resolved(b)?,
        C::from_resolved(c)?,
    )
    .to_resolved_value(output_span_range)
}

pub(crate) fn apply_fn3_optional1<A, B, C, D, R>(
    f: fn(&mut MethodCallContext, A, B, C, Option<D>) -> R,
    a: ResolvedValue,
    b: ResolvedValue,
    c: ResolvedValue,
    d: Option<ResolvedValue>,
    context: &mut MethodCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    C: FromResolved,
    D: FromResolved,
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(
        context,
        A::from_resolved(a)?,
        B::from_resolved(b)?,
        C::from_resolved(c)?,
        d.map(|d| D::from_resolved(d)).transpose()?,
    )
    .to_resolved_value(output_span_range)
}

macro_rules! create_unary_interface {
    ($method_name:path[$($arg_part:ident)+ : $ty:ty $(,)?]) => {
        UnaryOperationInterface {
            method: |context, a| apply_unary_fn($method_name, a, context),
            argument_ownership: <$ty as FromResolved>::OWNERSHIP,
        }
    };
}

pub(crate) fn apply_unary_fn<A, R>(
    f: fn(UnaryOperationCallContext, A) -> R,
    a: ResolvedValue,
    context: UnaryOperationCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(context, A::from_resolved(a)?).to_resolved_value(output_span_range)
}

macro_rules! create_binary_interface {
    ($method_name:path[
        $($lhs_part:ident)+ : $lhs_ty:ty,
        $($rhs_part:ident)+ : $rhs_ty:ty $(,)?
    ]) => {
        BinaryOperationInterface {
            method: |context, lhs, rhs| apply_binary_fn($method_name, lhs, rhs, context),
            lhs_ownership: <$lhs_ty as FromResolved>::OWNERSHIP,
            rhs_ownership: <$rhs_ty as FromResolved>::OWNERSHIP,
        }
    };
}

pub(crate) fn apply_binary_fn<A, B, R>(
    f: fn(BinaryOperationCallContext, A, B) -> R,
    lhs: ResolvedValue,
    rhs: ResolvedValue,
    context: BinaryOperationCallContext,
) -> ExecutionResult<ResolvedValue>
where
    A: FromResolved,
    B: FromResolved,
    R: ResolvableOutput,
{
    let output_span_range = context.output_span_range;
    f(context, A::from_resolved(lhs)?, B::from_resolved(rhs)?).to_resolved_value(output_span_range)
}

pub(crate) struct MethodCallContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub output_span_range: SpanRange,
}

impl<'a> HasSpanRange for MethodCallContext<'a> {
    fn span_range(&self) -> SpanRange {
        self.output_span_range
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UnaryOperationCallContext<'a> {
    pub operation: &'a UnaryOperation,
    pub output_span_range: SpanRange,
}

#[derive(Clone, Copy)]
pub(crate) struct BinaryOperationCallContext<'a> {
    pub operation: &'a BinaryOperation,
    pub output_span_range: SpanRange,
}

impl<'a> BinaryOperationCallContext<'a> {
    #[allow(unused)]
    pub(crate) fn err<T>(&self, message: impl std::fmt::Display) -> ExecutionResult<T> {
        self.operation.value_err(message)
    }

    pub(crate) fn error(&self, message: impl std::fmt::Display) -> ExecutionInterrupt {
        self.operation.value_error(message)
    }
}

macro_rules! define_interface {
    (
        struct $type_data:ident,
        parent: $parent_type_data:ident,
        $mod_vis:vis mod $mod_name:ident {
            $mod_methods_vis:vis mod methods {
                $(
                    $([$method_context:ident])? fn $method_name:ident($($method_args:tt)*) $(-> $method_output_ty:ty)? $([ignore_type_assertion $method_ignore_type_assertion:tt])? $method_body:block
                )*
            }
            $mod_unary_operations_vis:vis mod unary_operations {
                $(
                    $([$unary_context:ident])? fn $unary_name:ident($($unary_args:tt)*) $(-> $unary_output_ty:ty)? $([ignore_type_assertion $unary_ignore_type_assertion:tt])? $unary_body:block
                )*
            }
            $mod_binary_operations_vis:vis mod binary_operations {
                $(
                    $([$binary_context:ident])? fn $binary_name:ident($($binary_args:tt)*) $(-> $binary_output_ty:ty)? $([ignore_type_assertion $binary_ignore_type_assertion:tt])? $binary_body:block
                )*
            }
            interface_items {
                $($items:item)*
            }
        }
    ) => {
        #[derive(Clone, Copy)]
        pub(crate) struct $type_data;

        $mod_vis mod $mod_name {
            use super::*;

            $mod_vis const fn parent() -> Option<$parent_type_data> {
                // Type ids aren't const, and strings aren't const-comparable, but I can use this work-around:
                // https://internals.rust-lang.org/t/why-i-cannot-compare-two-static-str-s-in-a-const-context/17726/2
                const OWN_TYPE_NAME: &'static [u8] = stringify!($type_data).as_bytes();
                const PARENT_TYPE_NAME: &'static [u8] = stringify!($parent_type_data).as_bytes();
                match PARENT_TYPE_NAME {
                    OWN_TYPE_NAME => None,
                    _ => Some($parent_type_data),
                }
            }

            #[allow(unused)]
            fn asserts() {
                $(
                    $type_data::assert_first_argument::<handle_first_arg_type!($($method_args)*,)>();
                    if_exists! {
                        {$($method_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$method_output_ty>();)?}
                    }
                )*
                $(
                    $type_data::assert_first_argument::<handle_first_arg_type!($($unary_args)*,)>();
                    if_exists! {
                        {$($unary_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$unary_output_ty>();)?}
                    }
                )*
                $(
                    $type_data::assert_first_argument::<handle_first_arg_type!($($binary_args)*,)>();
                    if_exists! {
                        {$($binary_ignore_type_assertion)?}
                        {}
                        {$($type_data::assert_output_type::<$binary_output_ty>();)?}
                    }
                )*
            }

            $mod_methods_vis mod methods {
                #[allow(unused)]
                use super::*;
                $(
                    pub(crate) fn $method_name(if_empty!([$($method_context)?][_context]): &mut MethodCallContext, $($method_args)*) $(-> $method_output_ty)? {
                        $method_body
                    }
                )*
            }

            $mod_methods_vis mod method_definitions {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_methods_vis fn $method_name() -> MethodInterface {
                        create_method_interface!(methods::$method_name[$($method_args)*])
                    }
                )*
            }

            $mod_unary_operations_vis mod unary_operations {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_unary_operations_vis fn $unary_name(if_empty!([$($unary_context)?][_context]): UnaryOperationCallContext, $($unary_args)*) $(-> $unary_output_ty)? {
                        $unary_body
                    }
                )*
            }

            $mod_unary_operations_vis mod unary_definitions {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_unary_operations_vis fn $unary_name() -> UnaryOperationInterface {
                        create_unary_interface!(unary_operations::$unary_name[$($unary_args)*])
                    }
                )*
            }

            $mod_binary_operations_vis mod binary_operations {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_binary_operations_vis fn $binary_name(if_empty!([$($binary_context)?][_context]): BinaryOperationCallContext, $($binary_args)*) $(-> $binary_output_ty)? {
                        $binary_body
                    }
                )*
            }

            $mod_binary_operations_vis mod binary_definitions {
                #[allow(unused)]
                use super::*;
                $(
                    $mod_binary_operations_vis fn $binary_name() -> BinaryOperationInterface {
                        create_binary_interface!(binary_operations::$binary_name[$($binary_args)*])
                    }
                )*
            }

            impl HierarchicalTypeData for $type_data {
                type Parent = $parent_type_data;
                const PARENT: Option<Self::Parent> = $mod_name::parent();

                #[allow(unreachable_code)]
                fn resolve_own_method(method_name: &str) -> Option<MethodInterface> {
                    Some(match method_name {
                        $(
                            stringify!($method_name) => method_definitions::$method_name(),
                        )*
                        _ => return None,
                    })
                }

                // Pass through resolve_own_unary_operation and resolve_own_binary_operation
                // until there's a better way to define them
                $($items)*
            }
        }
    }
}

pub(crate) use {
    create_binary_interface, create_method_interface, create_unary_interface, define_interface,
    handle_first_arg_type, if_empty, ignore_all,
};
