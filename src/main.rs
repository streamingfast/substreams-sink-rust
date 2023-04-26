use anyhow::{format_err, Context, Error};
use futures03::StreamExt;
use pb::sf::substreams::v1::{module_output::Data, BlockScopedData, Package};
use prost::Message;
use std::{env, process::exit, sync::Arc};
use substreams::SubstreamsEndpoint;
use substreams_stream::{BlockResponse, SubstreamsStream};

mod pb;
mod substreams;
mod substreams_stream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = env::args();
    if args.len() != 4 {
        println!("usage: stream <endpoint> <spkg> <module>");
        println!();
        println!("The environment variable SUBSTREAMS_API_TOKEN must be set also");
        println!("and should contain a valid Substream API token.");
        exit(1);
    }

    let endpoint_url = env::args().nth(1).unwrap();
    let package_file = env::args().nth(2).unwrap();
    let module_name = env::args().nth(3).unwrap();

    let token_env = env::var("SUBSTREAMS_API_TOKEN").unwrap_or("".to_string());
    let mut token: Option<String> = None;
    if token_env.len() > 0 {
        token = Some(token_env);
    }

    let package = read_package(&package_file)?;
    let endpoint = Arc::new(SubstreamsEndpoint::new(&endpoint_url, token).await?);

    // FIXME: Handling of the cursor is missing here. It should be loaded from
    // somewhere (local file, database, cloud storage) and then `SubstreamStream` will
    // be able correctly resume from the right block.
    let cursor: Option<String> = None;

    let mut stream = SubstreamsStream::new(
        endpoint.clone(),
        cursor,
        package.modules.clone(),
        module_name.to_string(),
        0,
        0,
    );

    loop {
        match stream.next().await {
            None => {
                println!("Stream consumed");
                break;
            }
            Some(Ok(BlockResponse::New(data))) => {
                process_block_scoped_data(&data)?;

                // FIXME: Handling of the cursor is missing here. It should be saved each time
                // a full block has been correctly processed/persisted. The saving location
                // is your responsibility.
                //
                // By making it persistent, we ensure that if we crash, on startup we are
                // going to read it back from database and start back our SubstreamsStream
                // with it ensuring we are continuously streaming without ever losing a single
                // element.
                //
                // save_cursor(data.cursor)?;
            }
            Some(Err(err)) => {
                println!();
                println!("Stream terminated with error");
                println!("{:?}", err);
                exit(1);
            }
        }
    }

    Ok(())
}

fn process_block_scoped_data(data: &BlockScopedData) -> Result<(), Error> {
    let output_module = match &data.outputs[..] {
        [output] => output,
        x => {
            return Err(format_err!(
                "outputs should have exactly 1 element, got {}",
                x.len()
            ))
        }
    };

    match output_module.data.as_ref().unwrap() {
        Data::MapOutput(output) => {
            // You can decode the actual Any type received using this code:
            //
            //     use prost::Message;
            //     let value = Message::decode::<GeneratedStructName>(data.value.as_slice())?;
            //
            // Where GeneratedStructName is the Rust code generated for the Protobuf representing
            // your type.

            println!(
                "Block #{} - Payload {} ({} bytes)",
                data.clock.as_ref().unwrap().number,
                output.type_url.replace("type.googleapis.com/", ""),
                output.value.len()
            );

            Ok(())
        }
        Data::DebugStoreDeltas(_) => Err(format_err!(
            "invalid module output DebugStoreDeltasStoreDeltas, expecting MapOutput"
        )),
    }
}

fn read_package(file: &str) -> Result<Package, anyhow::Error> {
    let content = std::fs::read(file).context(format_err!("read package {}", file))?;
    Package::decode(content.as_ref()).context("decode command")
}
