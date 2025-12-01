use std::collections::HashMap;
use syn::visit_mut::{self, VisitMut};
use syn::{visit::Visit, Ident, Type};

// Remaining generics features
// - can consider re-allowing optional params on import types
// - gathering generics across all param types and return types to auto-apply bounds, not just for self

/// Visitor to replace wasm bindgen generics with their concrete types
/// The concrete type is the default type on the import if specified when it was defined.
struct GenericConcreteValueVisitor<'a> {
    generics: &'a HashMap<&'a Ident, syn::Type>,
}

impl<'a> VisitMut for GenericConcreteValueVisitor<'a> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(type_path) = ty {
            // Handle <T as Trait>::AssocType
            if let Some(qself) = &mut type_path.qself {
                if let Type::Path(qself_path) = &mut *qself.ty {
                    if qself_path.qself.is_none() && qself_path.path.segments.len() == 1 {
                        let ident = &qself_path.path.segments[0].ident;
                        if let Some(concrete) = self.generics.get(ident) {
                            *qself.ty = concrete.clone();
                            return;
                        }
                    }
                }
            }
            // Normal T::...
            if type_path.qself.is_none() && !type_path.path.segments.is_empty() {
                let first_seg = &type_path.path.segments[0];

                if let Some(concrete) = self.generics.get(&first_seg.ident) {
                    if type_path.path.segments.len() == 1 {
                        *ty = concrete.clone();
                    } else {
                        if let Type::Path(concrete_path) = concrete {
                            let remaining: Vec<_> =
                                type_path.path.segments.iter().skip(1).cloned().collect();
                            type_path.path.segments = concrete_path.path.segments.clone();
                            type_path.path.segments.extend(remaining);
                        }
                    }
                    return;
                }
            }
        }
        visit_mut::visit_type_mut(self, ty);
    }
}

/// Helper visitor for generic parameter usage
#[derive(Debug)]
pub struct GenericNameVisitor<'a> {
    name_set_a: &'a Vec<&'a Ident>,
    name_set_b: Option<&'a Vec<&'a Ident>>,
    /// Was a generic parameter in name set A found?
    pub found_a: bool,
    /// Were all generic parameters in name set A reference usage?
    pub a_ref_only: bool,
    /// Was a generic parameter in name set B found?
    pub found_b: bool,
    /// Were all generic parameters in name set B reference usage?
    pub b_ref_only: bool,
}

/// Helper visitor for generic parameter usage
impl<'a> GenericNameVisitor<'a> {
    /// Construct a new generic name visitors with a param search set,
    /// and optionally a second parameter search set.
    pub fn new(name_set_a: &'a Vec<&'a Ident>, name_set_b: Option<&'a Vec<&'a Ident>>) -> Self {
        Self {
            name_set_a,
            name_set_b,
            found_a: false,
            a_ref_only: true,
            found_b: false,
            b_ref_only: true,
        }
    }
}

/// Adds a lifetime parameter to the generics if not already present.
/// Returns the lifetime that was added or found.
pub(crate) fn add_lifetime(generics: &mut syn::Generics, lifetime_name: &str) -> syn::Lifetime {
    let lifetime: syn::Lifetime = syn::parse_str(lifetime_name)
        .unwrap_or_else(|_| panic!("Invalid lifetime name: {}", lifetime_name));
    generics.params.insert(
        0,
        syn::GenericParam::Lifetime(syn::LifetimeParam {
            attrs: vec![],
            lifetime: lifetime.clone(),
            colon_token: None,
            bounds: syn::punctuated::Punctuated::new(),
        }),
    );
    lifetime
}

