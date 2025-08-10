/// Procedural macros for generating card themes and theme queries from CSS files.
use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, path::PathBuf};
use syn::LitStr;

/// Builds a `CardTheme` enum from the themes found in assets/css/themes.
#[proc_macro]
pub fn build_card_themes(_input: TokenStream) -> TokenStream {
    let metas = collect_themes();
    if metas.is_empty() {
        panic!("No .css themes found in assets/css/themes");
    }

    let variants = metas.iter().map(|m| {
        let ident = &m.variant_ident;
        let doc = &m.doc_lit;
        quote! { #[doc = #doc] #ident }
    });

    let arms = metas.iter().map(|m| {
        let ident = &m.variant_ident;
        let include = &m.include_lit;
        quote! { CardTheme::#ident => include_str!(#include) }
    });

    let enum_doc = LitStr::new(
        "CardTheme is generated from CSS files in assets/css/themes at compile time.",
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        #[doc = #enum_doc]
        #[derive(Clone, Debug)]
        pub enum CardTheme { #( #variants, )* }

        impl CardTheme {
            #[doc = "Returns the CSS content associated with this theme."]
            pub fn load_css(&self) -> &'static str {
                match self { #( #arms, )* }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Builds a `ThemeQuery` enum from the themes found in assets/css/themes.
#[proc_macro]
pub fn build_theme_query(_input: TokenStream) -> TokenStream {
    let metas = collect_themes();
    if metas.is_empty() {
        panic!("No .css themes found in assets/css/themes");
    }

    let variants = metas.iter().map(|m| {
        let ident = &m.variant_ident;
        let doc = &m.doc_lit;
        let rename = &m.rename_lit;
        quote! { #[doc = #doc] #[serde(rename = #rename)] #ident }
    });

    let arms = metas.iter().map(|m| {
        let ident = &m.variant_ident;
        quote! { ThemeQuery::#ident => CardTheme::#ident }
    });

    let enum_doc = LitStr::new(
        "ThemeQuery is generated from CSS files in assets/css/themes; query uses snake_case (kebab-case files).",
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        #[doc = #enum_doc]
        #[derive(Debug, Deserialize)]
        pub enum ThemeQuery { #( #variants, )* }

        impl From<ThemeQuery> for CardTheme {
            fn from(t: ThemeQuery) -> Self {
                match t { #( #arms, )* }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Converts a kebab-case or snake_case string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for ch in s.chars() {
        if ch == '-' || ch == '_' || ch == ' ' {
            capitalize = true;
            continue;
        }
        if capitalize {
            out.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Converts a kebab-case or snake_case stem to a human-friendly title.
fn to_title_from_stem(stem: &str) -> String {
    let mut out = String::new();
    let mut cap = true;
    for ch in stem.chars() {
        let ch = if ch == '-' || ch == '_' { ' ' } else { ch };
        if cap {
            for c in ch.to_uppercase() {
                out.push(c);
            }
            cap = false;
        } else {
            out.push(ch);
        }
        if ch == ' ' {
            cap = true;
        }
    }
    out
}

// Internal metadata describing a discovered theme file
struct ThemeMeta {
    /// PascalCase variant name for the enum
    variant_ident: syn::Ident,
    /// Human-friendly title used in Rustdoc
    doc_lit: LitStr,
    /// Absolute path literal for include_str!
    include_lit: LitStr,
    /// snake_case name used for serde(rename)
    rename_lit: LitStr,
}

/// Collects [ThemeMeta] from the assets/css/themes directory.
/// It will be used to generate the `CardTheme` and `ThemeQuery` enums.
///
/// The themes are expected to be in CSS files named in kebab-case (e.g., `dark-mode.css`).
/// The generated enum variants will be in PascalCase (e.g., `DarkMode`).
/// The serde rename will be in snake_case (e.g., `dark_mode`).
/// The doc comments will be generated from the file names, converting kebab-case to title case (e.g., `Dark Mode`).
fn collect_themes() -> Vec<ThemeMeta> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let themes_dir = PathBuf::from(manifest_dir).join("assets/css/themes");

    let mut out = Vec::new();
    let entries = fs::read_dir(&themes_dir).expect("Failed to read assets/css/themes");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("css") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };

        let variant = to_pascal_case(stem);
        let variant_ident = syn::Ident::new(&variant, proc_macro2::Span::call_site());

        let doc_text = to_title_from_stem(stem);
        let doc_lit = LitStr::new(&doc_text, variant_ident.span());

        let rename = stem.to_ascii_lowercase().replace('-', "_");
        let rename_lit = LitStr::new(&rename, proc_macro2::Span::call_site());

        let abs = path.canonicalize().unwrap_or(path.clone());
        let include_path = abs.to_string_lossy().to_string();
        let include_lit = LitStr::new(&include_path, proc_macro2::Span::call_site());

        out.push(ThemeMeta {
            variant_ident,
            doc_lit,
            include_lit,
            rename_lit,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    // Test-only variant that scans a provided base directory instead of CARGO_MANIFEST_DIR
    fn collect_themes_in_dir(base: &std::path::Path) -> Vec<ThemeMeta> {
        let themes_dir = base.join("assets/css/themes");
        let mut out = Vec::new();
        let entries = fs::read_dir(&themes_dir).expect("Failed to read assets/css/themes");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("css") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };

            let variant = to_pascal_case(stem);
            let variant_ident = syn::Ident::new(&variant, proc_macro2::Span::call_site());

            let doc_text = to_title_from_stem(stem);
            let doc_lit = LitStr::new(&doc_text, variant_ident.span());

            let rename = stem.to_ascii_lowercase().replace('-', "_");
            let rename_lit = LitStr::new(&rename, proc_macro2::Span::call_site());

            let abs = path.canonicalize().unwrap_or(path.clone());
            let include_path = abs.to_string_lossy().to_string();
            let include_lit = LitStr::new(&include_path, proc_macro2::Span::call_site());

            out.push(ThemeMeta {
                variant_ident,
                doc_lit,
                include_lit,
                rename_lit,
            });
        }
        out
    }

    #[test]
    fn fn_to_pascal_case() {
        assert_eq!(to_pascal_case("transparent-blue"), "TransparentBlue");
        assert_eq!(to_pascal_case("dark_mode"), "DarkMode");
        assert_eq!(to_pascal_case("Mixed-CASE_name"), "MixedCaseName");
        assert_eq!(to_pascal_case(" simple "), "Simple");
    }

    #[test]
    fn fn_to_title_from_stem() {
        assert_eq!(to_title_from_stem("transparent-blue"), "Transparent Blue");
        assert_eq!(to_title_from_stem("dark_mode"), "Dark Mode");
        assert_eq!(to_title_from_stem("simple"), "Simple");
    }

    // The following smoke test validates collect_themes() produces at least one entry
    // in this repository's layout. We don't assert exact contents to keep it stable.
    #[test]
    fn fn_collect_themes_smoke() {
        // Build a temporary assets/css/themes directory to scan
        let tmp = tempdir().expect("tempdir");
        let base = tmp.path();
        let themes_dir = base.join("assets/css/themes");
        fs::create_dir_all(&themes_dir).expect("mkdir -p assets/css/themes");

        // Create a sample theme file
        let css_path = themes_dir.join("transparent-blue.css");
        let mut f = File::create(&css_path).expect("create css");
        writeln!(f, ":root {{ --primary: #00f; }}").unwrap();

        let metas = collect_themes_in_dir(base);
        assert_eq!(metas.len(), 1);
        let m = &metas[0];
        assert_eq!(m.variant_ident.to_string(), "TransparentBlue");
        assert_eq!(m.doc_lit.value(), "Transparent Blue");
        assert!(
            m.include_lit.value().ends_with(
                Path::new("assets/css/themes/transparent-blue.css")
                    .to_string_lossy()
                    .as_ref()
            )
        );
        assert_eq!(m.rename_lit.value(), "transparent_blue");
    }
}
