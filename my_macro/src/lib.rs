// ------------------------------------
// API NOTES
//
// 1) All functions must use owned passing (no references) for thread safety (stop deadlocks)
// 2) The original class (Thingy) must have a constructor ("new" function)
// 3) The worker is created by calling <original_class_name>Worker::new()
// 4) Methods are not allowed to be non-blocking and have a return value (no promises)
// ------------------------------------

use convert_case::{Case, Casing};
use proc_macro::*;
use proc_macro_error::*;
use syn::*;

// Must use this until "proc_macro_quote" becomes stable
// At which point replace to_token_stream() with quote!(#method).to_string();
// Or convert this whole thing into using Tokens entirely
// This can be done slowly by still using to_token_stream in intermediate steps
use syn::__private::ToTokens;

fn params_to_arg_types_string(method: &ImplItemMethod) -> String {
    method.sig.inputs.pairs().fold(String::new(), |cur, next| {
        let symbols = match next.value() {
            FnArg::Receiver(_) => "".to_string(),
            FnArg::Typed(ty) => match &*ty.ty {
                Type::Path(path) => path.path.segments.pairs().fold(String::new(), |cur, next| {
                    cur + &next.value().ident.to_string()
                }),
                _ => "INVALID_TYPE_IN_FUNCTION_ARG_TYPES".to_string(),
            },
        };

        if cur.is_empty() {
            symbols
        } else {
            cur + ", " + &symbols
        }
    })
}

fn params_to_arg_names_string(method: &ImplItemMethod) -> String {
    method.sig.inputs.pairs().fold(String::new(), |cur, next| {
        let symbols = match next.value() {
            FnArg::Receiver(_) => "".to_string(),
            FnArg::Typed(ty) => match &*ty.pat {
                Pat::Ident(ident) => ident.ident.to_string(),
                _ => "INVALID_TYPE_IN_FUNCTION_ARG_NAMES".to_string(),
            },
        };

        if cur.is_empty() {
            symbols
        } else {
            cur + ", " + &symbols
        }
    })
}

fn returns_to_arg_types_string(method: &ImplItemMethod) -> Option<String> {
    match &method.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(path) => {
                Some(path.path.segments.pairs().fold(String::new(), |cur, next| {
                    cur + &next.value().ident.to_string()
                }))
            }
            _ => Some("INVALID_TYPE_IN_FUNCTION_RETURN".to_string()),
        },
    }
}

fn is_method_blocking(method: &ImplItemMethod) -> bool {
    method
        .attrs
        .iter()
        .any(|x| x.path.segments.iter().any(|x| x.ident == "blocking_method"))
}

fn is_method_static(method: &ImplItemMethod) -> bool {
    method.sig.inputs.pairs().all(|next| match next.value() {
        FnArg::Receiver(_) => false,
        FnArg::Typed(_) => true,
    })
}

