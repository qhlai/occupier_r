use std::hint::black_box;
use std::time::Duration;
use systemstat::{Platform, System};
use tokio;

#[derive(Debug, Clone)]
pub struct CPUOccupyConfig {
    pub rate: u16,      // 目标占用比例（0-100）
    pub cpu_num: u16,   // cpu数量
    pub loop_time: u16, // 循环时间
}

impl CPUOccupyConfig {
    // 构造函数：初始化占用配置
    pub fn new(rate: u16) -> Self {
        Self {
            rate,
            cpu_num: 0,
            loop_time: 0,
        }
    }
    pub async fn update(&self, sys: &System) -> std::io::Result<()> {
        let mut tasks = Vec::new();
        let target_cpu_usage_ratio: f64 = self.rate.into();
        let num_cpu_cores_to_use: usize = self.cpu_num.into();
        for i in 0..num_cpu_cores_to_use {
            // 为每个核心生成一个任务
            let task = tokio::spawn(busy_wait_task(target_cpu_usage_ratio, i, 10));
            tasks.push(task);
        }
        match tokio::time::timeout(Duration::from_secs(60), futures::future::join_all(tasks)).await
        {
            Ok(_) => println!("all done"),
            Err(_) => println!("\nstoped。"),
        }
        Ok(())
    }
}

async fn busy_wait_task(target_ratio: f64, cpu_id: usize, run_time: u64) {
    if target_ratio <= 0.0 {
        // 如果目标比例为0，直接永久休眠
        return;
    }

    // 设定一个周期，例如 10 毫秒
    const CYCLE_DURATION: Duration = Duration::from_millis(10);
    let cycle_nanos = CYCLE_DURATION.as_nanos() as f64;

    let run_time = Duration::from_millis(run_time);

    // 计算每个周期内需要"工作"的纳秒数
    let work_nanos = (cycle_nanos * target_ratio.min(1.0)) as u64;
    let work_duration = Duration::from_nanos(work_nanos);

    println!(
        "CPU {}: 开始以 {:.1}% 占用率运行...",
        cpu_id,
        target_ratio * 100.0
    );

    let work_loop_start = tokio::time::Instant::now();

    loop {
        let cycle_start = tokio::time::Instant::now();

        // --- 工作阶段 ---
        // 使用一个忙循环来消耗 CPU
        let work_start = tokio::time::Instant::now();
        while work_start.elapsed() < work_duration {
            // `black_box` 防止编译器优化掉这个空循环
            black_box(());
        }

        // --- 休眠阶段 ---
        // 计算周期中剩余的时间并休眠
        let elapsed = cycle_start.elapsed();
        if elapsed < CYCLE_DURATION {
            tokio::time::sleep(CYCLE_DURATION - elapsed).await;
        }
        if work_loop_start.elapsed() > run_time {
            return;
        }
    }
}
