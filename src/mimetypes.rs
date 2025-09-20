pub fn get_mimetype(key: &str) -> &'static str {
    match key {
        // Image formats
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".svg" => "image/svg+xml",
        ".ico" => "image/x-icon",
        ".webp" => "image/webp",

        // Audio formats
        ".m4a" => "audio/x-m4a",
        ".mp3" => "audio/mpeg",
        ".wav" => "audio/wav",
        ".ogg" => "audio/ogg",

        // Video formats
        ".mp4" => "video/mp4",
        ".avi" => "video/x-msvideo",
        ".mov" => "video/quicktime",
        ".flv" => "video/x-flv",
        ".wmv" => "video/x-ms-wmv",
        ".webm" => "video/webm",

        // Document formats
        ".pdf" => "application/pdf",
        ".doc" => "application/msword",
        ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".ppt" => "application/vnd.ms-powerpoint",
        ".pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ".xls" => "application/vnd.ms-excel",
        ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".txt" => "text/plain",

        // Web formats
        ".html" => "text/html",
        ".css" => "text/css",
        ".js" => "application/javascript",
        ".json" => "application/json",
        ".xml" => "application/xml",

        // Other formats
        ".csv" => "text/csv",
        ".zip" => "application/zip",
        ".tar" => "application/x-tar",
        ".gz" => "application/gzip",
        ".rar" => "application/vnd.rar",
        ".7z" => "application/x-7z-compressed",
        ".eps" => "application/postscript",
        ".sql" => "application/sql",
        ".java" => "text/x-java-source",
        _ => "application/octet-stream",
    }
}

pub fn get_mimetype_from_fp(file_path: &str) -> &str {
    // Sorry I just really wanted to get it done in a one liner.
    // This splits a filepath based off ".", in reverse order, so that the first element will
    // be the file extension (e.g. "~/.config/test.jpeg" becomes "jpeg")
    // This is formated back to ".jpeg" because it's how the match statement is working.
    // I could very easily change it but idk it was an interesting thing.
    //
    // Hey, so maybe you should change the match statement to not care about the '.'?
    // Then again this is just being used for this project, so I guess it doesn't really matter
    get_mimetype(&format!(
        ".{}",
        file_path
            .rsplit(".")
            .next()
            .unwrap_or("time_to_be_an_octet_stream_lmao")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_mime_test() {
        assert_eq!(get_mimetype(".tar"), "application/x-tar");
    }

    #[test]
    fn default_mime_test() {
        assert_eq!(get_mimetype(".bf"), "application/octet-stream");
    }

    #[test]
    fn mime_from_file() {
        assert_eq!(get_mimetype_from_fp("test.ico"), "image/x-icon");
    }

    #[test]
    fn mime_from_file_path() {
        assert_eq!(
            get_mimetype_from_fp("/home/testuser/Documents/test.pdf"),
            "application/pdf"
        );
        assert_eq!(
            get_mimetype_from_fp("./bucket_test/bucket_test_upload.txt"),
            "text/plain"
        )
    }

    #[test]
    fn no_ext() {
        assert_eq!(
            get_mimetype_from_fp("edge_case_lmao"),
            "application/octet-stream"
        )
    }
}
