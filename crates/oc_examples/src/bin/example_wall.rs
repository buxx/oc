use oc_examples::tests::wall;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = wall::run(None);
    Ok(())
}
