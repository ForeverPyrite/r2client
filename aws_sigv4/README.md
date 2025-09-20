# aws_signing library
Yeah so this is a library for signing things with AWS SigV4.
Pretty straightforward.
In fact, there is are only 3 methods on the public API
1. to make the client
2. Change the client region (cause why not?)
3. To transform an unsigned request that will be sent to an AWS compatible server to one with proper SigV4 headers
(With this wording, should I take a mutable reference to headers instead of...hmmm, we will see if I get bored enough to improve it myself.)

If I'm being completely honest, this was part of my own `r2client` that I wrote, which is uses S3
So I don't imagine that I come back to this to actually make a proper AWS Signing library.
I could imagine it being useful to *someone* who also wants to make an AWS abstraction that doesn't have 2 **BILLION** dependences.

## Usage

wait that's what the doc comments and `cargo doc` command is for lmao,
guess I'll get rid of this block

### Unexpected Errors
If you're getting errors related to invalid signatures and you're thinking "It's probably that damn crate I'm using from that newbie!",
you're probably right.
If you got here because crates.io search brought you here, nice.
Didn't mean for that to happen, sorry!
There are much more developed options out there, like [reqsign](https://github.com/apache/opendal-reqsign), which will also handle more
of the service related things for you.

Alternatively, you can try to stick with this crate (mistake)
Using log level trace with whatever logging crate you use will print out all steps of the AWS signing process, so you can follow along
with your favorite documentation/examples and figure out where things go wrong.
With this you can either raise an issue, or review the code yourself to implement a working solution.

## Todo

 - [ ] Create unit tests
 - [ ] Add option for session keys
 - [ ] Additional client or whatever for weirdos who want to use SigV4a
 - [ ] Perhaps drop `http` as a dependency?
 - [ ] Alternatively, have users pass a mutable reference to a HeaderMap instead of cloning and returning a new one.
  - This can kinda hurt ease of use...maybe, but I feel like the slight performance gain and easier management of custom headers makes it worth.
  - That's also acting like the crate is mature enough to do anything but provide the minimal headers for SigV4
 - [ ] SigV4a
  - Is this even really used?

## Contributing
What.

This crate is supposed to be a really simple way to sign AWS request headers and nothing else.
Again, I created it for my Cloudflare R2 (AWS S3) client and I imagine there are a few gaps for other services,
as well as some exceptions to the general programmatic things I saw in the documentation.

If you'd like to help improve it, maybe to make sign requests for some other AWS service, then I'd
appreciate if as specific unit test was added to ensure functionality.
This should be along with the other unit tests remaining intact  and passing (unless they are blatantly incompatible
with some AWS documentation)

The code base (one lib.rs file) is pretty straightforward, it very directly follows the outline that AWS's SigV4 documentation provides,
explicitly containing all prerequisite functions and having a function call for all 5 of the steps.
I did this since the other examples were hard to follow, just a chunk of code that doesn't explain itself at any point.
All prerequisite functions, along with functions that don't require any of the keys/credential fields (in SigV4Credentials)
are defined outside of the struct.

Gl;hf

# welp
figure I'd try my hand at writing some public stuff like this, even though I'm not planing on publishing this publicly, only
compiled in the r2client.
I feel...okay about it.
I still feel like this stuff will be so small no matter what that I can still put a little personality into it.

I am incredibly verbose, shocking I know, but I think that's a bad thing.
I should be more brief, and then more verbose when relevant within the code.
Outside of that though, I think this is...fine.
I just need to get better at less yapping.

I really think this crate is too intermingled with the R2 Client to be useful to anyone else, really, although I think this was 
*fine* for organizational purposes.
And who knows.
Maybe I'll need it again.
