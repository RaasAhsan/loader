use std::{arch::asm, fs::File, os::fd::AsRawFd};

use elf::elf64::{self, header::ProgramHeader};
use memmap2::MmapOptions;

fn main() {
    // let args = env::args().collect::<Vec<_>>();
    // if args.len() < 2 {
    //     panic!("Expected path to program");
    // }

    // let path = &args[1];

    let file = File::open("./examples/static-no-pie/a.out").expect("failed to open executable");

    let mmap =
        unsafe { MmapOptions::new().map(&file) }.expect("failed to map executable into memory");

    let headers = elf64::header::Headers::parse(&mmap).expect("failed to parse elf headers");

    if headers.header.e_type != 0x02 {
        panic!("only position-dependent executables are supported currently");
    }

    let mut loadable_hdrs = vec![];
    for hdr in headers.program_headers.iter() {
        match hdr.p_type {
            0x01 => {
                loadable_hdrs.push(hdr);
            }
            0x02 => {
                panic!("dynamic linking not supported currently");
            }
            _ => {}
        }
    }

    if loadable_hdrs.is_empty() {
        panic!("no loadable segments found");
    }

    let (base, size) = get_initial_memory_map(&loadable_hdrs);

    // Allocate a continguous region of virtual memory so that virtual addresses are intact.
    initialize_mapping(base, size);
    load_segments(&file, &loadable_hdrs);

    let entrypoint = headers.header.e_entry;

    unsafe { jump(entrypoint) };

    panic!("unexpected return");
}

unsafe fn jump(entrypoint: u64) {
    asm!(
        // "sub sp, sp, #32",
        // "mov x0, #0",
        // "str x0, [sp]",
        // "str x0, [sp, #8]",
        // "str x0, [sp, #16]",
        "mov x0, {}",
        "br x0",
        in(reg) entrypoint,
        // in(reg) sp,
        // in(reg) argc,
        // in(reg) argv,
        options(noreturn, nomem, nostack)
        // options(nomem, nostack)
    );
}

fn get_initial_memory_map(hdrs: &[&ProgramHeader]) -> (u64, usize) {
    let lowest = hdrs
        .iter()
        .min_by(|x, y| {
            let xv = x.p_vaddr;
            let yv = y.p_vaddr;
            xv.cmp(&yv)
        })
        .map(|h| h.p_vaddr)
        .unwrap();
    let highest = hdrs
        .iter()
        .max_by(|x, y| {
            let xv = x.p_vaddr;
            let yv = y.p_vaddr;
            xv.cmp(&yv)
        })
        .map(|h| h.p_vaddr + h.p_memsz)
        .unwrap();
    (lowest, (highest - lowest) as usize)
}

fn initialize_mapping(base: u64, length: usize) {
    let mapped_addr = unsafe {
        libc::mmap(
            base as *mut libc::c_void,
            length as libc::size_t,
            libc::PROT_WRITE | libc::PROT_READ, // TODO: this is a hack because memsize/filesize differences
            libc::MAP_FIXED_NOREPLACE | libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if mapped_addr == libc::MAP_FAILED {
        panic!("failed to map segment {}", unsafe {
            *libc::__errno_location()
        });
    }
}

fn initialize_stack() -> u64 {
    let size: usize = 2 * 1024 * 1024;
    let mapped_addr = unsafe {
        libc::mmap(
            std::ptr::null_mut::<libc::c_void>(),
            size as libc::size_t,
            libc::PROT_WRITE | libc::PROT_READ,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if mapped_addr == libc::MAP_FAILED {
        panic!("failed to map segment {}", unsafe {
            *libc::__errno_location()
        });
    }

    mapped_addr as u64 + size as u64
}

fn load_segments(file: &File, hdrs: &[&ProgramHeader]) {
    for hdr in hdrs.iter() {
        let addr = hdr.p_vaddr;
        let aligned_addr = addr & !0xfff;
        let file_size = hdr.p_filesz;
        let mem_size = hdr.p_memsz;

        let length = file_size + (addr - aligned_addr);
        let h_flags = hdr.p_flags;
        let h_offset = (hdr.p_offset & !0xfff) as i64;

        println!("Attempting to map {:08x}-{:08x}", addr, addr + hdr.p_memsz);

        let mut prot: libc::c_int = 0;
        if h_flags & 0b001 != 0 {
            prot |= libc::PROT_EXEC;
        }
        if h_flags & 0b010 != 0 {
            prot |= libc::PROT_WRITE;
        }
        if h_flags & 0b100 != 0 {
            prot |= libc::PROT_READ;
        }

        let mapped_addr = unsafe {
            libc::mmap(
                aligned_addr as *mut libc::c_void,
                length as libc::size_t,
                libc::PROT_EXEC | libc::PROT_WRITE | libc::PROT_READ, // TODO: this is a hack because of permissioning
                libc::MAP_FIXED | libc::MAP_PRIVATE,
                file.as_raw_fd(),
                h_offset,
            )
        };

        if mapped_addr == libc::MAP_FAILED {
            panic!("failed to map segment {}", unsafe {
                *libc::__errno_location()
            });
        }

        if file_size < mem_size {}

        println!(
            "Mapped {:08x}:{:08x} to {:08x}-{:08x} with prot {:?}",
            h_offset,
            length,
            mapped_addr as u64,
            (mapped_addr as u64) + length,
            h_flags
        );
    }
}
