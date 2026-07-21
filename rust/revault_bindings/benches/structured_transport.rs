//! Compares the retired Protobuf binding payload with its FlatBuffers replacement.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use prost::Message;
use std::hint::black_box;

#[allow(missing_docs, unsafe_op_in_unsafe_fn, unused_imports)]
mod transport {
    include!("../src/generated/revault_bindings_generated.rs");
}

use transport::revault::internal as fb;

#[derive(Clone, PartialEq, Message)]
struct ProtoEntry {
    #[prost(string, tag = "1")]
    path: String,
    #[prost(int32, tag = "2")]
    kind: i32,
    #[prost(uint64, tag = "3")]
    length: u64,
    #[prost(uint32, tag = "4")]
    permissions: u32,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoEntryList {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<ProtoEntry>,
}

fn fixtures(count: usize) -> (Vec<u8>, Vec<u8>) {
    let entries: Vec<_> = (0..count)
        .map(|index| ProtoEntry {
            path: format!("documents/year-2026/report-{index:04}.txt"),
            kind: 1,
            length: 32_768 + index as u64,
            permissions: 0o640,
        })
        .collect();
    let protobuf = ProtoEntryList {
        entries: entries.clone(),
    }
    .encode_to_vec();
    let object = fb::LockboxEntryListT {
        entries: Some(
            entries
                .into_iter()
                .map(|entry| fb::LockboxEntryT {
                    path: Some(entry.path),
                    kind: fb::LockboxEntryKind::FILE,
                    length: entry.length,
                    permissions: entry.permissions,
                })
                .collect(),
        ),
    };
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let root = object.pack(&mut builder);
    builder.finish(root, None);
    (protobuf, builder.finished_data().to_vec())
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("structured_result_decode");
    for count in [1usize, 100, 1_000] {
        let (protobuf, flatbuffer) = fixtures(count);
        group.throughput(criterion::Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("protobuf_owned", count),
            &protobuf,
            |b, bytes| {
                b.iter(|| {
                    let value = ProtoEntryList::decode(black_box(bytes.as_slice())).unwrap();
                    black_box(value.entries.last().map(|entry| entry.length))
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("flatbuffers_view", count),
            &flatbuffer,
            |b, bytes| {
                b.iter(|| {
                    // Result bytes came from the version-matched native library in this
                    // process, as they do for every language facade.
                    let value = unsafe {
                        flatbuffers::root_unchecked::<fb::LockboxEntryList>(black_box(bytes))
                    };
                    let entries = value.entries().unwrap();
                    black_box(entries.get(entries.len() - 1).length())
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("flatbuffers_owned", count),
            &flatbuffer,
            |b, bytes| {
                b.iter(|| {
                    let value = unsafe {
                        flatbuffers::root_unchecked::<fb::LockboxEntryList>(black_box(bytes))
                    }
                    .unpack();
                    black_box(
                        value
                            .entries
                            .and_then(|entries| entries.last().map(|entry| entry.length)),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
