fn main() {
    let path = std::env::args().nth(1).expect("Please provide a path");
    let buffer = std::fs::read(&path).expect("Failed to read file");
    let prx = core::format::executable(&buffer);

    println!("Module: {}", prx.name);
    println!("Imports:");
    for import in &prx.imports {
        println!(
            "  {} {} syscalls",
            import.library,
            import.functions.len()
        );
    }
}
