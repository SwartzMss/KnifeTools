use clap::Parser;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs;
use std::time::Instant;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "File Splitter")]
#[command(about = "Splits a large file into smaller files with a specified number of lines")]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(short, long, default_value_t = 5000)]
    lines: usize,
}

async fn split_file_by_lines(file_path: &str, lines_per_file: usize) -> io::Result<()> {
    let metadata = fs::metadata(file_path).await?;
    let file_size = metadata.len();
    info!("File size: {} bytes", file_size);
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.split(b'\n');

    let mut file_count = 0;
    let mut line_count = 0;
    let mut current_file: Option<File> = None;

    let start_time = Instant::now();

    while let Some(line) = lines.next_segment().await? {
        if line_count % lines_per_file == 0 {
            file_count += 1;
            let new_file_name = format!("{}_part_{}.log", file_path, file_count);
            current_file = Some(File::create(new_file_name).await?);
        }

        if let Some(ref mut file) = current_file {
            file.write_all(&line).await?;
            file.write_all(b"\n").await?;
        }

        line_count += 1;
    }

    info!("Total split files created: {}", file_count);
    info!("Total lines processed: {}", line_count);

    let duration = start_time.elapsed();
    info!("File splitting completed in: {:.2?}", duration);
    info!("Split job has been done!");

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_ansi(false)
        .init();

    let args = Args::parse();

    if let Err(e) = split_file_by_lines(&args.file, args.lines).await {
        error!("Error occurred: {}", e);
    }
}
