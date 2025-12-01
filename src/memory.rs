use systemstat::{saturating_sub_bytes, Platform, System};

#[derive(Debug, Clone)]
pub struct MemOccupyConfig {
    pub rate: u16,               // 目标占用比例（0-100）
    pub target_part_count: u64,  // 目标分区数量
    pub current_part_count: u64, // 当前分区数量
    pub part_size_mb: u16,       // 单个分区大小（MB）
    pub used_parts: Vec<u32>,    // 已使用的分区ID（用于存储文件命名/内存块追踪）
    pub seg_part_data: Vec<u8>,
    pub buckets: Vec<Vec<u8>>,
}

impl MemOccupyConfig {
    // 构造函数：初始化占用配置
    pub fn new(rate: u16, part_size_mb: u16) -> Self {
        Self {
            rate,
            target_part_count: 0,
            current_part_count: 0,
            part_size_mb,
            used_parts: Vec::new(),
            seg_part_data: Vec::new(),
            buckets: Vec::new(),
        }
    }

    // 单个分区的字节数（避免重复计算）
    fn part_size_bytes(&self) -> u64 {
        self.part_size_mb as u64 * 1024 * 1024
    }

    // 当前已占用的总字节数
    fn total_used_bytes(&self) -> u64 {
        self.current_part_count as u64 * self.part_size_bytes()
    }
    fn push(&mut self) {
        self.buckets.push(self.seg_part_data.to_vec());
        self.current_part_count += 1;
    }
    fn pop(&mut self) {
        self.buckets.pop();
        self.current_part_count -= 1;
    }
    pub fn init(&mut self) {
        if self.rate > 0 {
            let size = self.part_size_bytes();
            self.seg_part_data = vec![1; size as usize];
        };
        // self.init_storage_dir();
    }
    pub fn update(&mut self, sys: &System) -> std::io::Result<()> {
        if self.current_part_count != self.target_part_count {
            print!(
                "\r🧠 内存占用：当前 {} 个分区（{} MB），目标 {} 个分区（{} MB）",
                self.current_part_count,
                self.current_part_count * self.part_size_mb as u64,
                self.target_part_count,
                self.target_part_count * self.part_size_mb as u64
            );
        }
        // 打印当前状态
        let mem = match sys.memory() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("❌ 获取内存信息失败：{}", e);
                return Ok(());
            }
        };
        let total_bytes = mem.total.as_u64();
        let used_bytes = saturating_sub_bytes(mem.total, mem.free).as_u64();
        let target_total_bytes = total_bytes * self.rate as u64 / 100;

        // 计算其他程序已占用的字节数
        let other_used = used_bytes - self.total_used_bytes();
        let target_part_count = if target_total_bytes <= other_used {
            0
        } else {
            (target_total_bytes - other_used) / self.part_size_bytes()
        };
        // println!(
        //     "  total_bytes:{}. used_bytes：{} |target_total_bytes：{}, other_used：{} {}",
        //     total_bytes, used_bytes, target_total_bytes, other_used, self.current_part_count
        // );
        // 平滑调整
        if ((target_part_count as f64) / (self.target_part_count as f64) > 0.95)
            && ((target_part_count as f64) / (self.target_part_count as f64) < 1.05)
        {
            return Ok(());
        } else {
            self.target_part_count = target_part_count;
        }

        self.current_part_count = self.buckets.len() as u64;

        // 需新增内存块
        while self.current_part_count < self.target_part_count {
            self.push();
        }

        // 需释放内存块
        while self.current_part_count > self.target_part_count {
            self.pop();
        }
        return Ok(());
    }
}

// // 补充：程序退出时清理存储目录（避免残留文件）
impl Drop for MemOccupyConfig {
    fn drop(&mut self) {
        if self.rate > 0 {
            self.buckets.clear();
            println!("\n🗑️  程序退出，已清理存储临时目录");
        }
    }
}

// fn preallocate_memory_mb(size_mb: usize) -> Vec<u8> {
//     let size_bytes = size_mb * 1024 * 1024;
//     let mut vec = Vec::with_capacity(size_bytes);
//     // 可选：写入一个字节触发物理内存分配（否则可能仅占虚拟内存）
//     vec.push(0);
//     vec
// }