impl<'a> Visit<'_> for GenericNameVisitor<'a> {
    fn visit_type_reference(&mut self, type_ref: &syn::TypeReference) {
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            if let Some(first_segment) = type_path.path.segments.first() {
                if type_path.path.segments.len() == 1 && first_segment.arguments.is_empty() {
                    if self.name_set_a.contains(&&first_segment.ident) {
                        self.found_a = true;
                        return;
                    }
                    if let Some(name_set_b) = self.name_set_b {
                        if name_set_b.contains(&&first_segment.ident) {
                            self.found_b = true;
                            return;
                        }
                    }
                } else {
                    if self.name_set_a.contains(&&first_segment.ident) {
                        self.found_a = true;
                    }
                    if let Some(name_set_b) = self.name_set_b {
                        if name_set_b.contains(&&first_segment.ident) {
                            self.found_b = true;
                        }
                    }

                    syn::visit::visit_path_arguments(self, &first_segment.arguments);

                    for segment in type_path.path.segments.iter().skip(1) {
                        syn::visit::visit_path_segment(self, segment);
                    }
                    return;
                }
            }
        }

        // For other cases, continue normal visiting
        syn::visit::visit_type_reference(self, type_ref);
    }

    fn visit_path(&mut self, path: &syn::Path) {
        if let Some(first_segment) = path.segments.first() {
            if self.name_set_a.contains(&&first_segment.ident) {
                self.found_a = true;
                self.a_ref_only = false; // This is value usage
            }
            if let Some(name_set_b) = self.name_set_b {
                if name_set_b.contains(&&first_segment.ident) {
                    self.found_b = true;
                    self.b_ref_only = false;
                }
            }
        }

        for segment in &path.segments {
            match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    for arg in &args.args {
                        match arg {
                            syn::GenericArgument::Type(ty) => {
                                syn::visit::visit_type(self, ty);
                            }
                            syn::GenericArgument::AssocType(binding) => {
                                // Don't visit binding.ident, only visit binding.ty
                                syn::visit::visit_type(self, &binding.ty);
                            }
                            _ => {
                                syn::visit::visit_generic_argument(self, arg);
                            }
                        }
                    }
                }
                syn::PathArguments::Parenthesized(args) => {
                    // Handle function syntax like FnMut(T) -> Result<R, JsValue>
                    for input in &args.inputs {
                        syn::visit::visit_type(self, input);
                    }
                    if let syn::ReturnType::Type(_, return_type) = &args.output {
                        syn::visit::visit_type(self, return_type);
                    }
                }
                syn::PathArguments::None => {}
            }
        }
    }
}

/// Obtain the generic parameters and their optional defaults
pub(crate) fn generic_params<'a>(
    generics: &'a syn::Generics,
) -> Vec<(&'a Ident, Option<&'a syn::Type>)> {
    generics
        .type_params()
        .map(|tp| (&tp.ident, tp.default.as_ref()))
        .collect()
}

/// Obtain the generic parameters and their optional defaults
pub(crate) fn generic_param_names<'a>(generics: &'a syn::Generics) -> Vec<&'a Ident> {
    generics.type_params().map(|tp| &tp.ident).collect()
}

pub(crate) fn uses_generic_params(ty: &syn::Type, generic_names: &Vec<&Ident>) -> bool {
    let mut visitor = GenericNameVisitor::new(generic_names, None);
    visitor.visit_type(&ty);
    visitor.found_a
}

/// Renaming visitor for generic bounds
pub(crate) fn generic_bounds_rename<'a>(
    mut predicate: syn::WherePredicate,
    renames: &HashMap<&'a Ident, &'a Ident>,
) -> syn::WherePredicate {
    if renames.is_empty() {
        return predicate;
    }
    let concrete: HashMap<&Ident, syn::Type> = renames
        .iter()
        .map(|(from, to)| (*from, syn::parse_quote!(#to)))
        .collect();
    let mut visitor = GenericConcreteValueVisitor {
        generics: &concrete,
    };
    visitor.visit_where_predicate_mut(&mut predicate);
    predicate
}

/// Concrete type replacement visitor application
pub(crate) fn generic_to_concrete<'a>(
    mut ty: syn::Type,
    generic_names: &HashMap<&'a Ident, Option<&'a syn::Type>>,
) -> syn::Type {
    if generic_names.is_empty() {
        return ty;
    }
    let js_value: syn::Type = syn::parse_quote!(JsValue);
    let concrete: HashMap<&Ident, syn::Type> = generic_names
        .iter()
        .map(|(ident, opt_ty)| (*ident, opt_ty.cloned().unwrap_or_else(|| js_value.clone())))
        .collect();
    let mut visitor = GenericConcreteValueVisitor {
        generics: &concrete,
    };
    visitor.visit_type_mut(&mut ty);
    ty
}

/// Normalizes generics by moving inline trait bounds to where clauses.
/// This makes it easier to hoist bounds during code generation.
pub(crate) fn normalize_generics(generics: &mut syn::Generics) {
    let mut new_predicates =
        syn::punctuated::Punctuated::<syn::WherePredicate, syn::Token![,]>::new();

    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            if !type_param.bounds.is_empty() {
                let ident = &type_param.ident;
                let bounds = type_param.bounds.clone();
                let predicate = syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::parse_quote!(#ident),
                    colon_token: syn::Token![:](proc_macro2::Span::call_site()),
                    bounds,
                });
                new_predicates.push(predicate);
                type_param.bounds.clear();
            }
        }
    }

    if !new_predicates.is_empty() {
        generics
            .make_where_clause()
            .predicates
            .extend(new_predicates);
    }
}

