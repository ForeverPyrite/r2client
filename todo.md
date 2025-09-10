Okay I think I did everything, so to clean up:

# Fatal Bug
When building the headers for the async request (_async::r2client::tests::test_create_headers), the final header list is missing the host header.

Check and see if the `aws_signing` function `create_canonical_request_headers`'s output is getting tossed.
The output of that function includes request and headers with the host headers in there.
I imagine that, instead of parsing that string to turn into a header map or whatever the `signature` function returns, it is creating it's own "content-type" and "x-amz-date" headers and pushing them to it's own vector
I needed to come here to think cause I'm tired and would've fucked it up if I just brute forced it instead of critically thought about it

The expected solution is taking the "request", and turning in into a "HeaderMap", since we aren't low enough level to pass unsafe strings by ourselves.
Technically the output of the canonical_request is correct:
Request: `"PUT\n/bucket/key\n\ncontent-type:application/octet-stream\nhost:example.r2.cloudflarestorage.com\nx-amz-date:20250907T043710Z\n\ncontent-type;host;x-amz-date\ndeadbeef"`
However it's not in the format of the vector of strings, and also I don't think there should be a new line after put, but there should be two before the content type. Not sure.
But that's what the request looks like, then the headers that we could parse as well that the same function returns are as follows:
Headers: `"content-type:application/octet-stream\nhost:example.r2.cloudflarestorage.com\nx-amz-date:20250907T043710Z\n"`
Which seems to match up, however the final signature that the main `signature` function returns is:
Headers: `[("content-type", "application/octet-stream"), ("x-amz-date", "20250907T043710Z"), ("authorization", "AWS4-HMAC-SHA256 Credential=AKIAEXAMPLE/20250907/us-east-1/s3/aws4_request, SignedHeaders=content-type;host;x-amz-date, Signature=cfbed04d8dbd8610b9f621c72d7e86f0dae2ffac3174835b4a1ff23ad072893f")]`
So my assumption is that the `signature` function is making it's own headers and dropping the canonical ones instead of parsing one of the returned strings into one.
This is happening before the "authorization" headers (the only headers that the main function should be adding itself) are pushed to the current vector
I think the headers should be parsed into a Vec<(String, String)>, because that would allow the R2Client to turn the request into whatever header map it wants, or a string, which would hopefully make this AWS signing thing work for other services too, although I won't advertise it as it's own library or anything.

Okay after saying all that, surely I could've just fixed it by now and will remember it in the later.

- [X] Update the sync library
- [X] Make a .env with test-bucket creds
- [ ] Actually test the damn thing
- [ ] Cry (ALL OF THAT WORK, FOR WHAT!? A SINGLE `main.rs` ON GITHUB!?)

