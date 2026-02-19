//! Arrow schema definition and Parquet I/O for raw EVM event logs.
//!
//! Provides a single conversion path from alloy [`Log`]s directly to
//! columnar [`RecordBatch`]es — no intermediate row struct needed.

use std::path::Path;
use std::sync::{Arc, LazyLock};

use alloy::rpc::types::Log;
use anyhow::{Context, Result};
use arrow_array::{BooleanArray, RecordBatch, StringArray, UInt32Array, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::file::properties::WriterProperties;

/// Arrow schema mirroring the Ethereum `eth_getLogs` response structure.
static EVENT_SCHEMA: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new("block_number", DataType::UInt64, false),
        Field::new("tx_hash", DataType::Utf8, false),
        Field::new("tx_index", DataType::UInt32, false),
        Field::new("log_index", DataType::UInt32, false),
        Field::new("address", DataType::Utf8, false),
        Field::new("topic0", DataType::Utf8, false),
        Field::new("topic1", DataType::Utf8, true),
        Field::new("topic2", DataType::Utf8, true),
        Field::new("topic3", DataType::Utf8, true),
        Field::new("data", DataType::Utf8, false),
        Field::new("removed", DataType::Boolean, false),
    ]))
});

/// Convert RPC [`Log`]s directly into a columnar [`RecordBatch`].
///
/// Skips logs that are missing required fields (block number, tx hash, etc.).
/// Returns the batch together with the number of valid rows.
///
/// # Errors
///
/// Returns an error if the Arrow `RecordBatch` construction fails.
pub fn logs_to_batch(logs: &[Log]) -> Result<(RecordBatch, usize)> {
    let cap = logs.len();
    let mut block_numbers = Vec::with_capacity(cap);
    let mut tx_hashes = Vec::with_capacity(cap);
    let mut tx_indices = Vec::with_capacity(cap);
    let mut log_indices = Vec::with_capacity(cap);
    let mut addresses = Vec::with_capacity(cap);
    let mut topic0s = Vec::with_capacity(cap);
    let mut topic1s: Vec<Option<String>> = Vec::with_capacity(cap);
    let mut topic2s: Vec<Option<String>> = Vec::with_capacity(cap);
    let mut topic3s: Vec<Option<String>> = Vec::with_capacity(cap);
    let mut datas = Vec::with_capacity(cap);
    let mut removeds = Vec::with_capacity(cap);

    for log in logs {
        let (Some(bn), Some(th), Some(ti), Some(li)) = (
            log.block_number,
            log.transaction_hash,
            log.transaction_index,
            log.log_index,
        ) else {
            continue;
        };

        let topics = log.topics();
        let Some(t0) = topics.first() else {
            continue;
        };

        block_numbers.push(bn);
        tx_hashes.push(format!("{th:#x}"));
        #[allow(clippy::cast_possible_truncation)]
        {
            tx_indices.push(ti as u32);
            log_indices.push(li as u32);
        }
        addresses.push(format!("{:#x}", log.address()));
        topic0s.push(format!("{t0:#x}"));
        topic1s.push(topics.get(1).map(|t| format!("{t:#x}")));
        topic2s.push(topics.get(2).map(|t| format!("{t:#x}")));
        topic3s.push(topics.get(3).map(|t| format!("{t:#x}")));
        datas.push(format!("{}", log.data().data));
        removeds.push(log.removed);
    }

    let count = block_numbers.len();
    let batch = RecordBatch::try_new(
        Arc::clone(&EVENT_SCHEMA),
        vec![
            Arc::new(UInt64Array::from(block_numbers)),
            Arc::new(StringArray::from(tx_hashes)),
            Arc::new(UInt32Array::from(tx_indices)),
            Arc::new(UInt32Array::from(log_indices)),
            Arc::new(StringArray::from(addresses)),
            Arc::new(StringArray::from(topic0s)),
            Arc::new(StringArray::from(topic1s)),
            Arc::new(StringArray::from(topic2s)),
            Arc::new(StringArray::from(topic3s)),
            Arc::new(StringArray::from(datas)),
            Arc::new(BooleanArray::from(removeds)),
        ],
    )?;

    Ok((batch, count))
}

/// Return the maximum `block_number` value across all batches.
///
/// Used to determine per-contract sync progress independently of the
/// global cursor, preventing duplicate data on partial-failure re-runs.
#[must_use]
pub fn max_block_number(batches: &[RecordBatch]) -> Option<u64> {
    batches
        .iter()
        .filter(|b| b.num_rows() > 0)
        .filter_map(|batch| {
            let col = batch.column(0).as_any().downcast_ref::<UInt64Array>()?;
            col.values().iter().copied().max()
        })
        .max()
}

/// Read all existing record batches from a Parquet file.
///
/// Returns an empty vec if the file does not exist.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read(path: &Path) -> Result<Vec<RecordBatch>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| format!("reading parquet header: {}", path.display()))?
        .build()
        .with_context(|| format!("building parquet reader: {}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("reading batches from {}", path.display()))
}

/// Write record batches to a Parquet file using Zstd compression.
///
/// Uses atomic write (temp file + rename) to prevent corruption.
///
/// # Errors
///
/// Returns an error on I/O failure or if the Parquet writer rejects the data.
pub fn write(path: &Path, batches: &[RecordBatch]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("parquet.tmp");
    let file =
        std::fs::File::create(&tmp).with_context(|| format!("creating {}", tmp.display()))?;

    let props = WriterProperties::builder()
        .set_compression(parquet::basic::Compression::ZSTD(
            parquet::basic::ZstdLevel::try_new(3).context("invalid zstd level")?,
        ))
        .build();

    let mut writer = ArrowWriter::try_new(file, Arc::clone(&EVENT_SCHEMA), Some(props))?;
    for batch in batches {
        writer.write(batch)?;
    }
    writer.close()?;

    std::fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} → {}", tmp.display(), path.display()))?;

    Ok(())
}
