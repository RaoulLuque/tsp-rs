#[doc(hidden)]
// pub use paste::paste;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitInt, parse_macro_input};

#[proc_macro]
pub fn test_fn_on_instances_filtered(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input with parse_macro_input_args);
    let (fn_name, test_name, lower_bound, upper_bound) = input;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let instances_dir = std::path::Path::new(&manifest_dir).join("../../instances");

    let mut test_functions = Vec::new();

    // Iterate over subdirectories in instances/
    let instances_contents = std::fs::read_dir(&instances_dir).unwrap();
    for entry in instances_contents.flatten() {
        let subdir_path = entry.path();
        let subdir_name = subdir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Iterate over .tsp files in current subdirectory
        let subdir_contents = std::fs::read_dir(&subdir_path).unwrap();
        for tsp_file in subdir_contents.flatten() {
            let file_path = tsp_file.path();
            if let Some(extension) = file_path.extension() {
                // Skip non-tsp files
                if extension != "tsp" {
                    continue;
                }
                let filename = file_path
                    .file_stem()
                    .expect("File should have a stem")
                    .to_str()
                    .expect("Filename should be valid UTF-8");
                let num = extract_number_from_filename(filename).unwrap();
                if lower_bound <= num && num <= upper_bound {
                    // Generate test
                    let relative_path = format!("../../instances/{}/{}.tsp", subdir_name, filename);

                    let test_fn_name = format!("{}_{}_macro", test_name, filename);
                    let test_fn_ident =
                        syn::Ident::new(&test_fn_name, proc_macro2::Span::call_site());

                    test_functions.push(quote! {
                        #[test]
                        #[allow(non_snake_case)]
                        fn #test_fn_ident() {
                            #fn_name(#relative_path);
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        #(#test_functions)*
    };

    TokenStream::from(expanded)
}

/// Extracts the first number found in the filename string.
///
/// If no number is found, returns None.
fn extract_number_from_filename(filename: &str) -> Option<u64> {
    filename
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u64>()
        .ok()
}

/// Parses the input arguments for the procedural macro.
fn parse_macro_input_args(input: syn::parse::ParseStream) -> syn::Result<(Ident, Ident, u64, u64)> {
    let fn_name: Ident = input.parse()?;
    input.parse::<syn::Token![,]>()?;

    let test_name: Ident = input.parse()?;
    input.parse::<syn::Token![,]>()?;

    let lower_bound: LitInt = input.parse()?;
    let lower_bound_val = lower_bound.base10_parse::<u64>()?;
    input.parse::<syn::Token![,]>()?;

    let upper_bound: LitInt = input.parse()?;
    let upper_bound_val = upper_bound.base10_parse::<u64>()?;

    Ok((fn_name, test_name, lower_bound_val, upper_bound_val))
}
