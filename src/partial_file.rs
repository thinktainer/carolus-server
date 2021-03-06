// Copyright (c) 2017 Simon Dickson
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cmp;
use std::fs::File;
use std::io::{self, BufReader, SeekFrom, Seek, Read};
use std::str::FromStr;
use std::path::{Path, PathBuf};

use rocket::request::Request;
use rocket::response::{Response, Responder};
use rocket::http::Status;
use rocket::http::hyper::header::{ByteRangeSpec, ContentRangeSpec, AcceptRanges, RangeUnit, Range, ContentRange, ContentLength};

#[derive(Debug)]
pub enum PartialFileRange {
    AllFrom(u64),
    FromTo(u64,u64),
    Last(u64),
}

impl From<ByteRangeSpec> for PartialFileRange {
    fn from(b: ByteRangeSpec) -> PartialFileRange {
        match b {
            ByteRangeSpec::AllFrom(from) => PartialFileRange::AllFrom(from),
            ByteRangeSpec::FromTo(from, to) => PartialFileRange::FromTo(from, to),
            ByteRangeSpec::Last(last) => PartialFileRange::Last(last),
        }
    }
}

impl From<Vec<ByteRangeSpec>> for PartialFileRange {
    fn from(v: Vec<ByteRangeSpec>) -> PartialFileRange {
        match v.into_iter().next() {
            None => PartialFileRange::AllFrom(0),
            Some(byte_range) => PartialFileRange::from(byte_range),
        }
    }
}

#[derive(Debug)]
pub struct PartialFile {
    path: PathBuf,
    file: File
}

impl PartialFile {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<PartialFile> {
        let file = File::open(path.as_ref())?;
        Ok(PartialFile{ path: path.as_ref().to_path_buf(), file: file })
    }

    pub fn get_partial<Range>(self, response: &mut Response, range: Range) 
        where Range: Into<PartialFileRange> {
        use self::PartialFileRange::*;
        let metadata : Option<_> = self.file.metadata().ok();
        let file_length : Option<u64> = metadata.map(|m| m.len());
        let range : Option<(u64, u64)> = match (range.into(), file_length) {
            (FromTo(from, to), Some(file_length)) => {
                if from <= to && from < file_length {
                    Some((from, cmp::min(to, file_length - 1)))
                } else {
                    None
                }
            },
            (AllFrom(from), Some(file_length)) => {
                if from < file_length {
                    Some((from, file_length - 1))
                } else {
                    None
                }
            },
            (Last(last), Some(file_length)) => {
                if last < file_length {
                    Some((file_length - last, file_length - 1))
                } else {
                    Some((0, file_length - 1))
                }
            },
            (_, None) => None,
            
        };
        if let Some(range) = range {
            let content_range = ContentRange(ContentRangeSpec::Bytes {
                range: Some(range),
                instance_length: file_length,
            });
            let content_len = range.1 - range.0 + 1;
            response.set_header(ContentLength(content_len));
            response.set_header(content_range);
            let mut partial_content = BufReader::new(self.file);
            let _ = partial_content.seek(SeekFrom::Start(range.0));
            let result = partial_content.take(content_len);
            response.set_status(Status::PartialContent);
            response.set_streamed_body(result);
        } else {
            if let Some(file_length) = file_length {
                response.set_header(ContentRange(ContentRangeSpec::Bytes {
                    range: None,
                    instance_length: Some(file_length),
                }));
            };
            response.set_status(Status::RangeNotSatisfiable);
        };
    }
}

impl Responder<'static> for PartialFile {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        let mut response = Response::new();
        response.set_header(AcceptRanges(vec![RangeUnit::Bytes]));
        match req.headers().get_one("range") {
            Some (range) => {
                match Range::from_str(range) {
                    Ok(Range::Bytes(ref v)) => {
                        self.get_partial(&mut response, v.clone());
                        response.set_status(Status::PartialContent);
                    },
                    _ => {
                        response.set_status(Status::RangeNotSatisfiable);
                    },
                }
            },
            None => {
                response.set_streamed_body(BufReader::new(self.file));
            },
        }
        Ok(response)
    }
}

pub fn serve_partial(video_path: &Path) -> io::Result<PartialFile> {    
    PartialFile::open(video_path)
}
