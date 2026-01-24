# Reflection on types and forms - January 2026

In December 2025 I embarked on a longer than expected journey, to "migrate forms to leaves":

```rust
Shared(AnyValue::X(X::Y(y))) => AnyValue::X(X::Y(Shared(y)))
```

This was motivated by a few thoughts:
* It would be nice for e.g. a "Shared int" to be convertible to/from a "Shared any value"
* It's common knowledge that an `Option(&x)` is more useful than an `&Option(x)`
* The code around defining types and forms was ugly and repetitive and could do with a tidy up
* I wanted to start on methods/functions, for e.g. a `.map(|x| x * x)` but something in my
  early investigation of function variable closures, I realized that this leaf migration would
  be needed... both in an abstract sense, but also when I started investigating a "referenceable"
  abstraction to be shared between variables and closures.

### Original justification in the context of methods

- [ ] Improved Shared/Mutable handling - See the `Better handling of value sub-references` section. The key requirement is we need to allow a `Shared<X>` to map back to a `Shared<Value>`. Moving the "sharedness" to the leaves permits this.
  * A function specifies the bindings of its variables
  * If we have `my_len = |x: &array| x.len()` and invoke it as `my_len(a.b)` then
    when I invoke it, I need to end up with the variable `x := &a.b`
  * This is a problem - if we imagine changing what can be stored in a variable to
    the following, then it's clear that we need some way to have a `SharedValue` which
    has an outer-enum instead of an inner-enum.
  * We also need to think about how things like `IterableValue` works. Perhaps it's like an interface, and so defined via `Box<dyn Iterable>` / `Ref<dyn Iterable>` etc?

I don't fully understand what the problem I saw was now. I think that if `my_len` takes a `x: Shared<ArrayValue>` then to call `x.len()` I need to convert it back to a `Shared<AnyValue>` in order to resolve `x.len()`? Maybe? I'm not too sure.

## The problem

The problem is that this works fine for covariant things like shared references... but not at all for things like assignee / mutable. In particular, consider:

```rust
    fn swap(mut a: AnyValueAssignee, mut b: AnyValueAssignee) -> () {
        core::mem::swap(a.0.deref_mut(), b.0.deref_mut());
    }
```

This requires the thing that can be replaced to be the whole `AnyValue`, not just some leaf content.

In a way, the "everything in the leaf" paradigm corresponds to walking around with `let x: &leaf_type`, restricting the place to only support that particular leaf type.

## The reflection - the type of a place & type annotations

So far places have only ever had one type - `any`. Whilst values have leaf types, the places themselves can be swapped to take `AnyValue`. This is pretty much how most dynamically typed languages work.

If we wanted to support type annotations, they _could_ work like this, by constraining the type the place supports at a structural level...

Or we could save the trouble and always store an `AnyValue` structurally but insert a runtime type assertion. This is a tiny bit less performant at runtime, but keeps the code simpler AND less code = less compile time, which is more important than faster runtime on smaller codebases that only use preinterpet a bit.

### What would a hypothetical structurally type-restricted binding look like?

Sidenote: I don't think we should necessarily support this, for reasons mentioned above. But it would allow us to support something like slices, e.g. `Mutable<[ArrayValue]>`

Consider a structural `x: &mut integer`. Then, in the type system a value would look something like this:

```rust
AnyValueContent::Integer(Mutable(IntegerValueContent::U8(u8)))
```

Essentially what we have is a form where the form wrapper isn't restricted to being at the leaf; rather it can be at any level of the type!

But - how would this be modelled? i.e. how would we model a value which could be structurally type-restricted to be any type

Well it would probably want to look something like: Each parent type having a `Content` enum and `Wrapper` enum, which looks the same, except it has an `AsSelf` variant option as well, i.e.:

```rust
// &mut integer
AnyValueWrapper::Integer(IntegerValueWrapper::AsSelf(Mutable(IntegerValueContent::U8(u8))))
// &mut u8
AnyValueWrapper::Integer(IntegerValueWrapper::U8(Mutable(u8)))
// &mut any
AnyValueWrapper::AsSelf(Mutable(AnyValueContent::Integer(IntegerValueContent::U8(u8))))
```

