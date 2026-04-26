use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
pub fn scan_seeds_amd_gpu(
    s: i32,
    e: i32,
    _: i32,
    th: usize,
    prog: Arc<AtomicUsize>,
) -> Vec<(i32, usize)> {
    pollster::block_on(run(s, e, th, prog))
}

async fn run(start: i32, end: i32, th: usize, prog: Arc<AtomicUsize>) -> Vec<(i32, usize)> {
    let diff = (end as i64 - start as i64 + 1).max(0) as u32;
    if diff == 0 {
        return vec![];
    }
    let adapter = wgpu::Instance::default()
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .unwrap();
    let (dev, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: adapter.limits(),
            ..Default::default()
        })
        .await
        .unwrap();
    let shader = dev.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("gpu.wgsl").into()),
    });
    let chk = ((adapter.limits().max_storage_buffer_binding_size as u64 / (20 * 1152 * 4))
        .min(40000)
        .min(diff as u64) as u32
        & !63u32)
        .max(64);
    use wgpu::BufferUsages as BU;
    macro_rules! bf {
        ($s:expr, $u:expr) => {
            dev.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: $s,
                usage: $u,
                mapped_at_creation: false,
            })
        };
    }
    let (g, d, c, rs, rb, cfg) = (
        bf!((chk as u64) * 20 * 1152 * 4, BU::STORAGE),
        bf!((chk as u64) * 8192, BU::STORAGE),
        bf!((chk as u64) * 4, BU::STORAGE),
        bf!((chk as u64) * 4, BU::STORAGE | BU::COPY_SRC),
        bf!((chk as u64) * 4, BU::MAP_READ | BU::COPY_DST),
        bf!(16, BU::UNIFORM | BU::COPY_DST),
    );
    let lay = dev.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[0, 1, 2, 3, 4].map(|b| wgpu::BindGroupLayoutEntry {
            binding: b,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: if b == 0 {
                    wgpu::BufferBindingType::Uniform
                } else {
                    wgpu::BufferBindingType::Storage { read_only: false }
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }),
    });
    let bg = dev.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &lay,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: cfg.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: rs.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: g.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: d.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: c.as_entire_binding(),
            },
        ],
    });
    let pipe = dev.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(
            &dev.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[Some(&lay)],
                ..Default::default()
            }),
        ),
        module: &shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: Default::default(),
    });

    let (mut cur, mut r) = (start as i64, Vec::new());
    while cur <= end as i64 {
        let cd = ((end as i64 - cur + 1) as u64).min(chk as u64) as u32;
        queue.write_buffer(
            &cfg,
            0,
            bytemuck::cast_slice(&[cur as i32, cd as i32, chk as i32, th as i32]),
        );
        let mut enc = dev.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            p.set_pipeline(&pipe);
            p.set_bind_group(0, &bg, &[]);
            p.dispatch_workgroups((cd + 63) / 64, 1, 1);
        }
        let copy_size = cd as u64 * 4;
        let map_size = (copy_size + 7) & !7;
        enc.copy_buffer_to_buffer(&rs, 0, &rb, 0, copy_size);
        queue.submit(Some(enc.finish()));
        let sl = rb.slice(0..map_size);
        let (tx, rx) = std::sync::mpsc::channel();
        sl.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        dev.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        if rx.recv().unwrap().is_ok() {
            let d = sl.get_mapped_range();
            let c: &[u32] = bytemuck::cast_slice(&d[0..copy_size as usize]);
            r.extend((0..cd).filter_map(|i| {
                (c[i as usize] as usize >= th)
                    .then(|| (cur as i32 + i as i32, c[i as usize] as usize))
            }));
            prog.fetch_add(cd as usize, Ordering::Relaxed);
            drop(d);
            rb.unmap();
        }
        cur += cd as i64;
    }
    r.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
    r
}
