use anyhow::Result;
use anyhow::anyhow;
use tokio::io::AsyncWriteExt as _;
use tokio::io::AsyncReadExt as _;
use futures_util::StreamExt as _;

pub mod cidr;
pub mod container;
pub mod network;