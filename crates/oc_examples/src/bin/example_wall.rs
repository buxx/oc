use oc_examples::tests::wall;
use oc_root::end::End;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = wall::run(vec![], End::default());
    Ok(())
}