// TODO: JPB: (feature) Make everything except the worker methods private (including the original class?)
// TODO: JPB: (feature) Make the original class's constructor create the worker?
// TODO: JPB: (feature) Add the ability to use this method on traits as well
// TODO: JPB: (feature) Add publish/subscribe feature (maybe as another proc_macro_aatribute)
// TODO: JPB: (QOL) Change "Worker" to "NanoService"
#[proc_macro_error]
#[proc_macro_attribute]
pub fn worker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = item.clone();
    let input = parse_macro_input!(input as ItemImpl);

    let Type::Path(path) = &*input.self_ty else {
        abort!(input.self_ty, "Invalid type for impl name");
    };

    let class_name = path.path.segments.pairs().fold(String::new(), |cur, next| {
        cur + &next.value().ident.to_string()
    });
    let object_name = class_name.to_case(Case::Camel);

    // Generate Includes
    let includes_output = vec![
        "use crossbeam_channel;".to_string(), 
        "use futures;".to_string()];

    // Generate WorkerFuncs Enum
    let mut funcs_enum_output = vec![
        "enum WorkerFuncs {".to_string(),
        "WorkerQuit(),".to_string()];

    // Generate Struct Worker
    let worker_struct_output = vec![
        "#[derive(Clone, Debug)]".to_string(),
        format!("struct {class_name}Worker {{"),
        "send: crossbeam_channel::Sender<Box<WorkerFuncs>>,".to_string(),
        "}".to_string()];

    // Generate Impl Worker
    let mut worker_impl_new_intro = Vec::new();
    let mut worker_impl_new_match = Vec::new();
    let mut worker_impl_new_outro = Vec::new();
    let mut worker_impl_output = vec![
        format!("impl {class_name}Worker {{"),
        "pub fn stop_thread(&self) {".to_string(),
        "self.send.send(Box::new(WorkerFuncs::WorkerQuit())).expect(\"Failed to send stop_thread command\");".to_string(),
        "}".to_string()];

    // Check that the class has a public "new" method
    let mut new_exists = false;
    let mut pub_new_exists = false;
    for item in &input.items {
        if let ImplItem::Method(method) = item {
            let method_is_new = method.sig.ident == "new";
            let method_is_public = matches!(method.vis, Visibility::Public(_));
            new_exists |= method_is_new;
            pub_new_exists |= method_is_new && method_is_public;
            if pub_new_exists {
                break;
            }
        }
    }
    if !new_exists {
        emit_error!(input, "The \"{class_name}\" class does not have a public \"new\" method. All #[worker] classes must have a public \"new\" method. Please create a public \"new\" method.");
    } else if !pub_new_exists {
        emit_error!(input, "The \"{class_name}\" class has a private \"new\" method. All #[worker] classes must have a public \"new\" method. Please make your \"new\" method public.");
    }

    // Walk through original Impl functions
    input.items.iter().for_each(|item| {
      let ImplItem::Method(method) = item else {
        abort!(item, "Non-method found inside impl block. Only methods are allowed in impl blocks.");
      };

      if let Visibility::Public(_) = method.vis { // Only expose public functions
        let method_name = method.sig.ident.to_string();
        let method_signature = method.sig.to_token_stream().to_string();
        let method_params = method.sig.inputs.to_token_stream().to_string();
        let enum_name = method_name.to_case(Case::UpperCamel);
        let method_arg_names = params_to_arg_names_string(method);
        let method_arg_types = params_to_arg_types_string(method);
        let method_return_type = returns_to_arg_types_string(method);
        let method_return_type_str = match &method_return_type {
          None => "()",
          Some(return_type) => return_type,
        };

        let mut enum_arg_types = method_arg_types.clone();
        let mut enum_arg_names = method_arg_names.clone();

        let method_is_blocking = is_method_blocking(method);
        let method_is_static = is_method_static(method);
        let method_is_constructor = method_name == "new";

        // Debug Info
        println!("{} ({})", method_name, if method_is_blocking {"blocking"} else {"non-blocking"});

        // Check for methods that are non-blocking and have a return type
        if !method_is_static && !method_is_blocking && method_return_type.is_some() {
          emit_error!(method.sig, "Method {class_name}::{method_name} is a non-blocking method, but has a return type ({method_return_type_str}). This is not allowed.");
        }

        // Generate WorkerFuncs Enum
        if !method_is_static {
          if method_is_blocking {
            enum_arg_types = format!("futures::channel::oneshot::Sender<Box<{method_return_type_str}>>, {method_arg_types}");
            enum_arg_names = format!("send_ret, {method_arg_names}");
          };

          funcs_enum_output.push(format!("{enum_name}({enum_arg_types}),"));
        }

        // Generate Impl ThingyWorker
        if method_is_constructor {
          worker_impl_new_intro.push(format!("pub fn new({method_params}) -> (std::thread::JoinHandle<()>, Self) {{"));
          worker_impl_new_intro.push("let (send_func, recv_func) = crossbeam_channel::unbounded::<Box<WorkerFuncs>>();".to_string());
          worker_impl_new_intro.push(format!("let {object_name} = {class_name}::new({method_arg_names});"));
          worker_impl_new_intro.push("let handle = std::thread::spawn(move || {".to_string());
          worker_impl_new_intro.push("loop {".to_string());
          worker_impl_new_intro.push("match *recv_func.recv().expect(\"Error in Worker when receiving message \") {".to_string());

          worker_impl_new_match.push("WorkerFuncs::WorkerQuit() => break,".to_string());

          worker_impl_new_outro.push(String::new());
          worker_impl_new_outro.push("}".to_string());
          worker_impl_new_outro.push("}".to_string());
          worker_impl_new_outro.push("});".to_string());
          worker_impl_new_outro.push("(handle, Self {send: send_func})".to_string());
          worker_impl_new_outro.push("}".to_string());
        } else if !method_is_static {
          if method_is_blocking {
            worker_impl_new_match.push(format!("WorkerFuncs::{enum_name}({enum_arg_names}) => send_ret.send(Box::new({object_name}.{method_name}({method_arg_names}))).expect(\"Failed to send return value of {enum_name} in Worker\"),"));
          } else {
            worker_impl_new_match.push(format!("WorkerFuncs::{enum_name}({enum_arg_names}) => {object_name}.{method_name}({method_arg_names}),"));
          }
        }

        // Generate Impl ThingyWorker
        if !method_is_static {
          worker_impl_output.push(format!("pub {method_signature} {{"));
          if method_is_blocking {
            worker_impl_output.push(format!("let (send_ret, recv_ret) = futures::channel::oneshot::channel::<Box<{method_return_type_str}>>();;"));
          }
          worker_impl_output.push(format!("self.send.send(Box::new(WorkerFuncs::{enum_name}({enum_arg_names}))).expect(\"Failed to send {enum_name} to Worker\");"));
          if method_is_blocking {
            worker_impl_output.push("match futures::executor::block_on(async move { recv_ret.await }) {".to_string());
            worker_impl_output.push("Ok(x) => *x,".to_string());
            worker_impl_output.push(format!("Err(_) => panic!(\"Error on async await of result in {method_name}\"),"));
            worker_impl_output.push("}".to_string());
          }
          worker_impl_output.push("}".to_string());
        }
      }
    });

    // Generate WorkerFuncs Enum
    funcs_enum_output.push("}".to_string());

    // Generate Impl Worker
    worker_impl_output.push(worker_impl_new_intro.join("\n"));
    worker_impl_output.push(worker_impl_new_match.join("\n"));
    worker_impl_output.push(worker_impl_new_outro.join("\n"));
    worker_impl_output.push("}".to_string());

    //println!("----------------------------");

    format!(
        "{}\n{}\n{}\n{}\n{}\n",
        item,
        includes_output.join("\n"),
        funcs_enum_output.join("\n"),
        worker_struct_output.join("\n"),
        worker_impl_output.join("\n")
    )
    .parse()
    .expect("Generated invalid tokens")
}

#[proc_macro_attribute]
pub fn blocking_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
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

    let output = format!(
        r#"
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
