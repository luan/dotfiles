use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};

use criterion::Criterion;
use mux::sidebar::bench_support::SidebarBenchFixture;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        }
        new_ptr
    }
}

#[derive(Clone, Copy, Debug)]
struct AllocationStats {
    allocations: u64,
    deallocations: u64,
    allocated_bytes: u64,
    deallocated_bytes: u64,
}

impl AllocationStats {
    fn net_allocations(self) -> i128 {
        i128::from(self.allocations) - i128::from(self.deallocations)
    }

    fn net_bytes(self) -> i128 {
        i128::from(self.allocated_bytes) - i128::from(self.deallocated_bytes)
    }
}

fn main() {
    print_allocation_profile();

    let mut criterion = Criterion::default().configure_from_args();
    sidebar_allocation_benchmarks(&mut criterion);
    criterion.final_summary();
}

fn sidebar_allocation_benchmarks(c: &mut Criterion) {
    let small = SidebarBenchFixture::synthetic(12, 2);
    let medium = SidebarBenchFixture::synthetic(30, 2);
    let large = SidebarBenchFixture::synthetic(60, 3);

    c.bench_function("sidebar_alloc/snapshot_decode/large", |b| {
        b.iter(|| large.bench_decode_snapshot())
    });
    c.bench_function("sidebar_alloc/item_build/large", |b| {
        b.iter(|| large.bench_build_items())
    });
    c.bench_function("sidebar_alloc/item_build/medium", |b| {
        b.iter(|| medium.bench_build_items())
    });
    c.bench_function("sidebar_alloc/render_frame/large_45x48", |b| {
        b.iter(|| large.bench_render_frame(45, 48))
    });
    let mut large_render = large.reusable_render_frame(45, 48);
    c.bench_function("sidebar_alloc/render_frame_reused/large_45x48", |b| {
        b.iter(|| large_render.draw())
    });
    c.bench_function("sidebar_alloc/filter/large_perf", |b| {
        b.iter(|| large.bench_filter("perf"))
    });
    let large_filter = large.reusable_filter();
    c.bench_function("sidebar_alloc/filter_existing/large_empty", |b| {
        b.iter(|| large_filter.filter(""))
    });
    c.bench_function("sidebar_alloc/filter_existing/large_short", |b| {
        b.iter(|| large_filter.filter("w"))
    });
    c.bench_function("sidebar_alloc/filter_existing/large_fuzzy", |b| {
        b.iter(|| large_filter.filter("wapi"))
    });
    c.bench_function("sidebar_alloc/filter_existing/large_high_match", |b| {
        b.iter(|| large_filter.filter("work"))
    });
    c.bench_function("sidebar_alloc/meta_snapshot_conversion/large", |b| {
        b.iter(|| large.bench_meta_snapshot_roundtrip())
    });
    c.bench_function("sidebar_alloc/metadata_process_index/shared_2048", |b| {
        b.iter(|| large.shared_process_index_allocations(2048))
    });
    c.bench_function("sidebar_alloc/daemon_client_states/large_x8", |b| {
        b.iter(|| large.bench_daemon_client_states(8))
    });

    // Keep small decode in the allocation harness so snapshot-size scaling can be
    // checked without opening the larger throughput benchmark report.
    c.bench_function("sidebar_alloc/snapshot_decode/small", |b| {
        b.iter(|| small.bench_decode_snapshot())
    });
}

