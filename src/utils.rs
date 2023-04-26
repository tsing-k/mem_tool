use std::{fs::{OpenOptions}, cmp::min};
use memmap::{MmapOptions};
use crypto::{md5::Md5, digest::Digest};

const MD5_SEG_SIZE : usize = 1024 * 1024;  // 计算MD5时，防止内存占用太大，因此分段进行计算，此值为每段的大小

pub fn get_md5(addr: u64, size: usize) -> anyhow::Result<String> {
    let f = OpenOptions::new().read(true).write(false).open("/dev/mem")?;
    let mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map(&f)?
    };
    let mut md5 = Md5::new();
    let seg_cnt = (size + MD5_SEG_SIZE - 1) / MD5_SEG_SIZE;

    for i in 0..(seg_cnt - 1) {
        let start = i * MD5_SEG_SIZE;
        let end = start + MD5_SEG_SIZE;
        md5.input(&mmap[start..end]);
    }
    let start = (seg_cnt - 1) * MD5_SEG_SIZE;
    let end = size;
    md5.input(&mmap[start..end]);

    Ok(md5.result_str())
}

pub fn write(addr: u64, size: usize, value: Option<u8>) -> anyhow::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open("/dev/mem")?;
    let mut mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map_mut(&f)?
    };

    for item in mmap.iter_mut() {
        if let Some(v) = value {
            *item = v;
        } else {
            *item = rand::random();
        }
    }

    Ok(())
}

pub fn clear(addr: u64, size: usize, value: Option<u8>) -> anyhow::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open("/dev/mem")?;
    let mut mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map_mut(&f)?
    };

    mmap.fill(value.unwrap_or(0));

    Ok(())
}

fn dump(start_addr: u64, bytes: &[u8], unit: u8, unit_count: usize) {
    let byte_size = unit as usize * unit_count;
    let end_addr = start_addr + byte_size as u64;
    let end_addr_hex_str = format!("{:x}", end_addr);
    let mut addr_max_len = end_addr_hex_str.len();
    
    if addr_max_len % 2 == 1 { // 奇数，则+1
        addr_max_len += 1;
    }

    let chars_per_line = 16; // 每行最多显示字符数
    let line_count = (byte_size + chars_per_line - 1) / chars_per_line; // 行数，向上取整
    let mid_pos = chars_per_line / 2 - 1; // 每行中间位置

    let mut remain_bytes_count = byte_size;
    // 输出显示所有行数据
    for line in 0..line_count {
        // 计算该行数据的首地址
        let start_index = line * chars_per_line;
        let addr = start_addr + start_index as u64;
        let chars_num = min(chars_per_line, remain_bytes_count);

        // 每行开始输出地址
        print!("{addr:0>addr_max_len$x}: ");

        // 输出行内容
        for i in 0..chars_num {
            print!("{:02x} ", bytes[start_index + i]);
            if i == mid_pos {
                // 中间位置，多输出一个空格
                print!(" ");
            }
        }

        // 每行结束输出换行符
        println!();
        remain_bytes_count -= chars_num;
    }
}

pub fn read(addr: u64, number: usize) -> anyhow::Result<()> {
    let size = number;
    anyhow::ensure!(size > 0);

    let f = OpenOptions::new().read(true).write(false).open("/dev/mem")?;
    let mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map(&f)?
    };

    dump(addr, &mmap[..], 1, number);

    Ok(())
}