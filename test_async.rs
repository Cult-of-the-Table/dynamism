fn main() {
    std::thread::spawn(|| async {
        println!("This will not print");
    }).join().unwrap();
}
