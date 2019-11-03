//  Copyright (C) 2019  Éloïs SANCHEZ
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! JSON blocks chunk parser

use dubp_block_doc::parser::parse_json_block;
use dubp_block_doc::BlockDocument;
use failure::Error;
use std::io::Read;
use std::path::PathBuf;

/// Chunk file name begin
static CHUNK_FILE_NAME_BEGIN: &str = "chunk_";

/// Chunk file name end
static CHUNK_FILE_NAME_END: &str = "-250.json";

static CHUNK_SIZE: &usize = &250;

/// Open and parse one JSON Chunk
pub fn open_and_parse_one_json_chunk(
    json_chunks_path: &PathBuf,
    chunk_number: usize,
) -> (usize, Vec<BlockDocument>) {
    // Open chunk file
    let chunk_file_content_result = open_json_chunk_file(json_chunks_path, chunk_number);
    println!("json_chunks_path={:?}", json_chunks_path);
    if chunk_file_content_result.is_err() {
        panic!("Fail to open chunk file n°{}", chunk_number);
    }

    // Parse chunk file content
    let blocks_result = parse_json_chunk(&chunk_file_content_result.expect("safe unwrap"));
    let blocks = match blocks_result {
        Ok(blocks) => blocks,
        Err(e) => {
            panic!("Fail to parse chunk file n°{} : {}", chunk_number, e);
        }
    };
    (chunk_number, blocks)
}

fn open_json_chunk_file(
    json_chunks_path: &PathBuf,
    chunk_number: usize,
) -> std::io::Result<(String)> {
    let mut chunk_file_path = json_chunks_path.clone();
    chunk_file_path.push(&format!(
        "{}{}{}",
        CHUNK_FILE_NAME_BEGIN, chunk_number, CHUNK_FILE_NAME_END
    ));
    let file = std::fs::File::open(chunk_file_path)?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

/// Parse json chunk into BlockDocument Vector
fn parse_json_chunk(json_chunk_content: &str) -> Result<Vec<BlockDocument>, Error> {
    let mut block_doc_vec = Vec::with_capacity(*CHUNK_SIZE);

    let json_value = json_pest_parser::parse_json_string(json_chunk_content)?;
    if let Some(json_object) = json_value.to_object() {
        if let Some(blocks) = json_object.get("blocks") {
            if let Some(blocks_array) = blocks.to_array() {
                for json_block in blocks_array {
                    block_doc_vec.push(parse_json_block(json_block)?);
                }
            } else {
                panic!("Fail to parse json chunk : field \"blocks\" must be an array !");
            }
        } else {
            panic!("Fail to parse json chunk : field \"blocks\" don't exist !");
        }
    } else {
        panic!("Fail to parse json chunk : json root node must be an object !");
    }

    Ok(block_doc_vec)
}
