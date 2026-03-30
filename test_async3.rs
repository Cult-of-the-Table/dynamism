fn main() {
    std::thread::spawn(async move || {
        println!("This is inside an async closure");
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
}
