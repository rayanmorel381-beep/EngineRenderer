cargo run --release -- run --fps 120 --width 1280 --height 720 --seconds 3
   Compiling enginerenderer v0.0.1 (/home/rayan/projets/EngineRenderer)
    Finished `release` profile [optimized] target(s) in 3.57s
     Running `target/release/enginerenderer run --fps 120 --width 1280 --height 720 --seconds 3`
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (1280×720, ptr=0x7904e4d5c010, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3320 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (230×130, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (230×130, 717600B, phys=0x7904e5329010)
scheduler: 230×130 tiles=29×33=957 workers=10/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 12 tiles, 12 total workgroups, kernel=0x5f22a38d, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 12 tiles (29952 threads) kernel=0x5f22a38d scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 957 tiles, 29900 pixels, 4.4ms, 10 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=121 pixels=3776 2.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=115 pixels=3616 3.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=120 pixels=3768 3.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=113 pixels=3536 2.9ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=109 pixels=3416 2.9ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=0 pixels=0 0.0ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=119 pixels=3736 2.7ms affinity=0xffffffffffffffff
  worker-7: core=7 tiles=105 pixels=3276 2.5ms affinity=0xffffffffffffffff
  worker-8: core=8 tiles=46 pixels=1384 0.9ms affinity=0xffffffffffffffff
  worker-9: core=9 tiles=109 pixels=3392 2.6ms affinity=0xffffffffffffffff
tracer: total=6.6ms (dispatch=4.8 assemble=0.2 denoise=1.6 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3320 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (138×78, ptr=0x59d1519de350, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (138×78, 258336B, phys=0x59d151a1ae50)
scheduler: 138×78 tiles=18×20=360 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (10816 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 360 tiles, 10764 pixels, 1.9ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=102 pixels=3112 1.6ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=73 pixels=2168 1.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=75 pixels=2236 1.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=110 pixels=3248 1.6ms affinity=0xffffffffffffffff
tracer: total=2.7ms (dispatch=2.0 assemble=0.1 denoise=0.6 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (144×81, ptr=0x59d1519a73c0, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (144×81, 279936B, phys=0x59d1519c1010)
scheduler: 144×81 tiles=18×21=378 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (11712 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 378 tiles, 11664 pixels, 2.2ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=97 pixels=3008 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=99 pixels=3024 2.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=87 pixels=2712 2.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=95 pixels=2920 1.9ms affinity=0xffffffffffffffff
tracer: total=3.4ms (dispatch=2.4 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (11712 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 378 tiles, 11664 pixels, 2.2ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=89 pixels=2776 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=98 pixels=2968 2.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=87 pixels=2712 1.9ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=104 pixels=3208 1.9ms affinity=0xffffffffffffffff
tracer: total=3.0ms (dispatch=2.3 assemble=0.1 denoise=0.6 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3233 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (150×84, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (150×84, 302400B, phys=0x59d151ae7f30)
scheduler: 150×84 tiles=19×21=399 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.9ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=117 pixels=3680 2.7ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=113 pixels=3584 2.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=27 pixels=856 0.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=117 pixels=3688 2.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=25 pixels=792 0.5ms affinity=0xffffffffffffffff
tracer: total=4.6ms (dispatch=3.6 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 4.7ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=60 pixels=1896 4.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=100 pixels=3176 2.3ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=0 pixels=0 0.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=125 pixels=3904 2.3ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=114 pixels=3624 1.8ms affinity=0xffffffffffffffff
tracer: total=5.7ms (dispatch=4.8 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 4.0ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=69 pixels=2184 1.6ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=97 pixels=3048 1.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=97 pixels=3088 1.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=86 pixels=2696 1.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=50 pixels=1584 1.5ms affinity=0xffffffffffffffff
tracer: total=4.9ms (dispatch=4.2 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 4.0ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=83 pixels=2624 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=97 pixels=3088 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=79 pixels=2504 1.4ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=91 pixels=2848 1.4ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=49 pixels=1536 0.9ms affinity=0xffffffffffffffff
tracer: total=5.0ms (dispatch=4.2 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.0ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=79 pixels=2496 1.6ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=118 pixels=3752 1.7ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=71 pixels=2240 1.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=36 pixels=1128 1.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=95 pixels=2984 1.6ms affinity=0xffffffffffffffff
tracer: total=3.8ms (dispatch=2.3 assemble=0.1 denoise=1.4 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.5ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=66 pixels=2080 2.2ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=78 pixels=2480 2.2ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=113 pixels=3576 2.2ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=63 pixels=1968 2.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=79 pixels=2496 2.1ms affinity=0xffffffffffffffff
tracer: total=4.4ms (dispatch=2.9 assemble=0.1 denoise=1.4 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.5ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=101 pixels=3168 2.1ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=88 pixels=2784 2.1ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=105 pixels=3344 2.1ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=48 pixels=1496 2.3ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=57 pixels=1808 2.0ms affinity=0xffffffffffffffff
tracer: total=3.9ms (dispatch=2.9 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.3ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=81 pixels=2584 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=49 pixels=1560 1.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=93 pixels=2928 1.9ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=92 pixels=2872 1.9ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=84 pixels=2656 1.8ms affinity=0xffffffffffffffff
tracer: total=3.6ms (dispatch=2.6 assemble=0.1 denoise=1.0 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 1.7ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=82 pixels=2584 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=69 pixels=2160 1.4ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=79 pixels=2512 1.4ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=90 pixels=2840 1.4ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=79 pixels=2504 1.4ms affinity=0xffffffffffffffff
tracer: total=2.6ms (dispatch=1.8 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3194 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (156×87, ptr=0x59d151aae920, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (156×87, 325728B, phys=0x59d1519c1010)
scheduler: 156×87 tiles=20×22=440 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 2.0ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=92 pixels=2840 1.7ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=82 pixels=2512 1.7ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=77 pixels=2348 1.7ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=98 pixels=3040 1.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=91 pixels=2832 1.6ms affinity=0xffffffffffffffff
tracer: total=4.9ms (dispatch=3.8 assemble=0.1 denoise=1.0 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 2.3ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=83 pixels=2560 1.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=86 pixels=2596 1.9ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=90 pixels=2840 2.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=86 pixels=2664 1.8ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=95 pixels=2912 1.9ms affinity=0xffffffffffffffff
tracer: total=3.6ms (dispatch=2.6 assemble=0.1 denoise=1.0 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 2.5ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=130 pixels=3980 2.2ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=95 pixels=2960 2.2ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=48 pixels=1512 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=87 pixels=2656 1.4ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=80 pixels=2464 1.4ms affinity=0xffffffffffffffff
tracer: total=3.9ms (dispatch=2.8 assemble=0.1 denoise=1.0 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 2.5ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=103 pixels=3120 2.1ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=95 pixels=2944 2.1ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=58 pixels=1776 2.1ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=95 pixels=2964 2.0ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=89 pixels=2768 1.9ms affinity=0xffffffffffffffff
tracer: total=3.7ms (dispatch=2.7 assemble=0.1 denoise=1.0 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 1.8ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=88 pixels=2704 1.6ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=82 pixels=2528 1.3ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=76 pixels=2376 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=97 pixels=2924 1.5ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=97 pixels=3040 1.5ms affinity=0xffffffffffffffff
tracer: total=2.8ms (dispatch=2.0 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3158 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (162×90, ptr=0x59d151a92310, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (162×90, 349920B, phys=0x59d151a9ce00)
scheduler: 162×90 tiles=21×23=483 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (14592 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 483 tiles, 14580 pixels, 2.1ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=89 pixels=2744 1.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=123 pixels=3632 1.9ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=115 pixels=3452 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=40 pixels=1208 1.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=116 pixels=3544 1.8ms affinity=0xffffffffffffffff
tracer: total=3.1ms (dispatch=2.3 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3225 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (168×94, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (168×94, 379008B, phys=0x59d151b3c3b0)
scheduler: 168×94 tiles=21×24=504 workers=6/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4, 5]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (15808 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 504 tiles, 15792 pixels, 2.3ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=95 pixels=3008 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=82 pixels=2560 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=79 pixels=2480 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=78 pixels=2432 1.8ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=87 pixels=2720 1.8ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=83 pixels=2592 1.7ms affinity=0xffffffffffffffff
tracer: total=4.3ms (dispatch=2.5 assemble=0.1 denoise=1.7 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (15808 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 504 tiles, 15792 pixels, 1.9ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=79 pixels=2464 1.6ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=55 pixels=1744 1.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=85 pixels=2656 1.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=92 pixels=2896 1.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=95 pixels=2976 1.5ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=98 pixels=3056 1.5ms affinity=0xffffffffffffffff
tracer: total=3.0ms (dispatch=2.1 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3233 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (175×98, ptr=0x59d151ad3f70, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (175×98, 411600B, phys=0x59d151b98c40)
scheduler: 175×98 tiles=22×25=550 workers=6/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4, 5]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.3ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=96 pixels=2992 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=107 pixels=3368 2.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=97 pixels=3052 2.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=127 pixels=3950 2.0ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=81 pixels=2512 1.9ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=42 pixels=1276 0.7ms affinity=0xffffffffffffffff
tracer: total=3.5ms (dispatch=2.5 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.1ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=118 pixels=3670 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=118 pixels=3688 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=72 pixels=2244 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=86 pixels=2680 1.7ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=85 pixels=2636 1.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=71 pixels=2232 1.7ms affinity=0xffffffffffffffff
tracer: total=3.3ms (dispatch=2.3 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.5ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=121 pixels=3770 2.1ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=91 pixels=2812 2.1ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=130 pixels=4060 2.1ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=40 pixels=1220 1.3ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=35 pixels=1104 1.3ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=133 pixels=4184 2.1ms affinity=0xffffffffffffffff
tracer: total=3.9ms (dispatch=2.7 assemble=0.1 denoise=1.1 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.2ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=105 pixels=3328 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=118 pixels=3662 2.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=115 pixels=3576 1.9ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=79 pixels=2456 1.9ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=70 pixels=2160 1.8ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=63 pixels=1968 1.7ms affinity=0xffffffffffffffff
tracer: total=3.4ms (dispatch=2.5 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.8ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=81 pixels=2500 2.1ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=79 pixels=2496 2.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=92 pixels=2860 2.4ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=100 pixels=3104 2.3ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=100 pixels=3160 2.3ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=98 pixels=3030 2.2ms affinity=0xffffffffffffffff
tracer: total=4.0ms (dispatch=3.0 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.6ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=72 pixels=2226 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=91 pixels=2812 2.2ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=81 pixels=2544 2.2ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=101 pixels=3176 2.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=90 pixels=2800 2.2ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=115 pixels=3592 2.1ms affinity=0xffffffffffffffff
tracer: total=3.8ms (dispatch=2.8 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (17152 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 550 tiles, 17150 pixels, 2.0ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=106 pixels=3312 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=52 pixels=1620 1.7ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=113 pixels=3520 1.7ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=114 pixels=3566 1.7ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=75 pixels=2324 1.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=90 pixels=2808 1.6ms affinity=0xffffffffffffffff
tracer: total=3.2ms (dispatch=2.3 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3240 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (182×102, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (182×102, 445536B, phys=0x59d151a1ae50)
scheduler: 182×102 tiles=23×26=598 workers=7/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4, 5, 6]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (18624 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 598 tiles, 18564 pixels, 2.5ms, 7 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=72 pixels=2256 2.2ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=135 pixels=4200 2.2ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=85 pixels=2608 2.2ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=71 pixels=2180 2.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=96 pixels=2992 2.1ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=102 pixels=3200 2.1ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=37 pixels=1128 1.1ms affinity=0xffffffffffffffff
tracer: total=4.3ms (dispatch=2.9 assemble=0.1 denoise=1.3 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (18624 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 598 tiles, 18564 pixels, 7.4ms, 7 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=83 pixels=2576 2.3ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=136 pixels=4224 2.3ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=77 pixels=2392 2.2ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=76 pixels=2408 2.5ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=110 pixels=3412 2.2ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=70 pixels=2160 2.2ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=46 pixels=1392 1.2ms affinity=0xffffffffffffffff
tracer: total=9.3ms (dispatch=7.7 assemble=0.1 denoise=1.4 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3227 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (109×61, ptr=0x59d1519b4530, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (109×61, 159576B, phys=0x59d1519c1010)
scheduler: 109×61 tiles=14×16=224 workers=3/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 2 tiles, 2 total workgroups, kernel=0x5de702ba, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 2 tiles (6656 threads) kernel=0x5de702ba scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 224 tiles, 6649 pixels, 1.8ms, 3 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=1844 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=83 pixels=2521 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=77 pixels=2284 1.2ms affinity=0xffffffffffffffff
tracer: total=2.4ms (dispatch=1.9 assemble=0.0 denoise=0.4 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3191 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (113×63, ptr=0x59d1519a3370, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (113×63, 170856B, phys=0x59d1519e7f70)
scheduler: 113×63 tiles=15×16=240 workers=3/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 2 tiles, 2 total workgroups, kernel=0x5de702ba, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 2 tiles (7168 threads) kernel=0x5de702ba scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 240 tiles, 7119 pixels, 1.6ms, 3 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=81 pixels=2432 1.4ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=92 pixels=2735 1.4ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=67 pixels=1952 1.4ms affinity=0xffffffffffffffff
tracer: total=2.3ms (dispatch=1.7 assemble=0.0 denoise=0.5 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3222 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (118×66, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (118×66, 186912B, phys=0x59d151a6e530)
scheduler: 118×66 tiles=15×17=255 workers=3/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2]
compute: GPU-amdgpu-32cu compiling kernel 'trace-tile' (device=Gpu)
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xb2617a6d, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 4 tiles (7808 threads) kernel=0xb2617a6d scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 255 tiles, 7788 pixels, 1.7ms, 3 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=92 pixels=2800 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=79 pixels=2412 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=84 pixels=2576 1.5ms affinity=0xffffffffffffffff
tracer: total=2.3ms (dispatch=1.8 assemble=0.1 denoise=0.4 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3249 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (123×69, ptr=0x59d151a48880, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (123×69, 203688B, phys=0x59d1519c1010)
scheduler: 123×69 tiles=16×18=288 workers=3/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xb2617a6d, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 4 tiles (8512 threads) kernel=0xb2617a6d scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 288 tiles, 8487 pixels, 2.2ms, 3 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=78 pixels=2327 2.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=76 pixels=2196 2.0ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=134 pixels=3964 2.0ms affinity=0xffffffffffffffff
tracer: total=2.7ms (dispatch=2.2 assemble=0.0 denoise=0.4 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (128×72, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (128×72, 221184B, phys=0x59d151a1ae50)
scheduler: 128×72 tiles=16×18=288 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xb2617a6d, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 4 tiles (9216 threads) kernel=0xb2617a6d scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 288 tiles, 9216 pixels, 2.1ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=54 pixels=1728 1.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=76 pixels=2432 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=77 pixels=2464 1.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=81 pixels=2592 1.7ms affinity=0xffffffffffffffff
tracer: total=2.9ms (dispatch=2.2 assemble=0.0 denoise=0.7 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xb2617a6d, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 4 tiles (9216 threads) kernel=0xb2617a6d scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 288 tiles, 9216 pixels, 1.7ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2048 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=59 pixels=1888 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=90 pixels=2880 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=75 pixels=2400 1.5ms affinity=0xffffffffffffffff
tracer: total=2.4ms (dispatch=1.8 assemble=0.0 denoise=0.5 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3299 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (133×75, ptr=0x59d151a59e70, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (133×75, 239400B, phys=0x59d1519c1010)
scheduler: 133×75 tiles=17×19=323 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (9984 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 323 tiles, 9975 pixels, 1.7ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=72 pixels=2244 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=85 pixels=2615 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=72 pixels=2208 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=94 pixels=2908 1.4ms affinity=0xffffffffffffffff
tracer: total=2.7ms (dispatch=1.9 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3320 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (138×78, ptr=0x59d1519a6770, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (138×78, 258336B, phys=0x59d151ad61c0)
scheduler: 138×78 tiles=18×20=360 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (10816 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 360 tiles, 10764 pixels, 1.7ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=58 pixels=1708 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=106 pixels=3144 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=104 pixels=3096 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=92 pixels=2816 1.5ms affinity=0xffffffffffffffff
tracer: total=2.5ms (dispatch=1.9 assemble=0.1 denoise=0.6 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (144×81, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (144×81, 279936B, phys=0x59d1519c1010)
scheduler: 144×81 tiles=18×21=378 workers=4/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (11712 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 378 tiles, 11664 pixels, 2.0ms, 4 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=81 pixels=2544 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=116 pixels=3592 1.7ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=76 pixels=2288 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=105 pixels=3240 1.8ms affinity=0xffffffffffffffff
tracer: total=3.1ms (dispatch=2.1 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3233 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (150×84, ptr=0x59d151a10bf0, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (150×84, 302400B, phys=0x59d151adbad0)
scheduler: 150×84 tiles=19×21=399 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 2.2ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=65 pixels=2016 1.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=86 pixels=2728 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=76 pixels=2392 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=83 pixels=2640 1.8ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=89 pixels=2824 1.8ms affinity=0xffffffffffffffff
tracer: total=3.8ms (dispatch=2.8 assemble=0.1 denoise=0.9 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (12608 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 399 tiles, 12600 pixels, 1.7ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=83 pixels=2640 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=83 pixels=2608 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=62 pixels=1944 1.4ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=84 pixels=2656 1.5ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=87 pixels=2752 1.4ms affinity=0xffffffffffffffff
tracer: total=2.6ms (dispatch=1.9 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3194 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (156×87, ptr=0x59d151b25820, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (156×87, 325728B, phys=0x59d1519c1010)
scheduler: 156×87 tiles=20×22=440 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (13632 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 440 tiles, 13572 pixels, 1.7ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=73 pixels=2312 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=89 pixels=2720 1.5ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=96 pixels=2924 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=94 pixels=2880 1.4ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=88 pixels=2736 1.4ms affinity=0xffffffffffffffff
tracer: total=2.6ms (dispatch=1.9 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3158 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (162×90, ptr=0x59d151a1ae50, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (162×90, 349920B, phys=0x59d151a84ef0)
scheduler: 162×90 tiles=21×23=483 workers=5/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (14592 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 483 tiles, 14580 pixels, 1.8ms, 5 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=75 pixels=2256 1.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=108 pixels=3232 1.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=96 pixels=2884 1.5ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=99 pixels=3104 1.5ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=105 pixels=3104 1.5ms affinity=0xffffffffffffffff
tracer: total=2.8ms (dispatch=2.0 assemble=0.1 denoise=0.7 dma_flush=true)
compute: submitted=true
hardware: logical_cores=12 vram=0MB ram=62201MB
native-ram: page=4096 total=62201MB available=30969MB
native-cpu: arch=x86_64 logical_cores=5 vec=256bit workgroup=64
native-math: mvp_probe=4.3225 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=5 l2=512KB ht=false vec=256bit
compute: registered device 'GPU-amdgpu-32cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=amdgpu family=amd vram=7961847MB, CU=32, device=0000)
renderer: GPU framebuffer allocated (168×94, ptr=0x59d15199cb80, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (168×94, 379008B, phys=0x59d151b3c3b0)
scheduler: 168×94 tiles=21×24=504 workers=6/12 simd=AVX2 l2_tile=16384px fastest_cores=[0, 1, 2, 3, 4, 5]
compute: GPU-amdgpu-32cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (15808 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 504 tiles, 15792 pixels, 2.0ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=50 pixels=1568 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=118 pixels=3696 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=100 pixels=3120 1.8ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=88 pixels=2768 1.7ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=57 pixels=1792 1.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=91 pixels=2848 1.6ms affinity=0xffffffffffffffff
tracer: total=3.2ms (dispatch=2.3 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
compute: GPU-amdgpu-32cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xe2d3b37f, scene=0x5f2c87c1d1451bff
gpu-compute: GPU-amdgpu-32cu dispatched 6 tiles (15808 threads) kernel=0xe2d3b37f scene=0x5f2c87c1d1451bff objs=5 tris=80 → cs_id=9 driver=amdgpu dwords=9
scheduler: 504 tiles, 15792 pixels, 2.1ms, 6 workers, simd=AVX2, gpu=true, dma=true
  worker-0: core=0 tiles=69 pixels=2144 1.8ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=101 pixels=3184 1.8ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=66 pixels=2048 1.7ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=78 pixels=2448 1.7ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=92 pixels=2880 1.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=98 pixels=3088 1.6ms affinity=0xffffffffffffffff
tracer: total=3.2ms (dispatch=2.3 assemble=0.1 denoise=0.8 dma_flush=true)
compute: submitted=true
realtime: frames=360 target_fps=120 achieved_fps=130.3 stable_ratio=0.96 avg_render_ms=4.98 internal=168x94 output=1280x720 headless=false