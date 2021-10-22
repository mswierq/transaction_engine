use std::error::Error;
use transaction_engine::accounts_base::serialize_accounts_base;
use transaction_engine::TransactionEngine;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let engine = TransactionEngine::new(&args[1]);
    let accounts = engine.process()?;
    let _ = serialize_accounts_base(&accounts, std::io::stdout())?;
    Ok(())
}
