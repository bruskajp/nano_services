
// ------------------------------------
// API NOTES
//
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)
// 2) The original class (Thingy) must have a constructor ("new" function)
// 3) The worker is created by calling <original_class_name>Worker::new()
// ------------------------------------

extern crate proc_macro;

use proc_macro::*;
use syn::*;
use convert_case::{Case, Casing};

// Must use this until "proc_macro_quote" becomes stable
// At which point replace to_token_stream() with quote!(#method).to_string(); 
// Or convert this whole thing into using Tokens entirely
// This can be done slowly by still using to_token_stream in intermediate steps
use syn::__private::ToTokens; 

fn params_to_arg_types_string(method: &ImplItemMethod) -> String {
  method.sig.inputs.pairs().fold(String::new(), |cur, next| {
    let symbols = match next.value() {
      FnArg::Receiver(_) => { "".to_string() },
      FnArg::Typed(ty) => match &*ty.ty {
        Type::Path(path) => {
          format!("{}",
            path.path.segments.pairs().fold(String::new(),
              |cur, next| { cur + &next.value().ident.to_string() } 
            )
          )
        },
        _ => "INVALID_TYPE_IN_FUNCTION_ARG_TYPES".to_string(),
      }
    };

    if cur.is_empty() { symbols }
    else { cur + ", " + &symbols }
  })
}

fn params_to_arg_names_string(method: &ImplItemMethod) -> String {
  method.sig.inputs.pairs().fold(String::new(), |cur, next| {
    let symbols = match next.value() {
      FnArg::Receiver(_) => { "".to_string() },
      FnArg::Typed(ty) => match &*ty.pat {
        Pat::Ident(ident) => {ident.ident.to_string()},
        _ => "INVALID_TYPE_IN_FUNCTION_ARG_NAMES".to_string(),
      }
    };

    if cur.is_empty() { symbols }
    else { cur + ", " + &symbols }
  })
}

fn returns_to_arg_types_string(method: &ImplItemMethod) -> Option<String> {
  match &method.sig.output {
    ReturnType::Default => None,
    ReturnType::Type(_, ty) => match &**ty {
      Type::Path(path) => {
        Some(format!("{}",
          path.path.segments.pairs().fold(String::new(),
            |cur, next| { cur + &next.value().ident.to_string() } 
          )
        ))
      },
      _ => Some("INVALID_TYPE_IN_FUNCTION_RETURN".to_string()),
    }
  }
}

// TODO: JPB: Change is_method_blocking to another way of determining if a method is blocking
fn is_method_blocking(method: &ImplItemMethod) -> bool {
  match &method.sig.output {
    ReturnType::Default => false,
    ReturnType::Type(_, _) => true,
  }
}

fn is_method_static(method: &ImplItemMethod) -> bool {
  method.sig.inputs.pairs().fold(true, |cur, next| {
    cur && match next.value() {
      FnArg::Receiver(_) => false,
      FnArg::Typed(_) => true,
    }
  })
}

