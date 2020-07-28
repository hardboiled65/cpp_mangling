extern crate proc_macro;

use proc_macro::TokenStream;

#[derive(Clone)]
struct Arg {
    prefix: String,
    name: String,
}

impl Arg {
    fn parse(s: &str) -> Arg {
        let mut prefix = String::new();
        match is_reference(s) {
            true => prefix.push('R'),
            false => {}
        }
        match is_pointer(s) {
            true => prefix.push('P'),
            false => {}
        }
        match is_const(s) {
            true => prefix.push('K'),
            false => {}
        }
        let name = extract_type(s);

        Arg {
            prefix: prefix,
            name: name,
        }
    }

    fn max_s(&self) -> u32 {
        self.prefix.len() as u32
    }
}

struct Args {
    list: Vec<Arg>,
}

impl Args {
    fn new() -> Args {
        Args { list: vec![] }
    }

    fn push(&mut self, arg: &Arg) {
        self.list.push(arg.clone());
    }

    fn find_index(&self, type_name: &str) -> Option<usize> {
        let mut idx = 0;
        for arg in self.list.iter() {
            if arg.name == type_name {
                return Some(idx);
            }
            idx += 1;
        }

        None
    }

    fn s_list(&self) -> Vec<String> {
        let mut list = vec![];

        for arg in self.list.iter() {
            let name = format!("{}{}", arg.name.len(), arg.name.to_string());
            if !list.contains(&name) {
                list.push(name);
            }
            // Prefix length 2. e.g. "RK".
            if arg.prefix.len() == 2 {
                let ch = &arg.prefix[1..];
                let name = format!("{}{}{}", ch, arg.name.len(), arg.name.to_string());
                if !list.contains(&name) {
                    list.push(name);
                }
                let ch = &arg.prefix[..];
                let name = format!("{}{}{}", ch, arg.name.len(), arg.name.to_string());
                if !list.contains(&name) {
                    list.push(name);
                }
            }
            // Prefix length 1. e.g. "K"
            if arg.prefix.len() == 1 {
                let name = format!("{}{}{}", arg.prefix, arg.name.len(), arg.name.to_string());
                if !list.contains(&name) {
                    list.push(name);
                }
            }
        }

        list
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        for arg in self.list.iter() {
            match self.find_index(&arg.name) {
                Some(idx) => {}
                None => {
                    result.push_str(&arg.prefix);
                    result.push_str(&arg.name.len().to_string());
                    result.push_str(&arg.name);
                }
            }
        }

        result
    }
}

fn is_pointer(arg: &str) -> bool {
    match arg.contains("*") {
        true => true,
        false => false,
    }
}

fn is_reference(arg: &str) -> bool {
    match arg.contains("&") {
        true => true,
        false => false,
    }
}

fn is_const(arg: &str) -> bool {
    match arg.contains("const") {
        true => true,
        false => false,
    }
}

fn extract_type(arg: &str) -> String {
    let mut result = String::from(arg);
    // Strip const.
    match result.strip_prefix("const") {
        Some(s) => {
            result = s.trim().to_string();
        }
        None => {}
    }
    // Strip *, &.
    match result.strip_suffix("*") {
        Some(s) => {
            result = s.trim().to_string();
        }
        None => {}
    }
    match result.strip_suffix("&") {
        Some(s) => {
            result = s.trim().to_string();
        }
        None => {}
    }

    result
}

fn mangle_args(args: &Vec<&str>) -> String {
    let mut result = Args::new();
    for arg in args.iter() {
        println!("mangle_args {:?}", arg);

    }

    result.to_string()
}

fn mangle(s: &str) -> String {
    let mut source = s;
    let mut result = String::from("_ZN");
    match source.strip_suffix(" const") {
        Some(s) => {
            source = s;
            result.push_str("K");
        }
        None => {}
    }
    let v = source.split('(').collect::<Vec<&str>>();
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
        let args_list = args.split(',').map(|x| x.trim()).collect::<Vec<&str>>();
        result.push_str(&mangle_args(&args_list));
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
    fn test_args_s_list() {
        let mut args = super::Args::new();
        args.push(&super::Arg::parse("const Foo&"));
        args.push(&super::Arg::parse("const Bar&"));

        println!("{:?}", args.s_list());
    }

    #[test]
    fn test_mangle() {
        let s = "MyClass::my_method()";
        println!("{}", super::mangle(&s));

        assert_eq!(super::mangle("Foo::bar() const"), "_ZNK3Foo3barEv");
        // assert_eq!(super::mangle("Foo::baz(const Bar&)"), "_ZN3Foo3bazERK3Bar");
    }
}
