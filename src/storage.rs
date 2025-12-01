// const STORAGE_TEMP_DIR: &str = "./storage_bucket_occupier_r_dsd1sakmf";
use std::{fs, fs::File, fs::OpenOptions, io, io::prelude::*, path::Path};
use systemstat::{saturating_sub_bytes, Platform, System};
extern crate vmm_sys_util;
use crate::param::STORAGE_TEMP_DIR;
use vmm_sys_util::fallocate::{fallocate, FallocateMode};

#[derive(Debug, Clone)]
pub struct FileOccupyConfig {
    pub rate: u16,               // 目标占用比例（0-100）
    pub target_part_count: u64,  // 目标分区数量
    pub current_part_count: u64, // 当前分区数量
    pub part_size_mb: u16,       // 单个分区大小（MB）
    pub used_parts: Vec<usize>,  // 已使用的分区ID（用于存储文件命名/内存块追踪）
    pub seg_part_data: Vec<u8>,
    // pub &System sys,
}

impl FileOccupyConfig {
    // 构造函数：初始化占用配置
    pub fn new(rate: u16, part_size_mb: u16) -> Self {
        Self {
            rate,
            target_part_count: 0,
            current_part_count: 0,
            part_size_mb,
            used_parts: Vec::new(),
            seg_part_data: Vec::new(),
        }
    }
    // 初始化存储临时目录
    fn init_storage_dir(&self) -> std::io::Result<()> {
        let dir_path = Path::new(STORAGE_TEMP_DIR);
        // 清理旧目录（忽略不存在的错误）
        if dir_path.exists() {
            fs::remove_dir_all(dir_path)?;
            println!("✅ 已清理旧存储目录: {}", STORAGE_TEMP_DIR);
        }
        // 创建新目录
        fs::create_dir(dir_path)?;
        println!("✅ 已创建新存储目录: {}", STORAGE_TEMP_DIR);
        Ok(())
    }
    // 单个分区的字节数（避免重复计算）
    fn part_size_bytes(&self) -> u64 {
        self.part_size_mb as u64 * 1024 * 1024
    }

    // 当前已占用的总字节数
    fn total_used_bytes(&self) -> u64 {
        self.current_part_count as u64 * self.part_size_bytes()
    }
    pub fn init(&mut self) {
        if self.rate > 0 {
            let size = self.part_size_bytes();
            self.seg_part_data = vec![1; size as usize];
        };
        let _ = self.init_storage_dir();
    }
    fn push(&mut self) -> std::io::Result<()> {
        // let part_id = used_parts.size() as u32;
        let part_id = self.used_parts.len();
        let file_path = format!("{}/{}.tmp", STORAGE_TEMP_DIR, part_id);
        // allocate_storage_file_lazy(&file_path[..], self.part_size_mb as u64 *1024 as u64)?;

        let mut file = File::create(&file_path)?;
        file.write_all(&self.seg_part_data.to_vec())?;
        file.flush()?; // 确保数据写入磁盘

        println!("{}", file_path);
        self.used_parts.push(part_id);
        self.current_part_count += 1;
        println!("{}", file_path);
        Ok(())
    }
    fn pop(&mut self) -> std::io::Result<()> {
        let part_id = self.used_parts.pop().unwrap();
        let file_path = format!("{}/{}.tmp", STORAGE_TEMP_DIR, part_id);
        fs::remove_file(&file_path)?;
        self.current_part_count -= 1;
        Ok(())
    }
    /// 获取存储信息（返回：总字节数、已用字节数、可用字节数）
    fn get_storage_info(sys: &System) -> std::io::Result<(u64, u64, u64)> {
        #[cfg(unix)]
        {
            let mount = sys.mount_at("/")?;
            let total = mount.total.as_u64();
            let avail = mount.avail.as_u64();
            let used = saturating_sub_bytes(mount.total, mount.avail).as_u64();
            return Ok((total, used, avail));
        }

        #[cfg(windows)]
        {
            let mounts = sys.mounts()?;
            let first_mount = mounts.first().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "未找到任何挂载点")
            })?;
            let total = first_mount.total.as_u64();
            let avail = first_mount.avail.as_u64();
            let used = saturating_sub_bytes(first_mount.total, first_mount.avail).as_u64();
            return Ok((total, used, avail));
        }

        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("unsupport system"),
        ));
    }

    pub fn update(&mut self, sys: &System) -> std::io::Result<()> {
        let (total, used, avail) = Self::get_storage_info(sys)?;
        let target_total_bytes = total * self.rate as u64 / 100;

        // 计算其他程序已占用的字节数
        let other_used = used - self.total_used_bytes();
        let target_part_count = if target_total_bytes <= other_used {
            0
        } else {
            (target_total_bytes - other_used) / self.part_size_bytes()
        };

        // 平滑调整：避免频繁波动
        if ((target_part_count as f64) / (self.target_part_count as f64) > 0.95)
            && ((target_part_count as f64) / (self.target_part_count as f64) < 1.05)
        {
            return Ok(());
        } else {
            self.target_part_count = target_part_count;
        }

        self.current_part_count = self.used_parts.len() as u64;
        let target_part_count = self.target_part_count;

        // 打印当前状态（仅当数量变化时）
        if self.current_part_count != target_part_count {
            println!(
                "\n📁 存储占用：当前 {} 个分区（{} MB），目标 {} 个分区（{} MB）",
                self.current_part_count,
                self.current_part_count * self.part_size_mb as u64,
                target_part_count,
                target_part_count * self.part_size_mb as u64
            );
        }

        // 需新增分区
        while self.current_part_count < self.target_part_count {
            let _ = self.push();
            println!("{} {} ", self.current_part_count, self.target_part_count);
        }

        // 需释放分区
        while self.current_part_count > self.target_part_count {
            let _ = self.pop();
        }

        return Ok(());
    }
}

