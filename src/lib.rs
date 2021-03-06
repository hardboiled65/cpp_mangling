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
                return format!("{}{}", &self.prefix, mangle_pod(&self.name));
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
                if !is_pod(&arg.name) {
                    list.push(name);
                }

                // POD type.
                if is_pod(&arg.name) {
                    if !arg.prefix.contains("P") && !arg.prefix.contains("R") {
                        // Not pointer or reference.
                        continue;
                    } else {
                        // Pointer or reference.
                        let tmp = arg.s_list();
                        for mangled_pod in tmp.iter() {
                            if !list.contains(&mangled_pod) {
                                list.push(mangled_pod.to_string());
                            }
                        }
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
                // Name contains but only "P" or "R" not contains.
                let mangled = arg.mangled();
                if !list.contains(&mangled) {
                    list.push(mangled);
                }
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
                if !is_pod(&arg.name) {
                    result.push_str(&arg.prefix);
                    result.push_str(&arg.name.len().to_string());
                    result.push_str(&arg.name);
                } else {
                    // POD type.
                    match arg.prefix.as_str() {
                        "PK" => {
                            result.push_str(&format!("PK{}", mangle_pod(&arg.name)));
                        }
                        "RK" => {
                            result.push_str(&format!("RK{}", mangle_pod(&arg.name)));
                        }
                        _ => {
                            result.push_str(&format!("{}", mangle_pod(&arg.name)));
                        }
                    }
                }
                if is_pod(&arg.name) {
                    if !arg.mangled().contains("PK") && !arg.mangled().contains("RK") {
                    } else {
                        name_list.push(arg.name.clone());
                    }
                } else {
                    name_list.push(arg.name.clone());
                }
            } else {
                if is_pod(&arg.name) && !result.contains(&arg.mangled()) {
                    match arg.prefix.as_str() {
                        "PK" => {
                            let partial = format!("K{}", mangle_pod(&arg.name));
                            match s_list.iter().position(|x| x == &partial) {
                                Some(idx) => {
                                    result.push_str(&format!("PS{}_", idx));
                                }
                                None => {
                                    result.push_str(&arg.mangled());
                                }
                            }
                        }
                        "RK" => {
                            let partial = format!("K{}", mangle_pod(&arg.name));
                            match s_list.iter().position(|x| x == &partial) {
                                Some(idx) => {
                                    result.push_str(&format!("RS{}_", idx));
                                }
                                None => {
                                    result.push_str(&arg.mangled());
                                }
                            }
                        }
                        _ => {
                            result.push_str(&arg.mangled());
                        }
                    }
                    continue;
                }
                match s_list.iter().position(|x| x == &arg.mangled()) {
                    // Exact match.
                    Some(idx) => {
                        result.push_str(&format!("S{}_", idx));
                    }
                    // Partial match.
                    None => {
                        // Match with "P" or "R"
                        let partial = if !is_pod(&arg.name) {
                            format!("{}{}{}", &arg.prefix[..1], arg.name.len(), &arg.name)
                        } else {
                            if arg.prefix.len() == 0 {
                                format!("{}", mangle_pod(&arg.name))
                            } else {
                                format!("{}{}", &arg.prefix[..1], mangle_pod(&arg.name))
                            }
                        };
                        // POD type.
                        if is_pod(&arg.name) && arg.prefix.len() == 0 {
                            result.push_str(&partial);
                            continue;
                        }
                        match s_list.iter().position(|x| x == &partial) {
                            Some(_) => {}
                            None => {
                                let len_name = format!("{}{}", &arg.name.len(), &arg.name);
                                match s_list.iter().position(|x| x == &len_name) {
                                    Some(idx) => {
                                        let s_notation = format!("{}S{}_", &arg.prefix[..1], idx);
                                        result.push_str(&s_notation);
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
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
        result.push(&Arg::parse(&arg));
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_pod() {
        assert_eq!(super::is_pod("int"), true);
    }

    #[test]
    fn test_arg_s_list() {
        let arg = super::Arg::parse("const int");
        assert_eq!(arg.s_list().len(), 0);
    }

    #[test]
    fn test_args_s_list() {
        let mut args = super::Args::new();
        args.push(&super::Arg::parse("const Foo&"));
        args.push(&super::Arg::parse("const Bar&"));
        args.push(&super::Arg::parse("Foo*"));
        args.push(&super::Arg::parse("Foo*"));

        println!("{:?}", args.s_list());

        let mut args = super::Args::new();
        args.push(&super::Arg::parse("int"));
        args.push(&super::Arg::parse("int*"));
        args.push(&super::Arg::parse("int*"));

        println!("{:?}", args.s_list());
    }

    #[test]
    fn tset_args_to_string() {
        let mut args = super::Args::new();
        args.push(&super::Arg::parse("const Foo&"));
        args.push(&super::Arg::parse("const Foo&"));
        args.push(&super::Arg::parse("const Foo"));
        args.push(&super::Arg::parse("Foo*"));

        println!("{}", args.to_string());
    }

    #[test]
    fn test_mangle() {
        let s = "MyClass::my_method()";
        println!("{}", super::mangle(&s));

        assert_eq!(super::mangle("Foo::bar() const"), "_ZNK3Foo3barEv");
        assert_eq!(super::mangle("Foo::baz(const Bar&)"), "_ZN3Foo3bazERK3Bar");
        assert_eq!(super::mangle("Foo::bar(int, const int*, int&)"), "_ZN3Foo3barEiPKiRi");
        assert_eq!(super::mangle("Foo::bar(const int*, const int&, const int*)"), "_ZN3Foo3barEPKiRS0_S1_");
        assert_eq!(super::mangle("Foo::bar(const int*, const int&, const int)"), "_ZN3Foo3barEPKiRS0_i");
    }
}
