//  Copyright (C) 2018  The Durs Project Developers.
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

use crate::constants::*;
use crate::sync::*;
use dubp_documents::parsers::blocks::parse_json_block;
use dubp_documents::Blockstamp;
use durs_common_tools::fatal_error;
use failure::Error;
use rayon::prelude::*;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use threadpool::ThreadPool;

/// Number of chunk parsed before sending them to the apply workers
static CHUNKS_STEP: &'static usize = &16;

/// Json reader worker
pub fn json_reader_worker(
    pool: &ThreadPool,
    profile: String,
    sender_sync_thread: mpsc::Sender<MessForSyncThread>,
    json_chunks_path: PathBuf,
    end: Option<u32>,
) {
    // Lauch json reader thread
    pool.execute(move || {
        let ts_job_begin = SystemTime::now();

        // Get list of json chunk files
        let chunks_set = get_chunks_set(&json_chunks_path);
        if chunks_set.is_empty() {
            fatal_error("json_files_path directory is empty !");
        }

        // Get max chunk number and max block id
        let (max_chunk_number, max_block_id): (usize, u32) = if let Some(end) = end {
            (end as usize / (*crate::constants::CHUNK_SIZE), end)
        } else {
            (
                chunks_set.len() - 1,
                (chunks_set.len() * (*crate::constants::CHUNK_SIZE) - 1) as u32,
            )
        };

        // Verify if max chunk exist
        if chunks_set.get(&max_chunk_number).is_none() {
            fatal_error(&format!("Missing chunk file n°{}", max_chunk_number));
        };

        // Open chunk file
        let chunk_file_content_result = open_json_chunk_file(&json_chunks_path, max_chunk_number);
        if chunk_file_content_result.is_err() {
            fatal_error(&format!("Fail to open chunk file n°{}", max_chunk_number));
        }

        // Parse chunk file content
        let blocks_result = parse_json_chunk(&chunk_file_content_result.expect("safe unwrap"));
        let last_chunk_blocks = match blocks_result {
            Ok(blocks) => blocks,
            Err(e) => {
                fatal_error(&format!(
                    "Fail to parse chunk file n°{} : {}",
                    max_chunk_number, e,
                ));
                unreachable!();
            }
        };

        if last_chunk_blocks.is_empty() {
            fatal_error("Last chunk is empty !");
        }

        let last_block = last_chunk_blocks
            .get(max_block_id as usize % *crate::constants::CHUNK_SIZE)
            .expect("safe unwrap because not empty");

        // Send TargetBlockcstamp
        sender_sync_thread
            .send(MessForSyncThread::Target(
                last_block.currency.clone(),
                last_block.blockstamp(),
            ))
            .expect("Fatal error : sync_thread unrechable !");

        // Get current local blockstamp
        debug!("Get local current blockstamp...");
        let db_path = duniter_conf::get_blockchain_db_path(&profile, &last_block.currency);
        let blocks_databases = BlocksV10DBs::open(Some(&db_path));
        let current_blockstamp: Blockstamp =
            durs_blockchain_dal::readers::block::get_current_blockstamp(&blocks_databases)
                .expect("ForksV10DB : RustBreakError !")
                .unwrap_or_default();
        info!("Local current blockstamp = {}", current_blockstamp);

        // Get first chunk number
        let first_chunk_number: usize =
            current_blockstamp.id.0 as usize / *crate::constants::CHUNK_SIZE;

        // Parse chunks
        let mut begin_chunk_number = first_chunk_number;
        while begin_chunk_number <= max_chunk_number {
            let last_chunk_number = if begin_chunk_number + *CHUNKS_STEP < max_chunk_number + 1 {
                begin_chunk_number + *CHUNKS_STEP
            } else {
                max_chunk_number + 1
            };
            let chunks_numbers: Vec<_> = (begin_chunk_number..last_chunk_number).collect();
            let mut chunks_blocks: HashMap<usize, Vec<BlockDocument>> = chunks_numbers
                .par_iter()
                .map(|chunk_number| treat_once_json_chunk(&json_chunks_path, *chunk_number))
                .collect();

            // Send blocks
            for chunk_number in chunks_numbers {
                for block in chunks_blocks
                    .remove(&chunk_number)
                    .expect("Dev error: sync: chunk_blocks not contain key chunk_number !")
                {
                    // Verify if the block number is within the expected interval
                    let block_id = block.blockstamp().id;
                    if (block_id > current_blockstamp.id && block_id.0 <= max_block_id)
                        || (block_id.0 == 0 && current_blockstamp == Blockstamp::default())
                    {
                        // Send block document
                        sender_sync_thread
                            .send(MessForSyncThread::BlockDocument(block))
                            .expect("Fatal error : sync_thread unrechable !");
                    }
                }
            }

            begin_chunk_number += *CHUNKS_STEP;
        }

        sender_sync_thread
            .send(MessForSyncThread::DownloadFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let ts_job_duration = SystemTime::now()
            .duration_since(ts_job_begin)
            .expect("duration_since error");
        info!(
            "ts_job_duration={},{:03} seconds.",
            ts_job_duration.as_secs(),
            ts_job_duration.subsec_millis()
        );
    });
}

/// Treat one JSON Chunk
fn treat_once_json_chunk(
    json_chunks_path: &PathBuf,
    chunk_number: usize,
) -> (usize, Vec<BlockDocument>) {
    // Open chunk file
    let chunk_file_content_result = open_json_chunk_file(json_chunks_path, chunk_number);
    if chunk_file_content_result.is_err() {
        fatal_error(&format!("Fail to open chunk file n°{}", chunk_number));
    }

    // Parse chunk file content
    let blocks_result = parse_json_chunk(&chunk_file_content_result.expect("safe unwrap"));
    let blocks = match blocks_result {
        Ok(blocks) => blocks,
        Err(e) => {
            fatal_error(&format!(
                "Fail to parse chunk file n°{} : {}",
                chunk_number, e,
            ));
            panic!(); // for compilator
        }
    };
    (chunk_number, blocks)
}

/// Parse json chunk into BlockDocument Vector
fn parse_json_chunk(json_chunk_content: &str) -> Result<Vec<BlockDocument>, Error> {
    let mut block_doc_vec = Vec::with_capacity(*crate::constants::CHUNK_SIZE);

    let json_value = json_pest_parser::parse_json_string(json_chunk_content)?;
    if let Some(json_object) = json_value.to_object() {
        if let Some(blocks) = json_object.get("blocks") {
            if let Some(blocks_array) = blocks.to_array() {
                for json_block in blocks_array {
                    block_doc_vec.push(parse_json_block(json_block)?);
                }
            } else {
                fatal_error("Fail to parse json chunk : field \"blocks\" must be an array !");
            }
        } else {
            fatal_error("Fail to parse json chunk : field \"blocks\" don't exist !");
        }
    } else {
        fatal_error("Fail to parse json chunk : json root node must be an object !");
    }

    Ok(block_doc_vec)
}

fn get_chunks_set(dir: &Path) -> HashSet<usize> {
    let json_chunk_file_list_result = fs::read_dir(dir);
    if json_chunk_file_list_result.is_err() {
        error!("Fail to read dir json_files_path !");
        panic!("Fail to read dir json_files_path !");
    }

    let mut chunks_set = HashSet::new();

    for dir_entry in json_chunk_file_list_result.expect("Dev error: err case must be treat before.")
    {
        if let Ok(dir_entry) = dir_entry {
            if let Ok(file_name) = dir_entry.file_name().into_string() {
                let file_name_len = file_name.len();

                if let Ok(file_type) = dir_entry.file_type() {
                    if file_type.is_file()
                        && file_name[0..CHUNK_FILE_NAME_BEGIN.len()] == *CHUNK_FILE_NAME_BEGIN
                        && file_name[file_name_len - CHUNK_FILE_NAME_END.len()..]
                            == *CHUNK_FILE_NAME_END
                    {
                        let chunk_number_result: Result<usize, std::num::ParseIntError> = file_name
                            [CHUNK_FILE_NAME_BEGIN.len()
                                ..file_name_len - CHUNK_FILE_NAME_END.len()]
                            .parse();

                        if let Ok(chunk_number) = chunk_number_result {
                            chunks_set.insert(chunk_number);
                        }
                    }
                }
            }
        }
    }

    chunks_set
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