// TODO: JPB: Make everything except the worker methods private (including the original class?)
// TODO: JPB: Make the original class's constructor create the worker?
#[proc_macro_attribute]
pub fn worker(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = item.clone();
  let input = parse_macro_input!(input as ItemImpl);

  let class_name = match *input.self_ty {
    Type::Path(path) => {
      format!("{}",
        path.path.segments.pairs().fold(String::new(),
          |cur, next| { cur + &next.value().ident.to_string() } 
        )
      )
    },
    _ => "INVALID_TYPE_FOR_IMPL_NAME".to_string(),
  };
  let object_name = class_name.to_case(Case::Camel);

  // Generate Includes
  let mut includes_output = Vec::new();
  includes_output.push(format!("use crossbeam_channel;"));
  includes_output.push(format!("use futures;"));

  // Generate WorkerFuncs Enum
  let mut funcs_enum_output = Vec::new();
  funcs_enum_output.push(format!("enum WorkerFuncs {{"));
  funcs_enum_output.push(format!("WorkerQuit(),"));

  // Generate Struct Worker
  let mut worker_struct_output = Vec::new();
  worker_struct_output.push(format!("#[derive(Clone, Debug)]"));
  worker_struct_output.push(format!("struct {class_name}Worker {{"));
  worker_struct_output.push(format!("send: crossbeam_channel::Sender<Box<WorkerFuncs>>,"));
  worker_struct_output.push(format!("}}"));

  // Generate Impl Worker
  let mut worker_impl_output = Vec::new();
  let mut worker_impl_new_intro = Vec::new();
  let mut worker_impl_new_match = Vec::new();
  let mut worker_impl_new_outro = Vec::new();
  worker_impl_output.push(format!("impl {class_name}Worker {{"));
  worker_impl_output.push(format!("pub fn stop_thread(&self) {{"));
  worker_impl_output.push(format!("self.send.send(Box::new(WorkerFuncs::WorkerQuit())).expect(\"Failed to send stop_thread command\");"));
  worker_impl_output.push(format!("}}"));

  // Walk through original Impl functions
  input
  .items.iter().for_each(|item| {
    match item {
      ImplItem::Method(method) => {
          match method.vis { // Only expose public functions
            Visibility::Public(_) => {
              let method_name = method.sig.ident.to_string();
              let method_signature = method.sig.to_token_stream().to_string(); 
              let method_params = method.sig.inputs.to_token_stream().to_string();
              let enum_name = method_name.to_case(Case::UpperCamel);
              let method_arg_names = params_to_arg_names_string(method);
              let method_arg_types = params_to_arg_types_string(method);
              let method_return_type = returns_to_arg_types_string(method);
              let method_return_type_str = match &method_return_type {
                None => { format!("()") },
                Some(return_type) => { format!("{}", return_type) }
              }; 
              
              let mut enum_arg_types = method_arg_types.clone();
              let mut enum_arg_names = method_arg_names.clone();

              let method_is_blocking = is_method_blocking(method);
              let method_is_static = is_method_static(method);
              let method_is_constructor = method_name == "new";

              println!("{} {}", method_name, method_is_blocking);

              // Generate WorkerFuncs Enum
              if !method_is_constructor && !method_is_static {
                if method_is_blocking {
                  enum_arg_types = format!("futures::channel::oneshot::Sender<Box<{}>>, {}", method_return_type_str, method_arg_types);
                  enum_arg_names = format!("send_ret, {}", method_arg_names);
                };

                funcs_enum_output.push(format!("{}({}),",
                  enum_name, enum_arg_types,
                ));
              }

              // Generate Impl ThingyWorker
              if method_is_constructor {
                worker_impl_new_intro.push(format!("pub fn new({}) -> (std::thread::JoinHandle<()>, Self) {{",
                  method_params
                ));
                worker_impl_new_intro.push(format!("let (send_func, recv_func) = crossbeam_channel::unbounded::<Box<WorkerFuncs>>();"));
                worker_impl_new_intro.push(format!("let {} = {}::new({});",
                  object_name, class_name, method_arg_names
                ));
                worker_impl_new_intro.push(format!("let handle = std::thread::spawn(move || {{"));
                worker_impl_new_intro.push(format!("loop {{"));
                worker_impl_new_intro.push(format!("match *recv_func.recv().expect(\"Error in Worker when receiving message \") {{"));
                
                worker_impl_new_match.push(format!("WorkerFuncs::WorkerQuit() => break,"));

                worker_impl_new_outro.push(format!(""));
                worker_impl_new_outro.push(format!("}}"));
                worker_impl_new_outro.push(format!("}}"));
                worker_impl_new_outro.push(format!("}});"));
                worker_impl_new_outro.push(format!("(handle, Self {{send: send_func}})"));
                worker_impl_new_outro.push(format!("}}"));
              } else if !method_is_static {
                if method_is_blocking {
                  worker_impl_new_match.push(format!("WorkerFuncs::{}({}) => send_ret.send(Box::new({}.{}({}))).expect(\"Failed to send return value of {} in Worker\"),",
                    enum_name, enum_arg_names, object_name, method_name, method_arg_names, enum_name
                  ));
                } else {
                  worker_impl_new_match.push(format!("WorkerFuncs::{}({}) => {}.{}({}),",
                    enum_name, enum_arg_names, object_name, method_name, method_arg_names
                  ));
                }
              }

              // Generate Impl ThingyWorker
              if !method_is_constructor && !method_is_static {
                worker_impl_output.push(format!("pub {} {{", method_signature));
                if method_is_blocking {
                  worker_impl_output.push(format!("let (send_ret, recv_ret) = futures::channel::oneshot::channel::<Box<{}>>();;", 
                    method_return_type_str
                  ));
                }
                worker_impl_output.push(format!("self.send.send(Box::new(WorkerFuncs::{}({}))).expect(\"Failed to send {} to Worker\");",
                  enum_name, enum_arg_names, enum_name
                ));
                if method_is_blocking {
                  worker_impl_output.push(format!("match futures::executor::block_on(async move {{ recv_ret.await }}) {{"));
                  worker_impl_output.push(format!("Ok(x) => *x,"));
                  worker_impl_output.push(format!("Err(_) => panic!(\"Error on async await of result in {}\"),", method_name));
                  worker_impl_output.push(format!("}}"));
                }
                worker_impl_output.push(format!("}}"));
              }
            }
            _ => {}
          }
        }
      _ => { println!("INVALID_FUNCTION_TYPE"); }
    }
  });

  // Generate WorkerFuncs Enum
  funcs_enum_output.push(format!("}}"));

  // Generate Impl Worker
  worker_impl_output.push(worker_impl_new_intro.join("\n"));
  worker_impl_output.push(worker_impl_new_match.join("\n"));
  worker_impl_output.push(worker_impl_new_outro.join("\n"));
  worker_impl_output.push(format!("}}"));

  //println!("----------------------------");

  format!("{}\n{}\n{}\n{}\n{}\n",
    item,
    includes_output.join("\n"),
    funcs_enum_output.join("\n"),
    worker_struct_output.join("\n"),
    worker_impl_output.join("\n")
  ).parse().expect("Generated invalid tokens")
}


