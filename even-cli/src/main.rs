use std::time::Duration;

use alloy_primitives::{FixedBytes, U256};
use anyhow::{ensure, Context, Result};
use bonsai_sdk::alpha as bonsai_sdk;
use clap::{Parser, Subcommand};
use risc0_zkvm::{serde::to_vec, Receipt};

mod contract;
mod seal;
mod tx_sender;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Deploy,
    TestVector {
        #[clap(short, long, require_equals = true)]
        image_id: String,

        #[clap(short, long, require_equals = true)]
        number: U256,
    },
    SendTx {
        #[clap(short, long, require_equals = true)]
        image_id: String,

        #[clap(short, long, require_equals = true)]
        chain_id: u64,

        #[clap(short, long, require_equals = true)]
        rpc_url: String,

        #[clap(short, long, require_equals = true)]
        contract: String,

        #[clap(short, long, require_equals = true)]
        number: U256,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args.command {
        Command::Deploy => deploy().await?,
        Command::TestVector { image_id, number } => {
            let (seal, post_state_digest) = prove(image_id, number).await?;

            println!("number: {}", number);
            print!("seal: ");
            for b in &seal {
                print!("\\x{:02x}", b);
            }
            println!("");
            println!("post_state_digest: {}", post_state_digest);
        }
        Command::SendTx {
            image_id,
            chain_id,
            rpc_url,
            contract,
            number,
        } => {
            let private_key = std::env::var("ETH_WALLET_PRIVATE_KEY")?;
            let tx_sender = tx_sender::TxSender::new(chain_id, &rpc_url, &private_key, &contract)?;

            let (seal, post_state_digest) = prove(image_id, number).await?;

            tx_sender
                .send(contract::set(number, seal, post_state_digest))
                .await?;
        }
    }

    Ok(())
}

fn compute_image_id(elf: &[u8]) -> String {
    let program = risc0_zkvm::Program::load_elf(elf, risc0_zkvm::GUEST_MAX_MEM as u32)
        .expect("Could not load ELF");
    let image = risc0_zkvm::MemoryImage::new(&program, risc0_zkvm::PAGE_SIZE as u32)
        .expect("Could not create memory image");
    hex::encode(image.compute_id())
}

async fn deploy() -> Result<()> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;

    // Compute the image_id, then upload the ELF with the image_id as its key.
    let image_id = compute_image_id(even_guests::IS_EVEN_ELF);
    client.upload_img(&image_id, even_guests::IS_EVEN_ELF.to_vec())?;
    println!("{}", image_id);

    Ok(())
}

async fn prove(image_id: String, number: U256) -> Result<(Vec<u8>, FixedBytes<32>)> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;

    // Prepare input data and upload it.
    let input_id = {
        let input_data = to_vec(&number).unwrap();
        let input_data = bytemuck::cast_slice(&input_data).to_vec();
        client.upload_input(input_data)?
    };

    // Fetch the receipt
    let session = client.create_session(image_id, input_id)?;
    eprintln!("Created session: {}", session.uuid);
    let receipt = loop {
        let res = session.status(&client)?;
        if res.status == "RUNNING" {
            eprintln!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;

            break receipt;
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
    };

    // Verify the receipt
    {
        receipt
            .verify(even_guests::IS_EVEN_ID)
            .expect("Receipt verification failed");
        println!("Journal digest: {:?}", receipt.get_metadata()?.output);

        let committed_number = U256::from_be_slice(receipt.journal.bytes.as_slice());
        ensure!(
            number == committed_number,
            "Commitment mismatch: {} != {}",
            number,
            committed_number
        );

        eprintln!("Receipt verified");
    }

    // Fetch the snark
    let snark_session = client.create_snark(session.uuid)?;
    eprintln!("Created snark session: {}", snark_session.uuid);
    let snark_receipt = loop {
        let res = snark_session.status(&client)?;
        match res.status.as_str() {
            "RUNNING" => {
                eprintln!("Current status: {} - continue polling...", res.status,);
                std::thread::sleep(Duration::from_secs(15));
                continue;
            }
            "SUCCEEDED" => {
                break res.output.expect("No snark generated :(");
            }
            _ => {
                panic!(
                    "Workflow exited: {} err: {}",
                    res.status,
                    res.error_msg.unwrap_or_default()
                );
            }
        }
    };

    // Convert from Bonsai snark format to Eth snark format
    let seal = seal::Seal::abi_encode(snark_receipt.snark).context("Read seal")?;
    let post_state_digest: FixedBytes<32> = snark_receipt
        .post_state_digest
        .as_slice()
        .try_into()
        .context("Read post_state_digest")?;

    Ok((seal, post_state_digest))
}
