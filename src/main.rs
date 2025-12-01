use occupier_r::param;

extern crate clap;
extern crate rand;
extern crate systemstat;

extern crate vmm_sys_util;
// const STORAGE_TEMP_DIR: &str = "./storage_bucket_occupier_r_dsd1sakmf";

use std::{io::prelude::*, thread, time::Duration};
use systemstat::{saturating_sub_bytes, Platform, System};
// use clap::{App, Arg , ArgMatches};

fn main() -> std::io::Result<()> {
    let sys = System::new();
    let args = param::parse_args();
    let _os_type = param::OsType::current();

    let mut config = param::parse_config(&args);

    if config.display_system {
        systemstat_example(&sys);
    }

    if config.cpu_rate == 0 && config.memory_config.rate == 0 && config.storage_config.rate == 0 {
        eprintln!("⚠️  all reosurce setting is zero, exit!");
        return Ok(());
    }

    if config.storage_config.rate > 0 {
        config.storage_config.init();
    }
    if config.memory_config.rate > 0 {
        config.memory_config.init();
    }

    let mut counter: i32 = 0;
    let flush_delay = Duration::from_millis((config.flush_delay * 1000.0) as u64);
    loop {
        let is_idle = true;

        let _ = config.storage_config.update(&sys);
        let _ = config.memory_config.update(&sys);

        if is_idle {
            thread::sleep(flush_delay * 5);
            println!("ℹ️  all resource is occupiered");
        } else {
            thread::sleep(flush_delay);
        }

        counter = counter.wrapping_add(1); // 避免溢出
    }
    Ok(())
}

/// 系统信息展示（优化输出格式，增加可读性）
fn systemstat_example(sys: &System) {
    println!("\n==================================================");
    println!(
        "📊 系统信息概览（操作系统：{:?}）",
        param::OsType::current()
    );
    println!("==================================================");

    // 挂载点信息
    match sys.mounts() {
        Ok(mounts) => {
            println!("\n📁 挂载点列表：");
            for (i, mount) in mounts.iter().take(5).enumerate() {
                // 只显示前 5 个，避免输出过长
                println!(
                    "  {}. 来源：{} | 类型：{} | 挂载点：{} | 可用：{} / 总计：{}",
                    i + 1,
                    mount.fs_mounted_from,
                    mount.fs_type,
                    mount.fs_mounted_on,
                    mount.avail,
                    mount.total
                );
            }
            if mounts.len() > 5 {
                println!("  ... 共 {} 个挂载点（省略剩余）", mounts.len());
            }
        }
        Err(x) => eprintln!("❌ 获取挂载点信息失败：{}", x),
    }

    // 根目录存储信息
    match sys.mount_at("/") {
        Ok(mount) => {
            println!("\n💾 根目录存储：");
            let used = saturating_sub_bytes(mount.total, mount.avail);
            println!(
                "  已用：{} | 可用：{} | 总计：{} | 占用率：{:.1}%",
                used,
                mount.avail,
                mount.total,
                100 * used.as_u64() / mount.total.as_u64()
            );
        }
        Err(x) => eprintln!("❌ 获取根目录存储信息失败：{}", x),
    }

    // 内存信息
    match sys.memory() {
        Ok(mem) => {
            println!("\n🧠 内存信息：");
            let used = saturating_sub_bytes(mem.total, mem.free);
            println!(
                "  已用：{} | 空闲：{} | 总计：{} | 占用率：{:.1}%",
                used,
                mem.free,
                mem.total,
                100 * (mem.total.as_u64() - mem.free.as_u64()) / mem.total.as_u64()
            );
        }
        Err(x) => eprintln!("❌ 获取内存信息失败：{}", x),
    }

    // CPU 负载
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            println!("\n⚡ CPU 负载（1 秒测量）：");
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            println!(
                "  用户态：{:.1}% | 系统态：{:.1}% | 空闲：{:.1}%",
                cpu.user * 100.0,
                cpu.system * 100.0,
                cpu.idle * 100.0
            );
        }
        Err(x) => eprintln!("❌ 获取 CPU 负载失败：{}", x),
    }

    // 系统运行时间
    match sys.uptime() {
        Ok(uptime) => println!("\n⏱️  系统运行时间：{:?}", uptime),
        Err(x) => eprintln!("❌ 获取运行时间失败：{}", x),
    }

    println!("\n==================================================");
}

#[cfg(test)]
mod tests {
    use super::*; // 引入当前包的所有公共项，包括模块和函数
    #[test]
    fn test_module1() {
        // module1::function_in_module1();
    }
    #[test]
    fn test_module2() {
        // module2::function_in_module2();
    }
}
