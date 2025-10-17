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
/// This type must only be implemented on types known to be repr[c] equivalent
/// to their Repr type.
pub unsafe trait SingularGeneric: Sized {
    /// The concrete type that the generic can be transmuted on
    type Repr;

    /// Upcast into the Repr base type.
    ///
    /// This is a zero-cost operation that removes the type parameter, converting
    /// to the standard concrete Repr value.
    #[inline]
    fn upcast(self) -> Self::Repr {
        // in future this can be transmute_unchecked
        unsafe { core::mem::transmute_copy(&core::mem::ManuallyDrop::new(self)) }
    }

    /// Upcast into the Repr base type by ref.
    ///
    /// This is a zero-cost operation that removes the type parameter, converting
    /// to the standard concrete Repr value.
    #[inline]
    fn upcast_ref(&self) -> &Self::Repr {
        unsafe { core::mem::transmute(&self) }
    }

    /// Perform an unchecked cast between singular generics with the same Repr.
    ///
    /// This is a zero-cost operation that changes the type parameter without any
    /// runtime validation. Use with caution - incorrect usage may cause runtime errors.
    #[inline]
    fn cast_unchecked<T: SingularGeneric<Repr = Self::Repr>>(self) -> T {
        // in future this can be transmute_unchecked
        unsafe { core::mem::transmute_copy(&core::mem::ManuallyDrop::new(self)) }
    }

    /// Perform an unchecked ref cast between singular generics with the same Repr.
    ///
    /// This is a zero-cost operation that changes the type parameter without any
    /// runtime validation. Use with caution - incorrect usage may cause runtime errors.
    #[inline]
    fn cast_unchecked_ref<T: SingularGeneric<Repr = Self::Repr>>(&self) -> &T {
        unsafe { core::mem::transmute::<&Self, &T>(self) }
    }
}