// // 补充：程序退出时清理存储目录（避免残留文件）
impl Drop for FileOccupyConfig {
    fn drop(&mut self) {
        if self.rate > 0 && std::path::Path::new(STORAGE_TEMP_DIR).exists() {
            let _ = std::fs::remove_dir_all(STORAGE_TEMP_DIR);
            println!("\n🗑️  程序退出，已清理存储临时目录");
        }
    }
}

// /// 用 fallocate 预分配文件空间（替代 write_all 写入全 1 数据）
// /// 跨平台存储预分配函数（Unix 用 fallocate，Windows 用 SetEndOfFile，其他系统用写零填充）
fn allocate_storage_file_lazy(file_path: &str, size_bytes: u64) -> io::Result<()> {
    // // 确保目录存在（避免文件路径中的目录未创建）
    // if let Some(dir) = Path::new(file_path).parent() {
    //     if !dir.exists() {
    //         std::fs::create_dir_all(dir)?;
    //     }
    // }

    // let file = File::create(file_path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path)
        .unwrap();
    // 跨平台预分配逻辑
    #[cfg(unix)]
    {
        if fallocate(&file, FallocateMode::PunchHole, true, 0, size_bytes).is_ok() {
            return Ok(());
        }
    }

    #[cfg(windows)]
    {
        if fallocate(&file, FallocateMode::PunchHole, true, 0, size_bytes).is_ok() {
            return Ok(());
        }
    }

    // #[cfg(not(any(unix, windows)))]
    // {
    // 其他系统（如 macOS 其实属于 unix，这里兼容极端情况）：用写零填充实现
    const BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16MB 缓冲区
    let buffer = vec![0; BUFFER_SIZE];
    let mut remaining = size_bytes;

    let mut file = file;
    while remaining > 0 {
        let write_size = std::cmp::min(remaining as usize, BUFFER_SIZE);
        file.write_all(&buffer[..write_size])?;
        remaining -= write_size as u64;
    }
    file.flush()?;
    // }

    Ok(())
}
fn allocate_storage_file_real(file_path: &str, size_bytes: u64) -> io::Result<()> {
    // 确保目录存在（避免文件路径中的目录未创建）
    if let Some(dir) = Path::new(file_path).parent() {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
    }
    // let file = File::create(file_path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path)
        .unwrap();

    const BUFFER_SIZE: usize = 1024 * 1024; // 1MB 缓冲区
    let buffer = vec![0; BUFFER_SIZE];
    let mut remaining = size_bytes;

    let mut file = file;
    while remaining > 0 {
        let write_size = std::cmp::min(remaining as usize, BUFFER_SIZE);
        file.write_all(&buffer[..write_size])?;
        remaining -= write_size as u64;
    }
    file.flush()?;

    Ok(())
}
