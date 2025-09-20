## For release:
 - [ ] Create a crate::Result that is Result<u8, R2Error>, and have Ok(status_code)
 - [ ] Consider dropping more dependencies, using hyper or some lower level stuff for async, and then http for blocking
 - [ ] A way to view the file contents (UTF-8 valid) would be cool
 - [ ] Add functions that will list files with their metadata (perhaps a simple R2File type?)
 - [ ] Clear out all all print statements and consider logging (this is a library, after all)

## Dev (since we're so back):
 - [X] Update the sync library
 - [X] Make a .env with test-bucket creds
 - [X] Actually test the damn thing

