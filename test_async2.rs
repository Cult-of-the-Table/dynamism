fn main() {
    std::thread::spawn(async move || {
        println!("This is inside an async closure");
    }).join().unwrap();
}
