# r2client
This is a Rust project ("cargo workspace" :nerd_face::point_up:) containing a few tools and utilities I created to
have an easier time interfacing with [Couldflare's R2 Bucket Storage](https://www.cloudflare.com/developer-platform/products/r2/).
Yeah.
Cool, right?

## r2client
A really really freaking simple API for uploading, downloading, and listing files from Couldflare R2 Buckets.
It's fast too, with minimal dependencies that WON'T add 40 seconds to your compile time!


Brief example of usage:
```rust
use r2client::{R2Bucket, R2Error};

#[tokio::main]
fn main() -> Result<(), R2Error> {
  // Assuming you have the required environment variables (as outlined in the totally 
  // existent documentation) set or in .env...
  let bucket = R2Bucket::new("my-bucket");

  bucket.upload_file("example.png").await?

  Ok(())
}
```
Would you look at that!
It's really that easy! Not to mention that is an asynchronous example too...

As of now it's absolutely usable.
There is room for some improvement on the backend, as well as the potential for various new features to be added.
Will absolutely be adding them.
Trust me bro.

A few notes:
 - Since the content-type is determined by the [local file's extension](./r2client/src/mimetypes.rs)...
   - When using tempfiles, you want them to preserve file extension
   - If a mimetype isn't known based off of file extension, then it will default to `application/octect-stream`
     - This is a mid quality list that I linked above, feel free to add to it or tell me I'm missing something
   - If you want to forgo or alter the file extension for one reason another, this is useless to you for now

The content type tomfoolery is just about it though, for most general purposes, this will do. (I hope to iron that out eventually)
I also hope that I would be much faster and easier to use than AWS SDKs, but I'm not benchmarking that.

## r2cli
Exactly what it sounds like.
It's a rudimentary wrapper for the r2client library.

I used it for some testing, and I imagine someone else out there can use it for verifying their R2 Credentials too lol.

No streamlined way to install it yet, sorry.
I'll probably take up a spot on crates.io just for you, one person who both stumbles across this and can use it.

## r2python
***LITERALLY DOES NOT EXIST YET!!!***

I will come back around and port this library to have a Python interface just for the experience, hopefully (that's
pretty much what this whole workspace is for).

For the time being, I don't know how and I want to get the project that I wanted this R2Client for done first.

## aws_sigv4
This part of the library is a Rust implementation of signing requests using AWS's SigV4, since R2 is "S3 compatible".
Barely any dependencies here. 
Take *THAT* AWS S3 SDK and your 400 freaking dependencies with 250 custom types for one request. Largo.

While I did make it it's own crate, just because I wanted to decouple it from the R2Client itself, it's still only
been tested using the S3 library, and it is not nearly as efficient or user-friendly as it could be.

This will only be useful for people who are using some API and need their requests signed with SigV4, either for
their own abstracted client or a specific one time use in a program or something atypical.

---
## Credit where credit's due
The libraries' APIs are inspired by [fayharinn's Python R2-Client](https://github.com/fayharinn/R2-Client), including their minimal dependency nature.
It's also where I blatantly stole the mimetypes from, but hey, it seemed AI generated!!!
<sub>oh yeah I'll commit my changes and submit a pull request for that one if I ever remember...although r2python should supersede it</sub>
