extern crate systemstat;

use {
    std::{
        thread,
        convert::TryInto,
        time::Duration,
        fs,
        fs::File,
        io::prelude::*
    },
    systemstat::{System, Platform, saturating_sub_bytes},
    rand::Rng,
    clap::{load_yaml, value_t, value_t_or_exit, App, Arg, ArgMatches}
    };

extern crate rand;

struct Occupy{
    rate:u16,
    target:u64,
    up:bool,
    once:bool,
    index: Vec<u32>
}


#[cfg(target_os = "linux")]
pub static OS_TYPE:u8 = 0;

#[cfg(target_os = "windows")]
pub static OS_TYPE:u8 = 1;

#[cfg(target_os = "macos")]
pub static OS_TYPE:u8 = 2;

struct SysOS{
    macos:bool,
    linux:bool,
    windows:bool
}
const MEMORY_PART_SIZE:usize=5;//MB
const STORAGE_PART_SIZE:usize=50;//MB
const STORAGE_PATH:&str="./storage_bucket_dsd1sakmf";
fn main()  -> std::io::Result<()>{
    let sys = System::new();
    let args_def = load_yaml!("occupier_r.yaml");

    let args = App::from_yaml(args_def).get_matches();
    let cpu_rate:u16 = args.value_of("cpu").unwrap_or("0").parse().unwrap_or(0);
    let memory_rate:u16 = args.value_of("memory").unwrap_or("0").parse().unwrap_or(0);
    let storage_rate:u16 = args.value_of("storage").unwrap_or("0").parse().unwrap_or(0);
    let granularity:usize = args.value_of("granularity").unwrap_or("0").parse().unwrap_or(50);
    let flush_delay_time:f32 = args.value_of("delay").unwrap_or("0").parse().unwrap_or(0.2);
    let display_system:bool = args.value_of("display_sys").unwrap_or("false").parse().unwrap_or(false);
    if display_system{
         systemstat_example();
    }
    //let memory_rate=50;
    let flush_delay_time=(flush_delay_time *1000.0) as u64;
    if cpu_rate+memory_rate+storage_rate==0
    {
        println!("all zero occupy,need no thing to do");
        return Ok(());
    }
    //let mut rng = rand::thread_rng();
    let mut counter:usize=0;

        let mut memory_occupy=Occupy{
            rate:memory_rate,
            target:0,
            up:false,
            once:true,
            index:Vec::new()
        };
        let mut memory_part:Vec<u8>=Vec::new();
        if memory_rate!=0{//less waste
            for i in 0..MEMORY_PART_SIZE*1024*1024
            {
                memory_part.push(1);
            }
        }
        let mut memory_bucket:Vec<Vec<u8>>=Vec::new();

        let mut storage_occupy=Occupy{
            rate:storage_rate,
            target:0,
            up:false,
            once:true,
            index:Vec::new()
        };
        
        let mut storage_part:Vec<u8>=Vec::new();
        if storage_rate!=0{ //less waste
            for i in 0..STORAGE_PART_SIZE*1024*1024
            {
                storage_part.push(1);
            }
        }
        match fs::remove_dir_all(STORAGE_PATH){
            Ok(_) => println!("\ndeleted seccessfully"),
            Err(x)=>println!("\ntry to delete rubbuish before:{}",x)
        };
        fs::create_dir(STORAGE_PATH)?;

    let mut is_free:bool=false;

    loop{        
        is_free=true;
        if storage_rate!=0{
            if storage_occupy.index.len()<storage_occupy.target as usize{
                storage_occupy.index.push(counter as u32);
                let mut buf = File::create(format!("{}/{}{}",STORAGE_PATH,counter,".tmp"))?;
                buf.write_all(&storage_part)?;
                buf.flush()?;
                is_free=false;
            }else if storage_occupy.index.len()>storage_occupy.target as usize{
                //let a=storage_occupy.bucket.pop();
                let name=storage_occupy.index.pop().unwrap();
                println!("free storage");
                fs::remove_file(format!("{}/{}{}",STORAGE_PATH,name,".tmp"))?;
                thread::sleep(Duration::from_millis(100)); 
                is_free=false;
            }
            else{
    
            }
            if (counter/STORAGE_PART_SIZE)%granularity==0 || is_free==true {
                storage_get_target(&sys,&mut storage_occupy);
                thread::sleep(Duration::from_millis(flush_delay_time)); 
            }
        }
        if memory_rate!=0{
            if memory_occupy.index.len()<memory_occupy.target as usize{
                memory_bucket.push(memory_part.clone());
                memory_occupy.index.push(0);
                //memory_occupy.bucket.push(counter.try_into().unwrap());
                is_free=false;
            }else if memory_occupy.index.len()>memory_occupy.target as usize{
                memory_bucket.pop();
                memory_occupy.index.pop();
                //println!("pop");
                is_free=false;
            }
            else{
                //is_free-=1;
            }
            if counter%(granularity/MEMORY_PART_SIZE)==0 || is_free==true {
                memory_get_target(&sys,&mut memory_occupy);
                thread::sleep(Duration::from_millis(flush_delay_time)); 
            }
    
        }

        counter+=1;
        if is_free{
            println!("sleep");
            thread::sleep(Duration::from_millis(flush_delay_time*5)); 
        }
    }
}

