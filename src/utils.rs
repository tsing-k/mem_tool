use std::{fs::{OpenOptions}};
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

    let line_cnt = size as f64 / 16_f64;    // 每行最多显示16个字符
    let line_cnt = line_cnt.round() as usize;        // 行数
    for line in 0..(line_cnt - 1) {
        let start_index = line * 16;
        print!("0x{:016x}: ", addr + start_index as u64);
        for column in 0..16 {
            print!("{:02x} ", mmap[start_index + column]);
            if column == 7 {
                print!(" ");  // 中间两个空格
            }
        }
        println!();
    }
    // 最后一行
    let start_index = (line_cnt - 1) * 16;
    let remain_len = size % 16;
    if remain_len > 0 {
        print!("0x{:016x}: ", addr + start_index as u64);
    }
    for column in 0..remain_len {
        print!("{:02x} ", mmap[start_index + column]);
        if column == 7 {
            print!(" ");  // 中间两个空格
        }
        if column == (remain_len - 1) {
            println!(); // 最后一行
        }
    }

    Ok(())
}