#[proc_macro_attribute]
pub fn intro(_args: TokenStream, input: TokenStream) -> TokenStream {
  let _input = input.clone();
  let input = parse_macro_input!(input as ItemStruct);

  //println!("----------------------------");
  //println!("FIELDS:");
  //input
  //.fields
  //.iter()
  //.for_each(|field| { println!("{}", field.ident.as_ref().unwrap()); });
  //println!("----------------------------");

  let class_name = &input.ident;

  let output = format!(r#"
      {_input}
      impl {class_name} {{
        pub fn introspect(){{
          println!("Introspect");
        }}
      }}
    "#
  );

  output.parse().expect("Generated invalid tokens")
}


// ------------------------------------
// UNIT TESTS

#[cfg(test)]
mod tests {
  #[test]
  fn basic_unit_test() {
    assert_eq!(1, 1);
  }
}


// ------------------------------------
// HELPFUL STUFF

// print type of variable
//fn print_type_of<T>(_: &T) -> String {
//  format!("{}", std::any::type_name::<T>())
//}

// List of options in syn::Type enum
//FnArg::Typed(ty) => match &*ty.ty {
//  Type::Array(_) => {format!("1")},
//  Type::BareFn(_) => {format!("2")},
//  Type::Group(_) => {format!("3")},
//  Type::ImplTrait(_) => {format!("4")},
//  Type::Infer(_) => {format!("5")},
//  Type::Macro(_) => {format!("6")},
//  Type::Never(_) => {format!("7")},
//  Type::Paren(_) => {format!("8")},
//  Type::Path(_) => {format!("9")},
//  Type::Ptr(_) => {format!("10")},
//  Type::Reference(_) => {format!("11")},
//  Type::Slice(_) => {format!("12")},
//  Type::TraitObject(_) => {format!("13")},
//  Type::Tuple(_) => {format!("14")},
//  Type::Verbatim(_) => {format!("15")},
//}

// macro on functions 
// https://stackoverflow.com/questions/52585719/how-do-i-create-a-proc-macro-attribute

// For a tutorial on how to implement a proc_macro_atrribute!!!
// https://doc.rust-lang.org/reference/procedural-macros.html
// https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#proceduralmacrosinrust
// https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/#customderivemacros

// Rename a function with a macro
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

// Capture stdio as macro for assert
// https://users.rust-lang.org/t/how-to-test-functions-that-use-println/67188/5


