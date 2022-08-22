#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::meta_storage_reader::hash_bytes_with;
use frame_benchmarking::benchmarks;
use frame_metadata::StorageHasher;
use sp_std::prelude::*;

pub struct Module<T: pallet_evm::Config>(pallet_evm::Module<T>);

pub trait Config: pallet_evm::Config {}

benchmarks! {
    blake2_128 {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Blake2_128) }
    blake2_256 {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Blake2_256) }
    blake2_128_concat {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Blake2_128Concat) }
    twox_128 {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Twox128) }
    twox_256 {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Twox256) }
    twox_64_concat {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Twox64Concat) }
    identity {
        let l in 0..1000;
        let data: Vec<_> = (0..l).map(|i| i as u8).collect();
    }: { hash_bytes_with(&data, &StorageHasher::Identity) }
}
