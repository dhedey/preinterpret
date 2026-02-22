//! ## Overview
//! 
//! We wish to define a dynamic reference model which upholds the rules of rust
//! references when those references are active, but which is flexible to the
//! needs of the dynamic preinterpret language.
//! 
//! In other words, it should catch real bugs, with clear error messages, but
//! not get in the way of things you might want to do in a dynamic language.
//! 
//! ## Model
//!
//! Our model is:
//! - When calling a method, all references in arguments must
//!   be in an active mode which upholds the rust reference invariants.
//!   - This ensures that Rust methods can be called without UB
//!   - We do this for user-defined methods to, for consistency, and to allow
//!     sensible interface design which matches expectations.
//! - Inside a user-defined method, we don't really want users caring about
//!   what a variable points to, as it adds a strictness which could be
//!   frustrating without a helpful compiler pointing the way. So,
//!   when variables/references aren't in active use, we relax the invariants
//!   to a weaker "InactiveReference" which upholds validity invariants
//!   (i.e. a reference can't point to a value which isn't of the correct type)
//!   but doesn't maintain mutability-related safety invariants (i.e. a
//!   shared reference does not prevent mutation if it is inactive).
//! 
//! As some examples, it should:
//! - Allow x.a: &mut AnyValue and x.b: &mut AnyValue to exist as active mutable
//!   references at the same time
//! - Prevent x: &mut AnyValue and x.a: &AnyValue from existing as active references
//!   at the same time
//! - Allow x: &mut Integer to mutate x whilst a currently unused variable
//!   y: &Integer exists in preinterpret, but not as an active rust reference.
//!
//! ## Implementation
//!
//! We define a concept of a [`ReferencePath`] with a partial order, roughly
//! signalling a depth/subtyping relation to do with mutable ownership.
//! It is more strictly defined on the ReferencePath docs.
//! 
//! There are three reference kinds, each wraps a `*mut T` for ease, and each
//! has the *validity* invariant that the pointer will always point to memory
//! representing the type `T`:
//! - `InactiveReference<T>`
//!   - Represents an inactive shared/mutable, or disabled mutable during a two-phase borrow
//! - "Active" `Shared<T>`
//!   - Implements `Deref<T>`
//!   - The existence of this grants the following safety-invariants:
//!     - T is immutable
//! - "Active" `Mutable<T>`
//!   - Implemented `DerefMut<T>`
//!   - The existence of this implies the following safety-invariants:
//!     - No other reference can mutate T
//!
//! This gives rise to the following rules:
//! (a) Can't create a Mutable from an InactiveReference if there is:
//!   (a.1) A strictly deeper reference of any kind; which could end up pointing to invalid memory
//!   (a.2) Any equal or shallower active reference; as that could break the other reference's safety invariant
//! (b) Can't create an Shared from an InactiveReference if there is:
//!   (b.1) Any equal or deeper ActiveMutable, as they could break our safety invariant
//!   (b.2) (NOTE: There can't be a strictly shallower ActiveMutable, as that would contravene a.1)
//! (c) Can't map an InactiveReference from A to deeper B if:
//!   (c.1) There is a shallower ActiveMutable than B, which could leave us pointing at invalid memory, as per a.1)
//!   (c.2) (NOTE: There can't be a strictly shallower ActiveMutable than A, as that would contravene a.1 already)
//!
//! Basically, a.1 and a.2 are our two key invariants, and all our actions must preserve them.
//!
//! Okay, but then -- how do we create `&mut x.a` and `&mut x.b`? As follows:
//! - Start with ActiveMutable(x), deactivate to InactiveReference(x)
//! - Duplicate InactiveReference(x), map InactiveReference(x) => InactiveReference(x.a), Activate
//! - Duplicate InactiveReference(x), map InactiveReference(x) => InactiveReference(x.b), Activate

mod referenceable;
mod reference_core;
mod inactive_reference;
mod mutable_reference;
mod shared_reference;

use crate::internal_prelude::*;
use reference_core::*;

pub(crate) use referenceable::*;
pub(crate) use inactive_reference::*;
pub(crate) use mutable_reference::*;
pub(crate) use shared_reference::*;