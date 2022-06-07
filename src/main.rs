use zero2prod::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Redirect all `log`'s events to our subscriber
    init_subscriber(get_subscriber(
        "zero2prod".to_string(),
        "info".to_string(),
        std::io::stdout,
    ));

    //configuration + database
    let configuration = get_configuration().expect("Failed to read configuration, désolé");

    Application::build(configuration)
        .await?
        .run_until_stopped()
        .await?;
    Ok(())
}
