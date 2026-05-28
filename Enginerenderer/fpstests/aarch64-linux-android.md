adb shell 'cd /data/local/tmp && ./enginerenderer run --fps 120 --width 1280 --height 720 --seconds 3'
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=873MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (1280×720, ptr=0xb400006e42fb1000, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=873MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (448×252, ptr=0xb400006ec5c3f500, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (448×252, 2709504B, phys=0xb400006e42d18800)
scheduler: 448×252 tiles=64×63=4032 workers=8/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1, 2, 3, 4, 5, 6, 7]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=164B
compute: tile dispatch — 112 tiles, 48 total workgroups, kernel=0x7bfb9469, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 48 tiles (49152 threads) kernel=0x7bfb9469 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 4032 tiles, 112896 pixels, 140.6ms, 8 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=511 pixels=14308 138.3ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=457 pixels=12796 137.6ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=513 pixels=14364 138.4ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=502 pixels=14056 137.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=498 pixels=13944 137.4ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=460 pixels=12880 137.0ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=650 pixels=18200 137.1ms affinity=0xffffffffffffffff
  worker-7: core=7 tiles=441 pixels=12348 135.1ms affinity=0xffffffffffffffff
tracer: total=165.4ms (dispatch=141.1 assemble=8.1 denoise=13.8 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=807MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3237 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (332×186, ptr=0xb400006e42ce8c50, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (332×186, 1482048B, phys=0xb400006e42b7c2c0)
scheduler: 332×186 tiles=56×47=2632 workers=8/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1, 2, 3, 4, 5, 6, 7]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=164B
compute: tile dispatch — 66 tiles, 48 total workgroups, kernel=0x3ac18b19, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 48 tiles (46592 threads) kernel=0x3ac18b19 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 2632 tiles, 61752 pixels, 82.4ms, 8 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=377 pixels=8924 80.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=386 pixels=8984 80.2ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=298 pixels=7012 80.2ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=260 pixels=6064 80.2ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=372 pixels=8640 79.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=370 pixels=8736 79.4ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=232 pixels=5412 54.7ms affinity=0xffffffffffffffff
  worker-7: core=7 tiles=337 pixels=7980 74.6ms affinity=0xffffffffffffffff
tracer: total=97.4ms (dispatch=82.9 assemble=2.8 denoise=7.9 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=802MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3249 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (246×138, ptr=0xb400006e42dae220, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (246×138, 814752B, phys=0xb400006e42ab3160)
scheduler: 246×138 tiles=62×35=2170 workers=8/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1, 2, 3, 4, 5, 6, 7]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 40 tiles, 40 total workgroups, kernel=0xd9e2b4d5, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 40 tiles (33952 threads) kernel=0xd9e2b4d5 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 2170 tiles, 33948 pixels, 42.7ms, 8 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=323 pixels=5152 41.0ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=233 pixels=3688 40.9ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=241 pixels=3840 40.6ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=241 pixels=3632 40.5ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=290 pixels=4524 39.5ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=206 pixels=3216 40.0ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=326 pixels=5008 39.8ms affinity=0xffffffffffffffff
  worker-7: core=7 tiles=310 pixels=4888 37.1ms affinity=0xffffffffffffffff
tracer: total=50.3ms (dispatch=43.1 assemble=2.7 denoise=2.6 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=799MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3240 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (182×102, ptr=0xb400006ee5e38110, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (182×102, 445536B, phys=0xb400006e42e783a0)
scheduler: 182×102 tiles=46×26=1196 workers=7/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1, 2, 3, 4, 5, 6]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 24 tiles, 24 total workgroups, kernel=0xbf95ca0e, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 24 tiles (18592 threads) kernel=0xbf95ca0e scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 1196 tiles, 18564 pixels, 30.9ms, 7 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=177 pixels=2748 29.2ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=114 pixels=1776 29.3ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=167 pixels=2592 29.1ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=132 pixels=2048 28.6ms affinity=0xffffffffffffffff
  worker-4: core=4 tiles=177 pixels=2744 28.7ms affinity=0xffffffffffffffff
  worker-5: core=5 tiles=243 pixels=3816 28.5ms affinity=0xffffffffffffffff
  worker-6: core=6 tiles=186 pixels=2840 28.4ms affinity=0xffffffffffffffff
tracer: total=37.7ms (dispatch=31.2 assemble=1.5 denoise=1.5 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=800MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3158 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (135×75, ptr=0xb400006fc5d41220, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (135×75, 243000B, phys=0xb400006e42297ac0)
scheduler: 135×75 tiles=27×19=513 workers=4/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1, 2, 3]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 15 tiles, 15 total workgroups, kernel=0x6a954a45, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 15 tiles (10144 threads) kernel=0x6a954a45 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 513 tiles, 10125 pixels, 28.7ms, 4 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=61 pixels=1205 27.5ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=227 pixels=4490 27.3ms affinity=0xffffffffffffffff
  worker-2: core=2 tiles=169 pixels=3330 26.0ms affinity=0xffffffffffffffff
  worker-3: core=3 tiles=56 pixels=1100 26.8ms affinity=0xffffffffffffffff
tracer: total=33.4ms (dispatch=28.9 assemble=0.5 denoise=0.8 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=799MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3233 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (100×56, ptr=0xb400007025d3f840, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (100×56, 134400B, phys=0xb400006e42bc0300)
scheduler: 100×56 tiles=15×14=210 workers=2/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 8 tiles, 8 total workgroups, kernel=0x811cda68, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 8 tiles (5600 threads) kernel=0x811cda68 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 210 tiles, 5600 pixels, 29.1ms, 2 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=100 pixels=2640 27.9ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=110 pixels=2960 27.9ms affinity=0xffffffffffffffff
tracer: total=34.5ms (dispatch=29.9 assemble=1.2 denoise=0.4 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=802MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3133 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (74×41, ptr=0xb400006f55d31010, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (74×41, 72816B, phys=0xb4000070d95e4390)
scheduler: 74×41 tiles=15×11=165 workers=2/8 simd=NEON l2_tile=8192px fastest_cores=[0, 1]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 6 tiles, 6 total workgroups, kernel=0xf138624d, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 6 tiles (3040 threads) kernel=0xf138624d scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 165 tiles, 3034 pixels, 18.1ms, 2 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=87 pixels=1600 17.3ms affinity=0xffffffffffffffff
  worker-1: core=1 tiles=78 pixels=1434 17.3ms affinity=0xffffffffffffffff
tracer: total=22.7ms (dispatch=18.2 assemble=1.4 denoise=0.3 dma_flush=true)
compute: submitted=true
hardware: logical_cores=8 vram=0MB ram=3580MB
native-ram: page=4096 total=3580MB available=802MB
native-cpu: arch=aarch64 logical_cores=7 vec=128bit workgroup=32
native-math: mvp_probe=4.3275 basis_probe=1.3511 tone_probe=1.6054
cpu: cores=7 l2=256KB ht=false vec=128bit
compute: registered device 'GPU-unknown-12cu' — 1
compute: registered device 'CPU-SIMD' — 2
renderer: GPU DRM backend active (driver=unknown family=unknown vram=229124MB, CU=12, device=0000)
renderer: GPU framebuffer allocated (64×36, ptr=0xb400006fd5e098d0, gem=true)
native-gpu: init_ok=true dispatch_ok=true framebuffer_ok=true
scheduler: DMA framebuffer allocated (64×36, 55296B, phys=0xb400006ee5e58130)
scheduler: 64×36 tiles=8×8=64 workers=1/8 simd=NEON l2_tile=8192px fastest_cores=[0]
compute: GPU-unknown-12cu kernel 'trace-tile' disk-cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.9ms affinity=0xffffffffffffffff
tracer: total=32.4ms (dispatch=29.8 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 14.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 14.5ms affinity=0xffffffffffffffff
tracer: total=17.5ms (dispatch=15.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.6ms affinity=0xffffffffffffffff
tracer: total=31.0ms (dispatch=28.3 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.2ms affinity=0xffffffffffffffff
tracer: total=32.8ms (dispatch=29.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.3ms affinity=0xffffffffffffffff
tracer: total=32.6ms (dispatch=29.8 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.1ms affinity=0xffffffffffffffff
tracer: total=32.3ms (dispatch=29.1 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.0ms affinity=0xffffffffffffffff
tracer: total=33.5ms (dispatch=30.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.1ms affinity=0xffffffffffffffff
tracer: total=32.7ms (dispatch=29.1 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.6ms affinity=0xffffffffffffffff
tracer: total=33.1ms (dispatch=29.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.2ms affinity=0xffffffffffffffff
tracer: total=32.1ms (dispatch=29.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.7ms affinity=0xffffffffffffffff
tracer: total=31.5ms (dispatch=28.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.1ms affinity=0xffffffffffffffff
tracer: total=30.2ms (dispatch=27.8 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 37.6ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 36.8ms affinity=0xffffffffffffffff
tracer: total=41.8ms (dispatch=37.8 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 20.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 19.7ms affinity=0xffffffffffffffff
tracer: total=24.2ms (dispatch=20.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 23.6ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 23.1ms affinity=0xffffffffffffffff
tracer: total=26.2ms (dispatch=23.7 assemble=0.1 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.1ms affinity=0xffffffffffffffff
tracer: total=29.9ms (dispatch=27.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 20.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 19.9ms affinity=0xffffffffffffffff
tracer: total=25.2ms (dispatch=22.4 assemble=0.4 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.2ms affinity=0xffffffffffffffff
tracer: total=31.9ms (dispatch=29.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.2ms affinity=0xffffffffffffffff
tracer: total=31.7ms (dispatch=29.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 26.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.5ms affinity=0xffffffffffffffff
tracer: total=30.0ms (dispatch=26.4 assemble=1.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 26.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.1ms affinity=0xffffffffffffffff
tracer: total=31.1ms (dispatch=28.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.6ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.9ms affinity=0xffffffffffffffff
tracer: total=32.6ms (dispatch=30.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.7ms affinity=0xffffffffffffffff
tracer: total=31.1ms (dispatch=28.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.3ms affinity=0xffffffffffffffff
tracer: total=31.1ms (dispatch=28.1 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 20.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.0ms affinity=0xffffffffffffffff
tracer: total=23.4ms (dispatch=20.9 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.3ms affinity=0xffffffffffffffff
tracer: total=32.8ms (dispatch=30.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 31.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 30.2ms affinity=0xffffffffffffffff
tracer: total=36.6ms (dispatch=33.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 26.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.4ms affinity=0xffffffffffffffff
tracer: total=30.2ms (dispatch=26.3 assemble=0.3 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 30.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 29.2ms affinity=0xffffffffffffffff
tracer: total=33.3ms (dispatch=30.4 assemble=0.3 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.9ms affinity=0xffffffffffffffff
tracer: total=29.7ms (dispatch=27.1 assemble=0.1 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.6ms affinity=0xffffffffffffffff
tracer: total=30.0ms (dispatch=27.4 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.7ms affinity=0xffffffffffffffff
tracer: total=28.6ms (dispatch=25.5 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 26.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.7ms affinity=0xffffffffffffffff
tracer: total=31.8ms (dispatch=28.1 assemble=0.2 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 24.6ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.0ms affinity=0xffffffffffffffff
tracer: total=29.6ms (dispatch=26.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.4ms affinity=0xffffffffffffffff
tracer: total=30.8ms (dispatch=27.3 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.7ms affinity=0xffffffffffffffff
tracer: total=32.2ms (dispatch=29.6 assemble=0.3 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.6ms affinity=0xffffffffffffffff
tracer: total=32.3ms (dispatch=28.7 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.8ms affinity=0xffffffffffffffff
tracer: total=31.7ms (dispatch=29.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.1ms affinity=0xffffffffffffffff
tracer: total=31.9ms (dispatch=29.3 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 21.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.6ms affinity=0xffffffffffffffff
tracer: total=25.6ms (dispatch=21.9 assemble=0.5 denoise=0.5 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 22.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 21.6ms affinity=0xffffffffffffffff
tracer: total=26.9ms (dispatch=22.8 assemble=0.2 denoise=0.6 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.3ms affinity=0xffffffffffffffff
tracer: total=29.4ms (dispatch=25.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 21.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.4ms affinity=0xffffffffffffffff
tracer: total=27.2ms (dispatch=22.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.0ms affinity=0xffffffffffffffff
tracer: total=37.2ms (dispatch=29.1 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 22.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 21.4ms affinity=0xffffffffffffffff
tracer: total=28.2ms (dispatch=22.9 assemble=1.7 denoise=0.5 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 24.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 22.2ms affinity=0xffffffffffffffff
tracer: total=27.1ms (dispatch=24.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.6ms affinity=0xffffffffffffffff
tracer: total=32.7ms (dispatch=28.4 assemble=0.1 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.6ms affinity=0xffffffffffffffff
tracer: total=30.8ms (dispatch=27.5 assemble=0.4 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 21.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.5ms affinity=0xffffffffffffffff
tracer: total=24.9ms (dispatch=21.3 assemble=0.4 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.9ms affinity=0xffffffffffffffff
tracer: total=30.8ms (dispatch=27.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 15.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 15.1ms affinity=0xffffffffffffffff
tracer: total=20.1ms (dispatch=16.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.5ms affinity=0xffffffffffffffff
tracer: total=30.2ms (dispatch=27.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 23.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 22.8ms affinity=0xffffffffffffffff
tracer: total=28.4ms (dispatch=23.6 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 24.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.1ms affinity=0xffffffffffffffff
tracer: total=30.2ms (dispatch=26.2 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.5ms affinity=0xffffffffffffffff
tracer: total=31.7ms (dispatch=28.3 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.7ms affinity=0xffffffffffffffff
tracer: total=31.2ms (dispatch=27.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 18.6ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 18.0ms affinity=0xffffffffffffffff
tracer: total=22.4ms (dispatch=18.7 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 21.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.4ms affinity=0xffffffffffffffff
tracer: total=26.0ms (dispatch=22.6 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.9ms affinity=0xffffffffffffffff
tracer: total=29.0ms (dispatch=25.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 22.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 22.3ms affinity=0xffffffffffffffff
tracer: total=28.2ms (dispatch=24.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 21.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 20.2ms affinity=0xffffffffffffffff
tracer: total=25.1ms (dispatch=21.2 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.2ms affinity=0xffffffffffffffff
tracer: total=31.7ms (dispatch=28.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 26.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.6ms affinity=0xffffffffffffffff
tracer: total=31.3ms (dispatch=27.9 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.6ms affinity=0xffffffffffffffff
tracer: total=33.1ms (dispatch=28.4 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.5ms affinity=0xffffffffffffffff
tracer: total=30.5ms (dispatch=27.3 assemble=0.4 denoise=0.3 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.0ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.2ms affinity=0xffffffffffffffff
tracer: total=33.2ms (dispatch=29.1 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.5ms affinity=0xffffffffffffffff
tracer: total=33.4ms (dispatch=29.3 assemble=1.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 29.1ms affinity=0xffffffffffffffff
tracer: total=32.7ms (dispatch=30.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.5ms affinity=0xffffffffffffffff
tracer: total=31.8ms (dispatch=28.7 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.4ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.8ms affinity=0xffffffffffffffff
tracer: total=29.1ms (dispatch=25.6 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.8ms affinity=0xffffffffffffffff
tracer: total=33.7ms (dispatch=28.7 assemble=0.4 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 25.2ms affinity=0xffffffffffffffff
tracer: total=29.3ms (dispatch=26.0 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 23.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 23.3ms affinity=0xffffffffffffffff
tracer: total=29.0ms (dispatch=25.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 25.2ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 24.6ms affinity=0xffffffffffffffff
tracer: total=28.7ms (dispatch=25.3 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.5ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.8ms affinity=0xffffffffffffffff
tracer: total=33.4ms (dispatch=29.7 assemble=0.4 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.6ms affinity=0xffffffffffffffff
tracer: total=30.6ms (dispatch=27.4 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 20.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 19.6ms affinity=0xffffffffffffffff
tracer: total=25.7ms (dispatch=22.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.1ms affinity=0xffffffffffffffff
tracer: total=34.5ms (dispatch=30.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.8ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 27.9ms affinity=0xffffffffffffffff
tracer: total=33.5ms (dispatch=30.2 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.7ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.0ms affinity=0xffffffffffffffff
tracer: total=33.9ms (dispatch=30.6 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 27.1ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 26.4ms affinity=0xffffffffffffffff
tracer: total=31.5ms (dispatch=27.2 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 29.3ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.6ms affinity=0xffffffffffffffff
tracer: total=33.0ms (dispatch=29.5 assemble=0.1 denoise=0.2 dma_flush=true)
compute: submitted=true
compute: GPU-unknown-12cu kernel 'trace-tile' cache-hit size=163B
compute: tile dispatch — 4 tiles, 4 total workgroups, kernel=0xd6b7c0d3, scene=0xd2aee0ffc690edcd
gpu-compute: GPU-unknown-12cu dispatched 4 tiles (2304 threads) kernel=0xd6b7c0d3 scene=0xd2aee0ffc690edcd objs=29 tris=102720 → cs_id=9 driver=unknown dwords=9
scheduler: 64 tiles, 2304 pixels, 28.9ms, 1 workers, simd=NEON, gpu=true, dma=true
  worker-0: core=0 tiles=64 pixels=2304 28.2ms affinity=0xffffffffffffffff
tracer: total=33.9ms (dispatch=30.3 assemble=0.2 denoise=0.2 dma_flush=true)
compute: submitted=true
realtime: frames=360 target_fps=120 achieved_fps=12.9 stable_ratio=0.00 avg_render_ms=48.60 internal=64x36 output=1280x720 headless=false
rayan@rayan-MS-7B84:~/projets/EngineRenderer$ 