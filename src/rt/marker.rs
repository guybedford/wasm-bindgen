/// Marker trait for types that support `#[wasm_bindgen(constructor)]`.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "JavaScript constructors are not supported for `{Self}`",
        label = "this function cannot be the constructor of `{Self}`",
        note = "`#[wasm_bindgen(constructor)]` is only supported for `struct`s and cannot be used for `enum`s.",
        note = "Consider removing the `constructor` option and using a regular static method instead."
    )
)]
pub trait SupportsConstructor {}
pub struct CheckSupportsConstructor<T: SupportsConstructor>(T);

/// Marker trait for types that support `#[wasm_bindgen(getter)]` or
/// `#[wasm_bindgen(Setter)]` on instance methods.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "JavaScript instance getters and setters are not supported for `{Self}`",
        label = "this method cannot be a getter or setter for `{Self}`",
        note = "`#[wasm_bindgen(getter)]` and `#[wasm_bindgen(setter)]` are only supported for `struct`s and cannot be used for `enum`s.",
    )
)]
pub trait SupportsInstanceProperty {}
pub struct CheckSupportsInstanceProperty<T: SupportsInstanceProperty>(T);

/// Marker trait for types that support `#[wasm_bindgen(getter)]` or
/// `#[wasm_bindgen(Setter)]` on static methods.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "JavaScript static getters and setters are not supported for `{Self}`",
        label = "this static function cannot be a static getter or setter on `{Self}`",
        note = "`#[wasm_bindgen(getter)]` and `#[wasm_bindgen(setter)]` are only supported for `struct`s and cannot be used for `enum`s.",
    )
)]
pub trait SupportsStaticProperty {}
pub struct CheckSupportsStaticProperty<T: SupportsStaticProperty>(T);

/// Marker trait for singular generics - types with this trait have the same
/// repr for all generic param values, and can therefore be transmuted on
/// the singular Repr type representation on ABI boundaries.
///
/// # Safety
/// This type must only be implemented on types known to be repr equivalent
/// to their Repr type.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "The generic type has been defined with a default concrete value that does not support being used as a generic",
        label = "parameter default is not a valid generic type",
        note = "Wasm Bindgen type-erasure generics only supports certain base repr types like JsValue",
    )
)]
pub unsafe trait SingularGeneric {
    /// The singular concrete type that all generic variants can be transmuted on
    type Repr;
}

#[diagnostic::do_not_recommend]
unsafe impl<'a, T: SingularGeneric> SingularGeneric for &'a mut T {
    type Repr = &'a mut T::Repr;
}

#[diagnostic::do_not_recommend]
unsafe impl<'a, T: SingularGeneric> SingularGeneric for &'a T {
    type Repr = &'a T::Repr;
}

/// Trait bound marker for types that are passed as an own generic type.
/// Encapsulating the singular generic invariant that must be maintained,
/// that the SingularGeneric property that the repr of the type is the type
/// of the concrete target type repr.
/// This is useful to provide simple debug output for generic bounds for
/// generated import bindgen code using generics.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "Unable to call function, since the concrete generic argument or return value cannot be type-erased into the expected generic repr type for the function",
        label = "passed concrete generic type does not match the expected generic repr type",
        note = "Wasm Bindgen generic parameters and return values for functions are defined to work for specific type-erasable generic types only",
    )
)]
pub trait GenericOwn<ConcreteTarget>: SingularGeneric {}

#[diagnostic::do_not_recommend]
impl<T, ConcreteTarget> GenericOwn<ConcreteTarget> for T
where
    ConcreteTarget: SingularGeneric,
    T: SingularGeneric<Repr = <ConcreteTarget as SingularGeneric>::Repr>,
{
}

/// Trait bound marker for types that are passed as a borrowed generic type.
/// Encapsulating the singular generic invariant that must be maintained,
/// that the SingularGeneric property that the repr of the type is the type
/// of the concrete target type repr.
/// This is useful to provide simple debug output for generic bounds for
/// generated import bindgen code using generics.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "Unable to call this function, since the concrete generic argument or return value cannot be type-erased into the expected generic repr type for the function",
        label = "concrete generic type does not match the expected generic repr type",
        note = "Wasm Bindgen generic parameters and return values for functions are defined to work for specific type-erasable generic types only",
    )
)]
pub trait GenericBorrow<ConcreteTarget: ?Sized> {}

#[diagnostic::do_not_recommend]
impl<'a, T: ?Sized, ConcreteTarget: ?Sized> GenericBorrow<ConcreteTarget> for T
where
    &'a ConcreteTarget: SingularGeneric + 'a,
    &'a T: SingularGeneric<Repr = <&'a ConcreteTarget as SingularGeneric>::Repr> + 'a,
{
}

/// Trait bound marker for types that are passed as a mutable borrowed generic type.
/// Encapsulating the singular generic invariant that must be maintained,
/// that the SingularGeneric property that the repr of the type is the type
/// of the concrete target type repr.
/// This is useful to provide simple debug output for generic bounds for
/// generated import bindgen code using generics.
#[cfg_attr(
    wbg_diagnostic,
    diagnostic::on_unimplemented(
        message = "Unable to call this function, since the concrete generic argument or return value cannot be type-erased into the expected generic repr type for the function",
        label = "concrete generic type does not match the expected generic repr type",
        note = "Wasm Bindgen generic parameters and return values for functions are defined to work for specific type-erasable generic types only",
    )
)]
pub trait GenericBorrowMut<Target: ?Sized> {}

#[diagnostic::do_not_recommend]
impl<'a, T: ?Sized, ConcreteTarget: ?Sized> GenericBorrowMut<ConcreteTarget> for T
where
    &'a mut ConcreteTarget: SingularGeneric + 'a,
    &'a mut T: SingularGeneric<Repr = <&'a mut ConcreteTarget as SingularGeneric>::Repr> + 'a,
{
}

/// Generic trait bound indicating that the given type
/// can be type erased into the parameter type repr
///
/// Note: generic types are restricted, this trait can only used in trait bounds.
pub trait Generic<T>: SingularGeneric<Repr = T> {}
impl<T: SingularGeneric> Generic<T::Repr> for T {}
