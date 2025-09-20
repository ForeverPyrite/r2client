use clap::{Parser, Subcommand};
use dotenv::dotenv;
use r2client::{R2Error, sync::R2Bucket};
use std::path::PathBuf;

/// Extremely minimal CLI meant for testing R2 credentials and manual uploads/downloads
#[derive(Parser, Debug)]
#[clap(name = "r2cli")]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: R2Command,
}

#[derive(Debug, Subcommand)]
enum R2Command {
    /// Lists all files and folders in the R2 Bucket
    List,
    /// Downloads a given file from the remote file key to the local path
    Download {
        /// The file key pointing to the desired R2 object
        remote_path: String,
        /// The local path to store the download at
        local_path: PathBuf,
    },
    /// Uploads a given file locally and uploads it to the R2 Bucket
    Upload {
        /// The local path of the file to upload
        local_path: PathBuf,
        /// The file key that the file should be uploaded to
        remote_path: String,
    },
    /// Deletes an object within the bucket
    Delete {
        /// The file key of the object to be deleted
        remote_key: String,
    },
}

fn main() -> Result<(), R2Error> {
    dotenv().ok();
    let r2bucket = R2Bucket::new(
        std::env::var("R2_BUCKET")
            .expect("Please set the R2_BUCKET environment varible and try again"),
    );

    let cmd_args = Args::parse();

    match cmd_args.command {
        R2Command::List => {
            println!("{:#?}", r2bucket.list_files().expect("error listing files"));
        }
        R2Command::Download {
            remote_path: remote,
            local_path: local,
        } => {
            r2bucket
                .download_file(
                    &remote,
                    local.as_path().to_str().expect("invalid download path"),
                )
                .unwrap();
            println!("Downloaded file with key {remote} to {local:?}")
        }
        R2Command::Delete { remote_key: remote } => {
            r2bucket.delete_file(&remote).expect("couldn't delete file");
            println!("Deleted file at {remote}")
        }
        R2Command::Upload {
            local_path: local,
            remote_path: remote,
        } => {
            r2bucket
                .upload_file(local.to_str().expect("invalid download path"), &remote)
                .expect("couldn't upload file");
            println!("Uploaded {local:?} to {remote}");
        }
    }
    Ok(())
}
