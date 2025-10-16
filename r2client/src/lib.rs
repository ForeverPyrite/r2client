mod error;
mod mimetypes;
// Should r2client::Result be r2client::R2Result just in case someone does a glob import or
// something? Or should that be left to the user of the library to use the "as" keyword?
pub use error::{R2Error, Result};

mod _async;
#[cfg(feature = "async")]
pub use _async::{R2Bucket, R2Client};

#[cfg(feature = "sync")]
pub mod sync;
