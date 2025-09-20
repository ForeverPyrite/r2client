# aws_sigv4
Used by the r2client's R2Client to sign requests.
So cool, ikr?

## Todo

 - [ ] Replace the http::Uri with a &str and parse it instead...duh (unless that doesn't exist, but it 100% does)
 - [ ] Use a mutable reference to a HeaderMap instead of that UGLY UGLY Vec<(String, String)> format
  - Although I'm not sure how much better this will be with the whole sort in alphabetical order, but it'll be a hell of a lot cooler
- [ ] The unit tests lol
 - [ ] Introduce the option for session tokens
 - [ ] SigV4a?
