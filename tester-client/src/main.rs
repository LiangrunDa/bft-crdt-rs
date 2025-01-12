use tracing::info;

mod cli;
mod orset;
mod logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse_args();
    let _file_appender_guard = logger::init(String::from("debug"), "tokio=error,crdts=trace")?;
    info!("Starting experiment with args: {:?}", args);
    
    match args.exp_name.as_str() {
        "orset" => {
            let mut orset_experiment = orset::ORSetExperiment::new(args);
            orset_experiment.run().await?;
        }
        _ => {
            panic!("Experiment not supported");
        }
    }

    Ok(())
}