Essentially, outside of the `Form` we have `Wrapper` and inside the form we have `Mutable`.

But then - how do we actually operate on this?? What operations for we supporty?

Honestly, it's really hard. Would need to think about what operations can be performed etc. Even defining a leaf mapper was nigh-on impossible.

Might come back to this at some point.

## What's next?

Let's put the "concepts" to-leaf on indefinite pause. It's definitely resulted in a lot of good cleaning up, BUT it's also created a lot of mess.

Eventually we will need to clean it up:
* Better abstract the `IsArgument` impl
* Finish migrating from `bindings.rs`
* Finish removing `into_any_value()` etc
* Finish clearing up `arguments.rs` and `outputs.rs`

We might well be left with only `BeOwned` and `BeRef`, `BeMut` - we'll see.

See the below half-baked todo list for where this got to.

---

But *first* I'd like to implement functions _without_ type annotations. Then if we add them, it would probably be easier for them to be runtime assertions rather than structural.

## Half-finished to-do lists copied over (TODO: clean these up!)

### Better handling of value sub-references

- [x] Migrate from `Owned` having a span to a `Spanned<Owned>`
- [ ] Implement and roll-out GATs
  - [x] Initial shell implementation in `concepts` folder
  - [x] Improved error handling in the macro (e.g. required arg after optional; no matching strongly-typed signature)
  - [x] Separate Hierarchical and DynCompatible Forms
  - [x] Improved macro support
  - [x] Add source type name to type macro/s
  - [x] Generate value kinds from the macros
    - [x] Add (temporary) ability to link to TypeData and resolve methods from there
    - [x] Then implement all the macros
    - [x] And use that to generate AnyValueLeafKind from the new macros
    - [x] Find a way to generate `from_source_name` - ideally efficiently
  - [x] Add ability to implement IsIterable
  - [x] `CastTarget` simply wraps `TypeKind`
  - [x] Create a `Ref` and a `Mut` form
    - [x] They won't implement `IsArgumentForm`
    - [x] Create mappers and suitably generic `as_ref()` and `as_mut()` methods on `Actual<F>`
  - [x] Migrate method resolution to the trait macro properly
    - [x] And property resolution `dyn TypeFeatureResolver`
    - [x] Replace `impl TypeFeatureResolver for $type_def`
    - [x] Remove `temp_type_data: $type_data:ident,`
    - [x] Remove `HierarchicalTypeData`
  - [ ] Stage 1 of the form migration:
    - [x] Add temp blanket impl from `IntoValue` for `IntoValueContent`
    - [x] Get rid of `BooleanValue` wrapper
    - [x] Get rid of `StreamValue` wrapper
    - [x] Get rid of `CharValue` wrapper
    - [x] Replace `IntegerValue` with `type IntegerValue = IntegerContent<'static, BeOwned>`
    - [x] Replace `FloatValue` with `type FloatValue = FloatContent<'static, BeOwned>`
    - [x] Replace `Value` with `type Value = ValueContent<'static, BeOwned>`
    - [x] Get rid of the `Actual` wrapper inside content
    // ---
    - [x] Trial getting rid of `Actual` completely?
    - [x] Replace `type FloatValue = FloatValueContent` with `type FloatValue = QqqOwned<FloatValueType>` / `type FloatValueRef<'a> = QqqRef<FloatValueType>` / `type FloatValueMut = QqqMut<FloatValueType>`
      - [x] Create new branch
      - [x] Resolve issue with `2.3` not resolving into `2f32` any more
    - [x] .. same for int...
    - [x] Update Value:
      - [x] `ValueType` => `AnyType`
      - [x] `type AnyValue = QqqOwned<ValueType>`
      - [x] `type AnyValueRef = QqqRef<ValueType>`
      - [x] `type AnyValueMut = QqqMut<ValueType>`
      - [x] ... and move methods
    - [x] Remove `OwnedValue`
    - [x] Get rid of `Owned`
  - [ ] Stage 2 of the form migration:
    - [x] Attempt to improve mappers:
      - [x] Try to replace `ToRefMapper` etc with a `FormMapper::<T, F1, F2>::map_content(content, |x| -> y)` - 6 methods... `map_content`, `map_content_ref`, `map_content_mut` and `try_x` *3; ... and a `ReduceMapper::<F1, T>::map(content, |x| -> y)`... sadly not possible! The lambda needs to be higher-ordered and work for all `L: IsValueLeaf`.
    - [x] Finish mapper improvements
      - [x] Migrate `self`
      - [x] Consider if we even need `MapperOutput` or just some helper functions
      - [x] Delay resolving the type kind into the error case when downcasting
    - [x] Create macro to define inline mappers in various forms
    - [x] Attempt to see if I can get rid of needing `leaf_to_content` and maybe `content_to_leaf` by exploiting a trick to bind associated types via a sub-trait (e.g. as I did with `IsValueContent<Type = <Self as IsLeafValueContent>::LeafType> + IsLeafValueContent`)
    - [x] Pivot the argument resolution to work with either old or new values
    - [ ] Remove as much from arguments.rs as possible
    - [ ] Fix `todo!("Argument")`
    - [ ] Pivot the return resolution to work with either old or new values
    - [ ] Remove `ResolveAs`
    - [ ] Look at better implementations of `FromArgument`
          .. potentially via some new selector trait `IsResolvable` with a resolution strategy of `Hierarchichal` | `Dyn` | `Custom`
          This will let us implement a more general resolution logic, and amalgamate argument parsing and downcast_resolve/dyn_resolve
    - [ ] Reproduce `CopyOnWrite`
      - [ ] Implement `TODO[concepts]: COPY ON WRITE MAPPING`
      - [ ] `BeCopyOnWrite::owned()` / `shared_as_..` can go via `AnyLevelCopyOnWrite => into_copy_on_write`
    - [ ] Reproduce `Shared` methods etc
    - [ ] Reproduce `Mutable` methods etc
    - [ ] Reproduce `Assignee` methods etc
    - [ ] Change `CopyOnWrite<AnyValue>` => `CopyOnWriteAnyValue` similarly with `Shared`, `Mutable` and `Assignee`
    - [ ] Migrate `CopyOnWrite`, `Shared`, `Mutable`, `Assignee`
      - [ ] `Shared<X>` only works for leaves
      - [ ] For specific parents, use e.g. `AnyValueShared`
      - [ ] If you need something to apply across all leaves, implement it on some trait
      `IsSelfSharedContent` depending/auto-implemented on `IsSelfValueContent<Form = BeShared>`
      - [ ] Same for `Mutable` and `Assignee`
  - [ ] Stage 3
    - [ ] Migrate `CopyOnWrite` and relevant interconversions
    - [ ] Finish migrating to new Argument/Returned resolution and delete old code
    - [ ] Remove `impl_resolvable_argument_for` and `TODO[concepts]: Remove when we get rid of impl_resolvable_argument_for`
  - [ ] Complete ownership definitions and inter-conversions, including maybe-erroring inter-conversions (possibly with `Result<X, Mapper::Error>` which we can map out of):
    - [ ] Consider migrating LateBound and interconversions
    - [ ] Argument, and `ArgumentOwnership` driven conversions into it
  - [ ] Generate test over all value kinds which checks for:
    - [ ] Maybe over TypeKinds with https://docs.rs/inventory/latest/inventory/ registered as a dev dependency
    - [ ] Check for duplicate TypeKind registrations
    - [ ] Source type has no spaces and is lower case, and is invertible
    - [ ] Ancestor types agree with type kinds
  - [ ] Clear up all `TODO[concepts]`
  - [ ] Have variables store a `Referenceable`
  - [ ] Separate methods and functions
  - [ ] Add ability to add comments to types, methods/functions and operations, and generate docs from them
  - [ ] Clean-up:
    - [ ] Move indexing and property access (see e.g.`into_indexed`) etc to the type resolution system - this map allow us to remove
    things like `AnyLevelCopyOnWrite` and the `TypeMapper`
    - [ ] Most matching methods on `AnyValue` would be better as leaf methods