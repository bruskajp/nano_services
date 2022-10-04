extern crate proc_macro;

use proc_macro::*;
//use quote::quote;
//use syn::{parse_macro_input, DeriveInput};
//use syn::{parse_macro_input, ItemStruct};
use syn::*;
use convert_case::{Case, Casing};

// Must use this until "proc_macro_quote" becomes stable
use syn::__private::ToTokens; 


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

fn params_to_args_string(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> String {
  inputs.pairs().fold(String::new(), |cur, next| {
    //println!("{:#?}", next.value());
    let symbols = match next.value() {
      FnArg::Receiver(_) => { "".to_string() },
      FnArg::Typed(ty) => match &*ty.pat {
        Pat::Ident(ident) => {ident.ident.to_string()},
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

fn is_static_method(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> bool {
  inputs.pairs().fold(true, |cur, next| {
    cur && match next.value() {
      FnArg::Receiver(_) => false,
      FnArg::Typed(_) => true,
    }
  })
}

#[proc_macro_attribute]
pub fn worker(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = item.clone();
  let input = parse_macro_input!(input as ItemImpl);

  //println!("{}", match *input.self_ty { Type::ImplTrait(a) => {a}, _ => TypeImplTrait() });
  //let name = &input.ident;
  let name = "Thingy"; // JPB: TODO: Get actual name of the class

  // Generate WorkerFuncs Enum
  let mut enum_output = Vec::new();
  enum_output.push(format!("enum WorkerFuncs {{"));
  enum_output.push(format!("WorkerQuit(),"));

  // Generate Struct Worker
  let worker_struct_output = format!("struct {name}Worker;");

  // JPB: TODO: Generate Impl Worker
  let worker_impl_output = format!("impl {name}Worker {{ }}");

  // Generate Struct Controller
  let controller_struct_output = format!("struct {name}Controller {{\nsend: Sender<Box<WorkerFuncs>>,\n}}");

  // Generate Impl Controller
  let mut controller_impl_output = Vec::new();
  controller_impl_output.push(format!("impl {name}Controller {{"));
  controller_impl_output.push(format!("pub fn controller_stop_thread(&self) {{"));
  controller_impl_output.push(format!("self.send.send(Box::new(WorkerFuncs::WorkerQuit())).unwrap();"));
  controller_impl_output.push(format!("}}"));

  // Walk through original Impl functions
  input
  .items.iter().for_each(|item| {
    match item {
      ImplItem::Method(method) => {
          // Print Info About types
          //println!("{}({})", 
          //  a.sig.ident,
          //  types_to_string(&a.sig.inputs),
          //);
 
          match method.vis { // Only expose public functions
            Visibility::Public(_) => {
              // Generate WorkerFuncs Enum
              if method.sig.ident != "new" && !is_static_method(&method.sig.inputs) {
                enum_output.push(format!("{}({}),",
                  method.sig.ident.to_string().to_case(Case::UpperCamel),
                  types_to_string(&method.sig.inputs),
                ));
              }

              // JPB: TODO: Generate Impl ThingyWorker
              

              // Generate Impl ThingyController
              if method.sig.ident != "new" && !is_static_method(&method.sig.inputs) {
                //controller_impl_output.push(quote!(#method).to_string());
                controller_impl_output.push(format!("pub {} {{\nself.send.send(Box::new(WorkerFuncs::{}({}))).unwrap();\n}}",
                  method.sig.to_token_stream().to_string(),
                  method.sig.ident.to_string().to_case(Case::UpperCamel),
                  params_to_args_string(&method.sig.inputs)
                ));
              }
            }
            _ => {}
          }
        }
      _ => { println!("INVALID_FUNCTION_TYPE"); }
    }
  });

  // Generate WorkerFuncs Enum
  enum_output.push(format!("}}"));

  // Generate Impl ThingyController
  controller_impl_output.push(format!("}}"));

  println!("----------------------------");

  format!("{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n",
    item,
    enum_output.join("\n"),
    worker_struct_output,
    worker_impl_output,
    controller_struct_output,
    controller_impl_output.join("\n")
  ).parse().expect("Generated invalid tokens")
}


#[proc_macro_attribute]
pub fn intro(_args: TokenStream, input: TokenStream) -> TokenStream {
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