fn memory_get_target(sys:&systemstat::platform::PlatformImpl,resource:&mut Occupy) -> u64 {
    match sys.memory() {
        Ok(mem) => {
            let mem_used=saturating_sub_bytes(mem.total, mem.free);
            println!("\nMemory: {} used / {} total, {}%,Occupied:{}MB",
                             mem_used, mem.total, 
                             100-100*mem.free.as_u64()/mem.total.as_u64(),resource.index.len()*1);//64/8/1024/1024
            let  mem_target=mem.total.as_u64()*(resource.rate as u64)/100;
            let mut part_num_target:u64=0;
            //mem_target-mem_used.as_u64()+resource.index.len()as u64*1024*1024*MEMORY_PART_SIZE as u64
            let other_used=mem_used.as_u64()-resource.index.len()as u64*1024*1024*MEMORY_PART_SIZE as u64;
            if mem_target<other_used{
                part_num_target=0;
            }
            else{
                //bytes =>num
                part_num_target= (mem_target-other_used)/(1024*1024*MEMORY_PART_SIZE as u64);//64/8
                //println!("{}   {}",other_used,part_num_target); 
            }
            //ignore tiny to improve performance
            if (part_num_target as f64/(resource.target as f64)<1.05) && (part_num_target as f64/resource.target as f64>0.95){
                resource.target=resource.target;
            }else{
                resource.target=part_num_target;
            }
            println!("now:{} target:{}",resource.index.len(),resource.target); 
            //return (mem_used,mem.free,mem.total);
            return resource.target;
        },
        Err(x) => {println!("\nMemory: error: {}", x);

            return 0;
        }
    }
}
fn storage_get_target(sys:&systemstat::platform::PlatformImpl,resource:&mut Occupy) -> u64 {
    let (storage_used,storage_avail,storage_total)=storage_getmsg(&sys);
    println!("\nStorage: Used:{} Avail:{}Total{}, {}%, Occupied:{}MB",
            storage_used,storage_avail,storage_total,
            100*storage_used.as_u64()/storage_total.as_u64(),resource.index.len()*STORAGE_PART_SIZE);     
    let storage_target=storage_total.as_u64()*(resource.rate as u64)/100;
    let other_used=storage_used.as_u64()-resource.index.len()as u64*STORAGE_PART_SIZE as u64*1024*1024;
    let mut part_num_target:u64=0;
    //println!("{}   {}",other_used,part_num_target); 
    if storage_target<other_used{
        //pass
        part_num_target=0;
    }
    else{
        //bytes =>num
        part_num_target= (storage_target-other_used)/(STORAGE_PART_SIZE as u64*1024*1024);
    }
    //ignore tiny to improve performance
    if (part_num_target as f64/(resource.target as f64)<1.05) && (part_num_target as f64/resource.target as f64>0.95){
        resource.target=resource.target;
    }else{
        resource.target=part_num_target;
    }
    return resource.target;
}
fn storage_getmsg(sys:&systemstat::platform::PlatformImpl) -> (systemstat::ByteSize, systemstat::ByteSize,systemstat::ByteSize) {
    //let sys = System::new();
    match OS_TYPE{
        0=>{//linux
            match sys.mount_at("/") {
                Ok(mount) => {
                    let storage_used=saturating_sub_bytes(mount.total,mount.avail);
                    return (storage_used,mount.avail,mount.total);
                    
                }
                
                Err(x) => println!("\nMount at /: error: {}", x)
            }
        }
        1 => {//windows
                match sys.mounts() {
                Ok(mounts) => {
                let storage_used=saturating_sub_bytes(mounts[0].total,mounts[0].avail);
                return (storage_used,mounts[0].avail,mounts[0].total);
                    }
                    Err(x) => println!("\nMounts: error: {}", x)
                }
            }

        _=>{
            return (systemstat::ByteSize(1),systemstat::ByteSize(1),systemstat::ByteSize(1));
        }
    }
    return (systemstat::ByteSize(1),systemstat::ByteSize(1),systemstat::ByteSize(1));
}
fn systemstat_example(){
    
    let sys = System::new();
    
    match sys.mounts() {
        Ok(mounts) => {
            println!("\nMounts:");
            for mount in mounts.iter() {
                println!("{} ---{}---> {} (available {} of {})",
                         mount.fs_mounted_from, mount.fs_type, mount.fs_mounted_on, mount.avail, mount.total);
            }
        }
        Err(x) => println!("\nMounts: error: {}", x)
    }

    match sys.mount_at("/") {
        Ok(mount) => {
            println!("\nMount at /:");
            println!("{} ---{}---> {} (available {} of {})",
                     mount.fs_mounted_from, mount.fs_type, mount.fs_mounted_on, mount.avail, mount.total);
        }
        Err(x) => println!("\nMount at /: error: {}", x)
    }

    match sys.block_device_statistics() {
        Ok(stats) => {
            for blkstats in stats.values() {
                println!("{}: {:?}", blkstats.name, blkstats);
            }
        }
        Err(x) => println!("\nBlock statistics error: {}", x.to_string())
    }

/* 
    match sys.networks() {
        Ok(netifs) => {
            println!("\nNetworks:");
            for netif in netifs.values() {
                println!("{} ({:?})", netif.name, netif.addrs);
            }
        }
        Err(x) => println!("\nNetworks: error: {}", x)
    }

    match sys.networks() {
        Ok(netifs) => {
            println!("\nNetwork interface statistics:");
            for netif in netifs.values() {
                println!("{} statistics: ({:?})", netif.name, sys.network_stats(&netif.name));
            }
        }
        Err(x) => println!("\nNetworks: error: {}", x)
    }
 */
    match sys.battery_life() {
        Ok(battery) =>
            print!("\nBattery: {}%, {}h{}m remaining",
                   battery.remaining_capacity*100.0,
                   battery.remaining_time.as_secs() / 3600,
                   battery.remaining_time.as_secs() % 60),
        Err(x) => print!("\nBattery: error: {}", x)
    }
    
    match sys.on_ac_power() {
        Ok(power) => println!(", AC power: {}", power),
        Err(x) => println!(", AC power: error: {}", x)
    }

    match sys.memory() {
        Ok(mem) => println!("\nMemory: {} used / {} ({} bytes) total ({:?})", saturating_sub_bytes(mem.total, mem.free), mem.total, mem.total.as_u64(), mem.platform_memory),
        Err(x) => println!("\nMemory: error: {}", x)
    }

    match sys.load_average() {
        Ok(loadavg) => println!("\nLoad average: {} {} {}", loadavg.one, loadavg.five, loadavg.fifteen),
        Err(x) => println!("\nLoad average: error: {}", x)
    }

    match sys.uptime() {
        Ok(uptime) => println!("\nUptime: {:?}", uptime),
        Err(x) => println!("\nUptime: error: {}", x)
    }

    match sys.boot_time() {
        Ok(boot_time) => println!("\nBoot time: {}", boot_time),
        Err(x) => println!("\nBoot time: error: {}", x)
    }

    match sys.cpu_load_aggregate() {
        Ok(cpu)=> {
            println!("\nMeasuring CPU load...");
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            println!("CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                cpu.user * 100.0, cpu.nice * 100.0, cpu.system * 100.0, cpu.interrupt * 100.0, cpu.idle * 100.0);
        },
        Err(x) => println!("\nCPU load: error: {}", x)
    }

    match sys.cpu_temp() {
        Ok(cpu_temp) => println!("\nCPU temp: {}", cpu_temp),
        Err(x) => println!("\nCPU temp: {}", x)
    }

    match sys.socket_stats() {
        Ok(stats) => println!("\nSystem socket statistics: {:?}", stats),
        Err(x) => println!("\nSystem socket statistics: error: {}", x.to_string())
    }
}