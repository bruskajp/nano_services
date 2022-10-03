extern crate proc_macro;

use proc_macro::*;
//use quote::quote;
//use syn::{parse_macro_input, DeriveInput};
//use syn::{parse_macro_input, ItemStruct};
use syn::*;
use convert_case::{Case, Casing};


// ------------------------------------
// HELPFUL STUFF

// macro on functions 
// https://stackoverflow.com/questions/52585719/how-do-i-create-a-proc-macro-attribute

// For a tutorial on how to implement a proc_macro_atrribute!!!
// https://doc.rust-lang.org/reference/procedural-macros.html
// https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#proceduralmacrosinrust
// https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#customderivemacros

// print type of variable
fn print_type_of<T>(_: &T) -> String {
  format!("{}", std::any::type_name::<T>())
}

// ------------------------------------


fn types_to_string(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> String {
  inputs.pairs().fold(String::new(), |cur, next| {
    let symbols = match next.value() {
      FnArg::Receiver(_) => { "".to_string() },
      FnArg::Typed(ty) => match &*ty.ty {
        Type::Path(path) => {
            format!("{}",
              path.path.segments.pairs().fold(
                String::new(),
                |cur, next| {cur + &next.value().ident.to_string()} 
              )
            )
          },
        _ => "INVALID_TYPE_IN_FUNCTION_ARGS".to_string(),
        //Type::Array(_) => {format!("1")},
        //Type::BareFn(_) => {format!("2")},
        //Type::Group(_) => {format!("3")},
        //Type::ImplTrait(_) => {format!("4")},
        //Type::Infer(_) => {format!("5")},
        //Type::Macro(_) => {format!("6")},
        //Type::Never(_) => {format!("7")},
        //Type::Paren(_) => {format!("8")},
        //Type::Ptr(_) => {format!("10")},
        //Type::Reference(_) => {format!("11")},
        //Type::Slice(_) => {format!("12")},
        //Type::TraitObject(_) => {format!("13")},
        //Type::Tuple(_) => {format!("14")},
        //Type::Verbatim(_) => {format!("15")},
      }
    };

    if cur.is_empty() {
      symbols
    } else {
      cur + ", " + &symbols
    }
  })
}

#[proc_macro_attribute]
pub fn worker(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = item.clone();
  let input = parse_macro_input!(input as ItemImpl);

  //println!("{}", match *input.self_ty { Type::ImplTrait(a) => {a}, _ => TypeImplTrait() });
  //let name = &input.ident;
  let name = "Worker";

  let mut enum_output = Vec::new();
  enum_output.push(format!("enum {name}Funcs {{"));
  enum_output.push(format!("WorkerQuit(),"));


  input
  .items.iter().for_each(|item| {
    match item {
      ImplItem::Method(a) => {
          // Print Info About types
          //println!("{}({})", 
          //  a.sig.ident,
          //  types_to_string(&a.sig.inputs),
          //);

          // Generate WorkerFuncs Enum
          if a.sig.ident != "new" {
            enum_output.push(format!("{}({}),",
              a.sig.ident.to_string().to_case(Case::UpperCamel),
              types_to_string(&a.sig.inputs),
            ));
          }
        }
      _ => { println!("INVALID_FUNCTION_TYPE"); }
    }
  });

  enum_output.push(format!("}}"));

  println!("----------------------------");

  format!("{}\n{}", item, enum_output.join("\n")).parse().expect("Generated invalid tokens")
}


#[proc_macro_attribute]
pub fn intro(args: TokenStream, input: TokenStream) -> TokenStream {
  let _input = input.clone();
  let input = parse_macro_input!(input as ItemStruct);

  println!("----------------------------");
  println!("FIELDS:");
  input
  .fields
  .iter()
  .for_each(|field| { println!("{}", field.ident.as_ref().unwrap()); });
  println!("----------------------------");

  let name = &input.ident;

  let output = format!(r#"
      {_input}
      impl {name} {{
        pub fn introspect(){{
          println!("Introspect");
        }}
      }}
    "#
  );

  // Return output TokenStream so your custom derive behavior will be attached.
  //let output: proc_macro::TokenStream = output.parse().unwrap();
  //_input + ""
  //_input + TokenStream(output.parse().expect("Generated invalid tokens"))
  output.parse().expect("Generated invalid tokens")
  
  //"".parse().expect("Generated invalid tokens")
}


// https://github.com/LevitatingLion/rename-item/blob/main/src/lib.rs
// https://github.com/Manishearth/rust-adorn/blob/master/src/lib.rs
// https://dev.to/naufraghi/procedural-macro-in-rust-101-k3f
// https://crates.io/crates/syn
//#[proc_macro_attribute]
//pub fn rename(attr: TokenStream, item: TokenStream) -> TokenStream {
//    // Parse attribute and item
//    let args = parse_macro_input!(attr as AttributeArgs);
//    let mut item = parse_macro_input!(item as Item);
//
//    // Convert macro input to target name
//    let name = MacroInput::from_list(&args).and_then(|input| input.into_name(Some(&item)));
//
//    // Apply target name to the item
//    let toks = name.and_then(|name| {
//        let ident = Ident::new(&name, Span::call_site());
//        set_ident(&mut item, ident)?;
//        Ok(item.into_token_stream())
//    });
//
//    // Handle errors
//    match toks {
//        Ok(toks) => toks,
//        Err(err) => err.write_errors(),
//    }
//    .into()
//}


