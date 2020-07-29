use cpp_mangling::mangle_fn;

#[mangle_fn(hello::world())]
fn useless_name() {
}

#[mangle_fn(Foo::Bar::baz(const QString&, int))]
fn foo_bar_baz() {
}

#[test]
fn test_mangle_fn() {
}
