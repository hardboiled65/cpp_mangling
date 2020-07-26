extern crate proc_macro;

use proc_macro::TokenStream;

fn is_const(arg: &str) -> bool {
    if arg.contains("const") {
        return true;
    }

    false
}

fn mangle(s: &str) -> String {
    let mut result = String::from("_ZN");
    let v = s.split('(').collect::<Vec<&str>>();
    let name = v[0].to_string();
    let mut args = v[1].to_string();
    args.pop(); // Remove ')'

    // Mangle name.
    let names = name.split("::");
    for n in names {
        let name = n.trim();
        let len = name.len().to_string();
        result.push_str(&len);
        result.push_str(name);
    }
    result.push('E');
    // Mangle args.
    if args == "" {
        result.push('v');
    } else {
        println!("`{}`", args);
    }

    result
}

#[proc_macro_attribute]
pub fn mangle_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let src = attr.to_string();
    let name = mangle(&src);

    let mut result = item.to_string();
    let mut name_rng = 0..0;
    // Next ident is source fn name.
    let mut is_fn = false;
    for i in item.into_iter() {
        if is_fn == true {
            let fn_name = i.to_string();
            name_rng = result.find(&fn_name).unwrap()..fn_name.len();
            break;
        }
        match i {
            proc_macro::TokenTree::Ident(ident) => {
                if ident.to_string() == "fn" {
                    is_fn = true;
                }
            }
            _ => {}
        }
    }
    result.replace_range(name_rng, &name);
    println!("{:?}", name);

    result.parse().unwrap()
}

#[proc_macro]
pub fn mangle_call(item: TokenStream) -> TokenStream {
    item
}

mod tests {
    #[test]
    fn test_mangle() {
        let s = "MyClass::my_method()";
        println!("{}", super::mangle(&s));
    }
}
