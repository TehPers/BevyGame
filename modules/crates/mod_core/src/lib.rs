#[no_mangle]
extern "C" fn _start() {
    println!("Loaded module :D");
}

#[no_mangle]
extern "C" fn on_update() {
    println!("Hello from wasi :D");
}
