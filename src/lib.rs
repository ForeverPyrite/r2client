mod aws_signing;
mod error;
mod mimetypes;
pub use error::R2Error;

mod _async;
#[cfg(feature = "async")]
pub use _async::{R2Bucket, R2Client};

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(test)]
mod test {
    // use crate::{R2Bucket, R2Client, sync};

    #[test]
    fn test() {}
}
