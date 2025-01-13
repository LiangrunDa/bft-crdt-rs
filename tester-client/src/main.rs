use tracing::info;

mod cli;
mod orset;
mod logger;
mod rga;

use std::collections::LinkedList;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = cli::parse_args();
    if args.seed == 0 {
        args.seed = rand::random();
    }
    let _file_appender_guard = logger::init(String::from("debug"), "tokio=error,crdts=error")?;
    info!("Starting experiment with args: {:?}", args);

    let res = match args.exp_name.as_str() {
        "orset" => {
            let mut orset_experiment = orset::ORSetExperiment::new(args);
            orset_experiment.run().await
        }
        "rga" => {
            let mut rga_experiment = rga::RGAExperiment::new(args);
            rga_experiment.run().await
        }
        _ => {
            panic!("Experiment not supported");
        }
    };
    
    match res {
        Ok(_) => info!("Experiment completed successfully"),
        Err(e) => info!("Experiment failed with error: {:?}", e),
    }
    
    Ok(())
}
