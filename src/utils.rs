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
    let mid_pos = chars_per_line / 2; // 每行中间位置，在该位置前多加一个空格

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
            // 中间位置，多输出一个空格
            if i == mid_pos {
                print!(" ");
            }

            match unit {
                2 => {
                    if i % 2 == 0 {
                        print!("{:02x}{:02x} ", 
                        bytes[start_index + i + 1], bytes[start_index + i]);
                    }
                },
                4 => {
                    if i % 4 == 0 {
                        print!("{:02x}{:02x}{:02x}{:02x} ", 
                        bytes[start_index + i + 3], bytes[start_index + i + 2], 
                        bytes[start_index + i + 1], bytes[start_index + i]);
                    }
                },
                8 => {
                    if i % 8 == 0 {
                        print!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x} ", 
                        bytes[start_index + i + 7], bytes[start_index + i + 6], 
                        bytes[start_index + i + 5], bytes[start_index + i + 4], 
                        bytes[start_index + i + 3], bytes[start_index + i + 2], 
                        bytes[start_index + i + 1], bytes[start_index + i]);
                    }
                },
                _ => { // 按照unit为1处理
                    print!("{:02x} ", bytes[start_index + i]);
                },
            }
        }

        // 每行结束输出换行符
        println!();
        remain_bytes_count -= chars_num;
    }
}

pub fn read(addr: u64, size: usize) -> anyhow::Result<()> {
    anyhow::ensure!(size > 0);

    let f = OpenOptions::new().read(true).write(false).open("/dev/mem")?;
    let mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map(&f)?
    };

    dump(addr, &mmap[..], 1, size);

    Ok(())
}

pub fn mem_dump(addr: u64, unit: usize, count: usize) -> anyhow::Result<()> {
    let size = unit * count;
    anyhow::ensure!(size > 0);

    let f = OpenOptions::new().read(true).write(false).open("/dev/mem")?;
    let mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map(&f)?
    };

    dump(addr, &mmap[..], unit as u8, count);

    Ok(())
}

pub fn mem_set(addr: u64, size: usize, value: u64) -> anyhow::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open("/dev/mem")?;
    let mut mmap = unsafe {
        MmapOptions::new()
            .offset(addr)
            .len(size)
            .map_mut(&f)?
    };

    match size {
        1 => {
            mmap[0] =  (value & 0xff) as u8;
        },
        2 => {
            mmap[0] =  (value & 0x00ff) as u8;
            mmap[1] = ((value & 0xff00) >> 8) as u8;
        },
        4 => {
            mmap[0] =  (value & 0x000000ff) as u8;
            mmap[1] = ((value & 0x0000ff00) >> 8) as u8;
            mmap[2] = ((value & 0x00ff0000) >> 16) as u8;
            mmap[3] = ((value & 0xff000000) >> 24) as u8;
        },
        8 => {
            mmap[0] =  (value & 0x00000000000000ff) as u8;
            mmap[1] = ((value & 0x000000000000ff00) >> 8) as u8;
            mmap[2] = ((value & 0x0000000000ff0000) >> 16) as u8;
            mmap[3] = ((value & 0x00000000ff000000) >> 24) as u8;
            mmap[4] = ((value & 0x000000ff00000000) >> 32) as u8;
            mmap[5] = ((value & 0x0000ff0000000000) >> 40) as u8;
            mmap[6] = ((value & 0x00ff000000000000) >> 48) as u8;
            mmap[7] = ((value & 0xff00000000000000) >> 56) as u8;
        },
        _ => {
            return Err(anyhow::format_err!("invalid size \'{size}\', please input --help to get usage"));
        },
    }

    Ok(())
}