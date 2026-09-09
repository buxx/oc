use oc_deployment::generator::profile::{Individual, Magazine, Profile, Squad, Weapons};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = serde_yaml::to_string(&example())?;
    println!("{}", value,);
    Ok(())
}

fn example() -> Profile {
    Profile {
        squads: vec![Squad {
            count: 2,
            label: "MySquad1".to_string(),
            individuals: vec![Individual {
                count: 3,
                weapons: Weapons {
                    main: Some("MosinNagantM1924".to_string()),
                },
                magazines: vec![Magazine {
                    count: 10,
                    name: "MosinNagant5x".to_string(),
                }],
            }],
        }],
    }
}
