use clap::{Parser, Subcommand};
use fusio::path::Path;
use tokio::fs;
use tonbo::{executor::tokio::TokioExecutor, DbOption, Record, DB};

/// Simple record schema for testing persistence
#[derive(Record, Debug, Clone)]
pub struct TestRecord {
    #[record(primary_key)]
    id: u32,
    value: String,
}

#[derive(Parser)]
#[command(name = "persistence_test")]
#[command(about = "Test data persistence in Tonbo", long_about = None)]
struct Cli {
    /// Path to the database directory
    #[arg(long, default_value = "./test_db")]
    db_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write data to the database using a transaction
    Write {
        /// ID of the record to write
        #[arg(long)]
        id: u32,
        /// Value to store
        #[arg(long)]
        value: String,
    },
    /// Read data from the database
    Read {
        /// ID of the record to read (if not specified, reads all)
        #[arg(long)]
        id: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Ensure the database directory exists
    fs::create_dir_all(&cli.db_path).await?;

    let options = DbOption::new(Path::from_filesystem_path(&cli.db_path)?, &TestRecordSchema);

    match cli.command {
        Commands::Write { id, value } => {
            println!("Opening database at: {}", cli.db_path);
            let db = DB::new(options, TokioExecutor::current(), TestRecordSchema).await?;

            println!("Starting transaction...");
            let mut txn = db.transaction().await;

            println!("Inserting record: id={}, value={}", id, &value);
            txn.insert(TestRecord {
                id,
                value: value.clone(),
            });

            println!("Committing transaction...");
            txn.commit().await?;

            println!("Transaction committed successfully!");
            println!("now exiting");
        }
        Commands::Read { id } => {
            println!("Opening database at: {}", cli.db_path);
            let db = DB::<TestRecord, _>::new(options, TokioExecutor::current(), TestRecordSchema)
                .await?;

            println!("Looking for record with id={}...", id);

            let result = db
                .get::<String>(&id, |entry| {
                    let record = entry.get();
                    Some(format!("Found: id={}, value={:?}", record.id, record.value))
                })
                .await?;

            match result {
                Some(msg) => println!("{}", msg),
                None => println!("Record with id={} NOT FOUND!", id),
            }
        }
    }

    Ok(())
}