mod tests {
    #[test]
    fn test_generic_name_visitor() {
        let t_ident = syn::Ident::new("T", proc_macro2::Span::call_site());
        let u_ident = syn::Ident::new("U", proc_macro2::Span::call_site());
        let name_set_a = vec![&t_ident];
        let name_set_b = Some(vec![&u_ident]);

        // Test T as value
        let ty: syn::Type = syn::parse_quote!(T);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only);

        // Test &T as reference
        let ty: syn::Type = syn::parse_quote!(&T);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);

        // Test T<U> - T as value, U as value
        let ty: syn::Type = syn::parse_quote!(T<U>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only);
        assert!(visitor.found_b);
        assert!(!visitor.b_ref_only);

        // Test &T<U> - T as reference, U as value
        let ty: syn::Type = syn::parse_quote!(&T<U>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);
        assert!(visitor.found_b);
        assert!(!visitor.b_ref_only);

        // Test T::<U>::Foo - T as value, U as value, Foo ignored
        let ty: syn::Type = syn::parse_quote!(T::<U>::Foo);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only);
        assert!(visitor.found_b);
        assert!(!visitor.b_ref_only);

        // Test Vec<T> - T as value, Vec ignored
        let ty: syn::Type = syn::parse_quote!(Vec<T>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only);
    }

    #[test]
    fn test_associated_type_binding() {
        let t_ident = syn::Ident::new("T", proc_macro2::Span::call_site());
        let u_ident = syn::Ident::new("U", proc_macro2::Span::call_site());
        let name_set_a = vec![&t_ident];
        let name_set_b = Some(vec![&u_ident]);

        // Test SomeTrait<T = U> - should find U (RHS) but NOT T (LHS assoc type name)
        let ty: syn::Type = syn::parse_quote!(SomeTrait<T = U>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(!visitor.found_a); // T is LHS assoc type name, should NOT be counted
        assert!(visitor.found_b); // U is RHS generic parameter, should be counted
        assert!(!visitor.b_ref_only);

        // Test SomeTrait<U = T> - should find T (RHS) but NOT U (LHS assoc type name)
        let ty: syn::Type = syn::parse_quote!(SomeTrait<U = T>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a); // T is RHS generic parameter, should be counted
        assert!(!visitor.a_ref_only);
        assert!(!visitor.found_b); // U is LHS assoc type name, should NOT be counted
    }

    #[test]
    fn test_nested_references() {
        let t_ident = syn::Ident::new("T", proc_macro2::Span::call_site());
        let u_ident = syn::Ident::new("U", proc_macro2::Span::call_site());
        let name_set_a = vec![&t_ident];
        let name_set_b = Some(vec![&u_ident]);

        // Test &T - should be ref
        let ty: syn::Type = syn::parse_quote!(&T);
        let mut visitor = crate::generics::GenericNameVisitor::new(&name_set_a, None);
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);

        // Test &&T - should be ref
        let ty: syn::Type = syn::parse_quote!(&&T);
        let mut visitor = crate::generics::GenericNameVisitor::new(&name_set_a, None);
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);

        // Test &&&T - should be ref
        let ty: syn::Type = syn::parse_quote!(&&&T);
        let mut visitor = crate::generics::GenericNameVisitor::new(&name_set_a, None);
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);

        // Test &T<U> - T should be ref, U should be value
        let ty: syn::Type = syn::parse_quote!(&T<U>);
        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(visitor.a_ref_only);
        assert!(visitor.found_b);
        assert!(!visitor.b_ref_only);
    }

    #[test]
    fn test_mixed_usage() {
        let t_ident = syn::Ident::new("T", proc_macro2::Span::call_site());
        let name_set_a = vec![&t_ident];

        // Test T + &T - should find both, ref_only = false
        let ty: syn::Type = syn::parse_quote!(SomeTrait<Item = T> + OtherTrait<Ref = &T>);
        let mut visitor = crate::generics::GenericNameVisitor::new(&name_set_a, None);
        syn::visit::visit_type(&mut visitor, &ty);
        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only); // Found both ref and value usage
    }

    #[test]
    fn test_complex_reference_with_closure() {
        let t_ident = syn::Ident::new("T", proc_macro2::Span::call_site());
        let r_ident = syn::Ident::new("R", proc_macro2::Span::call_site());
        let name_set_a = vec![&t_ident];
        let name_set_b = Some(vec![&r_ident]);

        let ty: syn::Type = syn::parse_quote!(&Closure<dyn FnMut(T) -> Result<R, JsValue>>);

        let mut visitor =
            crate::generics::GenericNameVisitor::new(&name_set_a, name_set_b.as_ref());
        syn::visit::visit_type(&mut visitor, &ty);

        assert!(visitor.found_a);
        assert!(!visitor.a_ref_only);
        assert!(visitor.found_b);
        assert!(!visitor.b_ref_only);
    }

    #[test]
    fn test_generic_args_to_concrete() {
        use std::collections::HashMap;

        // T -> String replacement
        let t = syn::parse_quote!(T);
        let str = Some(syn::parse_quote!(String));
        let generic_names: HashMap<&syn::Ident, Option<&syn::Type>> = {
            let mut map = HashMap::new();
            map.insert(&t, str.as_ref());
            map
        };

        // T gets replaced with String
        let generic_type: syn::Type = syn::parse_quote!(Promise<T>);
        let result = crate::generics::generic_to_concrete(generic_type, &generic_names);
        let expected: syn::Type = syn::parse_quote!(Promise<String>);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // Mixed: i32 stays, T becomes String
        let mixed_type: syn::Type = syn::parse_quote!(Promise<i32, T>);
        let result = crate::generics::generic_to_concrete(mixed_type, &generic_names);
        let expected: syn::Type = syn::parse_quote!(Promise<i32, String>);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // No generics to replace - unchanged
        let concrete_type: syn::Type = syn::parse_quote!(Promise<i32, bool>);
        let result = crate::generics::generic_to_concrete(concrete_type, &generic_names);
        let expected: syn::Type = syn::parse_quote!(Promise<i32, bool>);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );
    }

    #[test]
    fn test_generic_associated_type_replacement() {
        use std::collections::HashMap;

        let t: syn::Ident = syn::parse_quote!(T);
        let concrete: Option<syn::Type> = Some(syn::parse_quote!(MyConcreteType));
        let generic_names: HashMap<&syn::Ident, Option<&syn::Type>> = {
            let mut map = HashMap::new();
            map.insert(&t, concrete.as_ref());
            map
        };

        // T::DurableObjectStub -> MyConcreteType::DurableObjectStub
        let assoc_type: syn::Type = syn::parse_quote!(T::DurableObjectStub);
        let result = crate::generics::generic_to_concrete(assoc_type, &generic_names);
        let expected: syn::Type = syn::parse_quote!(MyConcreteType::DurableObjectStub);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // Nested: Vec<T::Item> -> Vec<MyConcreteType::Item>
        let nested: syn::Type = syn::parse_quote!(Vec<T::Item>);
        let result = crate::generics::generic_to_concrete(nested, &generic_names);
        let expected: syn::Type = syn::parse_quote!(Vec<MyConcreteType::Item>);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // Complex: WasmRet<<T::Stub as FromWasmAbi>::Abi>
        let complex: syn::Type = syn::parse_quote!(WasmRet<<T::Stub as FromWasmAbi>::Abi>);
        let result = crate::generics::generic_to_concrete(complex, &generic_names);
        let expected: syn::Type =
            syn::parse_quote!(WasmRet<<MyConcreteType::Stub as FromWasmAbi>::Abi>);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // T<Foo> gets fully replaced, args discarded
        let with_args: syn::Type = syn::parse_quote!(T<SomeArg>);
        let result = crate::generics::generic_to_concrete(with_args, &generic_names);
        let expected: syn::Type = syn::parse_quote!(MyConcreteType);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // QSelf: <T::DurableObjectStub as FromWasmAbi>::Abi
        let qself_type: syn::Type = syn::parse_quote!(<T::DurableObjectStub as FromWasmAbi>::Abi);
        let result = crate::generics::generic_to_concrete(qself_type, &generic_names);
        let expected: syn::Type =
            syn::parse_quote!(<MyConcreteType::DurableObjectStub as FromWasmAbi>::Abi);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );

        // QSelf with trait: <T as DurableObject>::DurableObjectStub
        let qself_trait: syn::Type = syn::parse_quote!(<T as DurableObject>::DurableObjectStub);
        let result = crate::generics::generic_to_concrete(qself_trait, &generic_names);
        let expected: syn::Type =
            syn::parse_quote!(<MyConcreteType as DurableObject>::DurableObjectStub);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string()
        );
    }
}
