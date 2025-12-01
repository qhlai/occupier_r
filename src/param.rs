extern crate clap;
use crate::memory::MemOccupyConfig;
use crate::storage::FileOccupyConfig;
use clap::{App, Arg, ArgMatches};
use systemstat::Platform;

pub const STORAGE_TEMP_DIR: &str = "./storage_bucket_occupier_r_dsd1sakmf";

#[derive(Debug, Clone)]
pub struct OccupyConfig {
    pub cpu_rate: u16,
    pub storage_config: FileOccupyConfig,
    pub memory_config: MemOccupyConfig,
    pub granularity: usize,
    pub flush_delay: f32,
    pub display_system: bool,
}

// // 资源占用配置结构体（精简字段，明确语义）
// #[derive(Debug, Clone)]
// pub struct CommOccupyConfig {
//     pub rate: u16,          // 目标占用比例（0-100）
//     pub target_part_count: u64,  // 目标分区数量
//     pub current_part_count: u64, // 当前分区数量
//     pub part_size_mb: u16,  // 单个分区大小（MB）
//     pub used_parts: Vec<u32>,    // 已使用的分区ID（用于存储文件命名/内存块追踪）
// }

// impl CommOccupyConfig {
//     // 构造函数：初始化占用配置
//     fn new(rate: u16, part_size_mb: u16) -> Self {
//         Self {
//             rate,
//             target_part_count: 0,
//             current_part_count: 0,
//             part_size_mb,
//             used_parts: Vec::new(),
//         }
//     }

//     // 单个分区的字节数（避免重复计算）
//     fn part_size_bytes(&self) -> u64 {
//         self.part_size_mb as u64 * 1024 * 1024
//     }

//     // 当前已占用的总字节数
//     fn total_used_bytes(&self) -> u64 {
//         self.current_part_count as u64 * self.part_size_bytes()
//     }
// }

/// 解析命令行参数（流式 API 构建，无生命周期问题）
pub fn parse_args() -> ArgMatches {
    App::new("资源占用工具")
        .version("1.0.0")
        .author("Your Name <your.email@example.com>")
        .about("指定比例/大小占用系统CPU、内存、存储资源")
        // CPU 占用比例（0-100）
        .arg(
            Arg::new("cpu")
                .short('c')
                .long("cpu")
                .value_name("CPU_RATE")
                .help("CPU 目标占用比例（0-100，默认 0）")
                .takes_value(true),
        )
        // 内存占用比例（0-100）
        .arg(
            Arg::new("memory")
                .short('m')
                .long("memory")
                .value_name("MEMORY_RATE")
                .help("内存目标占用比例（0-100，默认 0）")
                .takes_value(true),
        )
        // 存储占用比例（0-100）
        .arg(
            Arg::new("storage")
                .short('s')
                .long("storage")
                .value_name("STORAGE_RATE")
                .help("存储目标占用比例（0-100，默认 0）")
                .takes_value(true),
        )
        // .arg(
        //     Arg::new("Storage")
        //         .short('S')
        //         .long("storage")
        //         .value_name("STORAGE_RATE")
        //         .help("存储目标占用比例（0-100，默认 0）")
        //         .takes_value(true),
        // )
        // 单个内存分区大小（MB）
        .arg(
            Arg::new("memory_size")
                .long("memory-size")
                .value_name("MB")
                .help("单个内存分区大小（MB，默认 10）")
                .takes_value(true),
        )
        // 单个存储分区大小（MB）
        .arg(
            Arg::new("storage_size")
                .long("storage-size")
                .value_name("MB")
                .help("单个存储分区大小（MB，默认 50）")
                .takes_value(true),
        )
        // 调整粒度（控制更新频率）
        .arg(
            Arg::new("granularity")
                .long("granularity")
                .value_name("COUNT")
                .help("资源调整粒度（默认 50）")
                .takes_value(true),
        )
        // 刷新延迟（秒）
        .arg(
            Arg::new("delay")
                .short('d')
                .long("delay")
                .value_name("SECONDS")
                .help("资源状态刷新延迟（秒，默认 3.0s）")
                .takes_value(true),
        )
        // 显示系统信息
        .arg(
            Arg::new("status")
                .short('t')
                .long("status")
                .help("启动时显示系统信息概览（默认 false）")
                .takes_value(false),
        )
        .get_matches()
}

/// 解析配置参数（集中处理，便于维护）
// 修复后的 parse_config 函数
pub fn parse_config(args: &ArgMatches) -> OccupyConfig {
    // CPU 占用比例（默认 0）
    let cpu_rate = args
        .value_of("cpu")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // 内存占用配置（默认比例 0，单分区 16MB）
    let memory_rate = args
        .value_of("memory")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let memory_part_size = args
        .value_of("memory_size")
        .and_then(|s| s.parse().ok())
        .unwrap_or(16);
    let memory_config = MemOccupyConfig::new(memory_rate, memory_part_size);

    // 存储占用配置（默认比例 0，单分区 16MB）
    let storage_rate = args
        .value_of("storage")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let storage_part_size = args
        .value_of("storage_size")
        .and_then(|s| s.parse().ok())
        .unwrap_or(64);
    let storage_config = FileOccupyConfig::new(storage_rate, storage_part_size);

    // 调整粒度（默认 50，避免 0 导致除零错误）
    let granularity = args
        .value_of("granularity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50); // 原默认 0 可能有风险，改为 50 更合理

    // 刷新延迟（默认 0.2 秒，原默认 3 秒可能过久）
    let flush_delay = args
        .value_of("delay")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3.0);

    // 显示系统信息（flag 类型，用 is_present 判断是否传入）
    let display_system = args.is_present("status");

    // 初始化并返回配置结构体（核心修复：添加返回值）
    OccupyConfig {
        cpu_rate,
        storage_config,
        memory_config,
        granularity,
        flush_delay,
        display_system,
    }
}

// // 补充：程序退出时清理存储目录（避免残留文件）
// impl Drop for CommOccupyConfig {
//     fn drop(&mut self) {
//         if self.rate > 0 && std::path::Path::new(STORAGE_TEMP_DIR).exists() {
//             let _ = std::fs::remove_dir_all(STORAGE_TEMP_DIR);
//             println!("\n🗑️  程序退出，已清理存储临时目录");
//         }
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

impl OsType {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return OsType::Linux;
        #[cfg(target_os = "windows")]
        return OsType::Windows;
        #[cfg(target_os = "macos")]
        return OsType::MacOs;
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return OsType::Unknown;
    }
}
