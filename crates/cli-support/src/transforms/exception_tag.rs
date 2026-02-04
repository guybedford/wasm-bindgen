//! Transform to convert imported exception tags to local tags.
//!
//! The Rust compiler emits `(import "env" "__cpp_exception" (tag ...))` when using
//! the WebAssembly exception handling feature. However, browsers don't provide this
//! tag in the "env" module, causing instantiation to fail.
//!
//! This transform finds such imported tags and converts them to locally defined tags
//! that are then exported, so the JavaScript glue code or host environment can access
//! them if needed.

use walrus::{ImportKind, Module, TagKind};

/// Check if the module has any exception tags, indicating unwind support is enabled.
pub fn has_exception_tags(module: &Module) -> bool {
    module.tags.iter().next().is_some()
}

/// Convert imported exception tags from the "env" module to local tags.
///
/// This specifically targets the `__cpp_exception` tag that Rust/LLVM emits
/// as an import, but could be extended to handle other tags if needed.
///
/// Returns `true` if any tags were converted, indicating the module uses
/// exception handling / unwinding.
pub fn run(module: &mut Module) -> bool {
    // Collect tag imports from "env" module that need to be converted
    let tags_to_convert: Vec<_> = module
        .imports
        .iter()
        .filter_map(|import| {
            if import.module == "env" {
                if let ImportKind::Tag(tag_id) = import.kind {
                    return Some((import.id(), tag_id, import.name.clone()));
                }
            }
            None
        })
        .collect();

    let has_tags = !tags_to_convert.is_empty();

    // Convert each imported tag to a local tag
    for (import_id, tag_id, name) in tags_to_convert {
        // Delete the import
        module.imports.delete(import_id);

        // Update the tag to be local instead of imported
        let tag = module.tags.get_mut(tag_id);
        tag.kind = TagKind::Local;

        // Export the tag so it can be accessed from JavaScript if needed
        module.exports.add(&name, tag_id);
    }

    has_tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_imported_tag_to_local() {
        // Create a module with an imported tag from "env"
        let mut module = Module::default();

        // Create a function type for the tag (tags use function types for their signature)
        let ty = module.types.add(&[walrus::ValType::I32], &[]);

        // Add an imported tag
        let (tag_id, import_id) = module.add_import_tag("env", "__cpp_exception", ty);

        // Verify the tag is imported
        assert!(matches!(module.tags.get(tag_id).kind, TagKind::Import(_)));
        assert!(module.imports.get(import_id).module == "env");

        // Verify has_exception_tags detects the tag
        assert!(has_exception_tags(&module));

        // Run the transform
        let converted = run(&mut module);

        // Verify it returned true (tags were converted)
        assert!(converted);

        // Verify the import is gone
        assert!(module.imports.find("env", "__cpp_exception").is_none());

        // Verify the tag is now local
        assert!(matches!(module.tags.get(tag_id).kind, TagKind::Local));

        // Verify the tag is exported
        let export = module.exports.iter().find(|e| e.name == "__cpp_exception");
        assert!(export.is_some());

        // Verify has_exception_tags still detects the (now local) tag
        assert!(has_exception_tags(&module));
    }

    #[test]
    fn test_non_env_import_unchanged() {
        // Create a module with an imported tag from a different module
        let mut module = Module::default();
        let ty = module.types.add(&[walrus::ValType::I32], &[]);
        let (tag_id, _import_id) = module.add_import_tag("other", "some_tag", ty);

        // Run the transform
        let converted = run(&mut module);

        // Verify it returned false (no tags from "env" were converted)
        assert!(!converted);

        // Verify the tag is still imported (not from "env", so unchanged)
        assert!(matches!(module.tags.get(tag_id).kind, TagKind::Import(_)));
        assert!(module.imports.find("other", "some_tag").is_some());
    }

    #[test]
    fn test_no_tags() {
        let module = Module::default();
        assert!(!has_exception_tags(&module));
    }
}
