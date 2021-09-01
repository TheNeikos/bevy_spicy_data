bevy_spicy_data::data_config!(pub config, "examples/simple.toml");

fn main() {
    let data = std::fs::read("examples/simple.toml").unwrap();
    let config: config::Root = toml::from_slice(&data).unwrap();

    println!("{:#?}", config);

}