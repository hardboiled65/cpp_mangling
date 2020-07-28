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
            true => {
                if is_reference(s) || is_pointer(s) {
                    prefix.push('K')
                }
            }
            false => {}
        }
        let name = extract_type(s);

        Arg {
            prefix: prefix,
            name: name,
        }
    }

    fn mangled(&self) -> String {
        if is_pod(&self.name) {
            if !self.prefix.contains("P") && !self.prefix.contains("R") {
                return format!("{}", mangle_pod(&self.name));
            } else {
                return format!("{}{}", &self.prefix, &self.name);
            }
        } else {
            return format!("{}{}{}", &self.prefix, self.name.len(), &self.name);
        }
    }

    fn s_list(&self) -> Vec<String> {
        let mut list = vec![];

        if is_pod(&self.name) {
            if !self.prefix.contains("P") && !self.prefix.contains("R") {
                return list;
            }
            // If type is POD but pointer or reference.
            let mangled_type = mangle_pod(&self.name);
            list.push(mangled_type.to_string());
            match self.prefix.as_str() {
                "P" => {
                    list.push(format!("P{}", mangled_type));
                }
                "R" => {
                    list.push(format!("R{}", mangled_type));
                }
                "PK" => {
                    list.push(format!("K{}", mangled_type));
                    list.push(format!("PK{}", mangled_type));
                }
                "RK" => {
                    list.push(format!("K{}", mangled_type));
                    list.push(format!("RK{}", mangled_type));
                }
                _ => panic!("Invalid prefix: {}", self.prefix),
            }
        } else {

        }

        list
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

    fn s_list(&self) -> Vec<String> {
        let mut list = vec![];

        for arg in self.list.iter() {
            let name = format!("{}{}", arg.name.len(), arg.name.to_string());
            if !list.contains(&name) {
                list.push(name);

                // POD type.
                if is_pod(&arg.name) {
                    if !arg.prefix.contains("P") && !arg.prefix.contains("R") {
                        // Not pointer or reference.
                        continue;
                    } else {
                        // Pointer or reference.
                        let mut tmp = arg.s_list();
                        list.append(&mut tmp);
                        continue;
                    }
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
                // Prefix length 1. e.g. "P"
                if arg.prefix.len() == 1 {
                    let name = format!("{}{}{}", arg.prefix, arg.name.len(), arg.name.to_string());
                    if !list.contains(&name) {
                        list.push(name);
                    }
                }
            } else {
                continue;
            }
        }

        list
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        let s_list = self.s_list();
        let mut name_list = vec![];

        for arg in self.list.iter() {
            // First, if not in `name_list` then append full mangled name.
            // Else, append as S notation.
            if !name_list.contains(&arg.name) {
                result.push_str(&arg.prefix);
                result.push_str(&arg.name.len().to_string());
                result.push_str(&arg.name);
                name_list.push(arg.name.clone());
            } else {
                match s_list.iter().position(|x| x == &arg.name) {
                    Some(idx) => {}
                    None => {}
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

fn is_pod(type_name: &str) -> bool {
    match type_name {
        "int" => true,
        "float" => true,
        "double" => true,
        "char" => true,
        _ => false,
    }
}

fn mangle_pod(type_name: &str) -> &'static str {
    match type_name {
        "int" => "i",
        "float" => "f",
        "double" => "d",
        "char" => "c",
        _ => panic!("Invalid type name: {}", type_name),
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

[cfg(test)]
mod tests {
    #[test]
    fn test_args_s_list() {
        let mut args = super::Args::new();
        args.push(&super::Arg::parse("const Foo&"));
        args.push(&super::Arg::parse("const Bar&"));
        args.push(&super::Arg::parse("Foo*"));

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