fn print_allocation_profile() {
    let small = SidebarBenchFixture::synthetic(12, 2);
    let medium = SidebarBenchFixture::synthetic(30, 2);
    let large = SidebarBenchFixture::synthetic(60, 3);

    eprintln!(
        "sidebar allocation profile (single invocation; malloc-family calls counted by the bench allocator)"
    );
    eprintln!(
        "{:<42} {:>12} {:>12} {:>16} {:>16} {:>12} {:>12}",
        "scenario", "allocs", "deallocs", "alloc_bytes", "dealloc_bytes", "net_allocs", "net_bytes"
    );
    print_row(
        "snapshot_decode/small",
        measure_allocations(|| small.decode_snapshot()),
    );
    print_row(
        "snapshot_decode/large",
        measure_allocations(|| large.decode_snapshot()),
    );
    print_row(
        "item_build/small",
        measure_allocations(|| small.build_items()),
    );
    print_row(
        "item_build/medium",
        measure_allocations(|| medium.build_items()),
    );
    print_row(
        "item_build/large",
        measure_allocations(|| large.build_items()),
    );
    print_row(
        "render_frame/small_45x32",
        measure_allocations(|| small.render_frame(45, 32)),
    );
    print_row(
        "render_frame/large_45x48",
        measure_allocations(|| large.render_frame(45, 48)),
    );
    let mut small_render = small.reusable_render_frame(45, 32);
    print_row(
        "render_frame_reused/small_45x32",
        measure_allocations(|| small_render.draw()),
    );
    let mut large_render = large.reusable_render_frame(45, 48);
    print_row(
        "render_frame_reused/large_45x48",
        measure_allocations(|| large_render.draw()),
    );
    print_row(
        "filter/small_work",
        measure_allocations(|| small.filter("work")),
    );
    print_row(
        "filter/large_perf",
        measure_allocations(|| large.filter("perf")),
    );
    let large_filter = large.reusable_filter();
    print_row(
        "filter_existing/empty",
        measure_allocations(|| large_filter.filter("")),
    );
    print_row(
        "filter_existing/short",
        measure_allocations(|| large_filter.filter("w")),
    );
    print_row(
        "filter_existing/fuzzy",
        measure_allocations(|| large_filter.filter("wapi")),
    );
    print_row(
        "filter_existing/high_match",
        measure_allocations(|| large_filter.filter("work")),
    );
    print_row(
        "meta_snapshot_conversion/small",
        measure_allocations(|| small.meta_snapshot_roundtrip()),
    );
    print_row(
        "meta_snapshot_conversion/large",
        measure_allocations(|| large.meta_snapshot_roundtrip()),
    );
    print_row(
        "metadata_process_index/legacy_2048",
        measure_allocations(|| large.legacy_process_index_allocations(2048)),
    );
    print_row(
        "metadata_process_index/shared_2048",
        measure_allocations(|| large.shared_process_index_allocations(2048)),
    );
    print_row(
        "daemon_client_states/large_x1",
        measure_retained_allocations(|| large.daemon_client_state_payloads(1)),
    );
    print_row(
        "daemon_client_states/large_x4",
        measure_retained_allocations(|| large.daemon_client_state_payloads(4)),
    );
    print_row(
        "daemon_client_states/large_x8",
        measure_retained_allocations(|| large.daemon_client_state_payloads(8)),
    );
}

fn print_row(name: &str, stats: AllocationStats) {
    eprintln!(
        "{:<42} {:>12} {:>12} {:>16} {:>16} {:>12} {:>12}",
        name,
        stats.allocations,
        stats.deallocations,
        stats.allocated_bytes,
        stats.deallocated_bytes,
        stats.net_allocations(),
        stats.net_bytes()
    );
}

fn measure_allocations<F, R>(mut work: F) -> AllocationStats
where
    F: FnMut() -> R,
{
    {
        let warmup = work();
        black_box(warmup);
    }
    reset_allocations();
    {
        let result = work();
        black_box(result);
    }
    allocation_snapshot()
}

fn measure_retained_allocations<F, R>(mut work: F) -> AllocationStats
where
    F: FnMut() -> R,
{
    {
        let warmup = work();
        black_box(warmup);
    }
    reset_allocations();
    let result = work();
    black_box(&result);
    let stats = allocation_snapshot();
    drop(result);
    stats
}

fn reset_allocations() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    DEALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    DEALLOCATED_BYTES.store(0, Ordering::Relaxed);
}

fn allocation_snapshot() -> AllocationStats {
    AllocationStats {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        deallocations: DEALLOCATIONS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        deallocated_bytes: DEALLOCATED_BYTES.load(Ordering::Relaxed),
    }
}
