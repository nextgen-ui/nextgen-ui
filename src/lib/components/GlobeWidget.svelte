<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentTheme } from '$lib/stores/theme';
  import { isLandAt } from '$lib/data/landmask';

  export let height = 190;

  /**
   * Rotating point-cloud globe.
   *
   * The land mask is sampled once into unit vectors; each frame rotates them
   * about the polar axis and projects orthographically. Points are drawn in a
   * few depth buckets so the whole cloud costs one path fill per bucket rather
   * than a draw call per point.
   */
  const LAT_STEP = 3.5;    // degrees between sampled rows
  const FPS = 20;          // slow drift needs no more
  const IDLE_FPS = 2;
  const DEPTH_BUCKETS = 3;
  const SPIN = 0.06;       // degrees per frame

  let canvas: HTMLCanvasElement;
  let wrap: HTMLDivElement;
  let animFrame = 0;
  let theme = $currentTheme;
  const unsubTheme = currentTheme.subscribe((t) => { theme = t; });

  let pointCount = 0;

  onMount(() => {
    const ctx2d = canvas.getContext('2d');
    if (!ctx2d) return;
    const ctx: CanvasRenderingContext2D = ctx2d;

    // Build the cloud once. Longitude step scales with 1/cos(lat) so density
    // stays even instead of bunching at the poles.
    const xs: number[] = [], ys: number[] = [], zs: number[] = [];
    for (let lat = -84; lat <= 84; lat += LAT_STEP) {
      const rad = (lat * Math.PI) / 180;
      const lonStep = Math.min(30, LAT_STEP / Math.max(0.12, Math.cos(rad)));
      for (let lon = -180; lon < 180; lon += lonStep) {
        if (!isLandAt(lon, lat)) continue;
        const lonRad = (lon * Math.PI) / 180;
        xs.push(Math.cos(rad) * Math.sin(lonRad));
        ys.push(Math.sin(rad));
        zs.push(Math.cos(rad) * Math.cos(lonRad));
      }
    }
    pointCount = xs.length;

    let W = 0, H = 0, cx = 0, cy = 0, R = 0;
    function resize() {
      const rect = wrap.getBoundingClientRect();
      W = canvas.width = Math.max(1, Math.floor(rect.width));
      H = canvas.height = Math.max(1, Math.floor(height));
      cx = W / 2; cy = H / 2;
      R = Math.min(W, H) / 2 - 8;
    }
    resize();

    let angle = 0;
    function draw() {
      ctx.clearRect(0, 0, W, H);
      const sin = Math.sin(angle), cos = Math.cos(angle);

      // Limb
      ctx.strokeStyle = theme.accent;
      ctx.globalAlpha = 0.18;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(cx, cy, R, 0, Math.PI * 2);
      ctx.stroke();

      // Equator, drawn as a flattened ellipse for a little depth cueing
      ctx.globalAlpha = 0.1;
      ctx.beginPath();
      ctx.ellipse(cx, cy, R, R * 0.18, 0, 0, Math.PI * 2);
      ctx.stroke();

      // Land, front hemisphere only, batched by depth
      ctx.fillStyle = theme.accent;
      for (let b = 0; b < DEPTH_BUCKETS; b++) {
        const lo = b / DEPTH_BUCKETS, hi = (b + 1) / DEPTH_BUCKETS;
        ctx.globalAlpha = 0.22 + (b / (DEPTH_BUCKETS - 1)) * 0.6;
        ctx.beginPath();
        for (let i = 0; i < pointCount; i++) {
          const x = xs[i], z = zs[i];
          const zr = x * sin + z * cos;
          if (zr <= 0) continue;              // back of the sphere
          if (zr < lo || zr >= hi) continue;  // another bucket's turn
          const xr = x * cos - z * sin;
          ctx.rect(cx + xr * R - 0.6, cy - ys[i] * R - 0.6, 1.3, 1.3);
        }
        ctx.fill();
      }
      ctx.globalAlpha = 1;

      angle += (SPIN * Math.PI) / 180;
    }

    let last = 0;
    function loop(now: number) {
      animFrame = requestAnimationFrame(loop);
      const budget = 1000 / (document.hasFocus() ? FPS : IDLE_FPS);
      if (now - last < budget) return;
      last = now;
      draw();
    }
    function start() { if (!animFrame) animFrame = requestAnimationFrame(loop); }
    function stop() { if (animFrame) { cancelAnimationFrame(animFrame); animFrame = 0; } }

    const onVisibility = () => (document.hidden ? stop() : start());
    const ro = new ResizeObserver(() => { resize(); draw(); });
    ro.observe(wrap);
    document.addEventListener('visibilitychange', onVisibility);
    start();

    return () => {
      stop();
      ro.disconnect();
      document.removeEventListener('visibilitychange', onVisibility);
    };
  });

  onDestroy(() => {
    if (animFrame) cancelAnimationFrame(animFrame);
    unsubTheme();
  });
</script>

<div class="globe">
  <div class="globe-head">
    <span class="title">◍ WORLD VIEW</span>
    <span class="sub">GLOBAL MAP</span>
  </div>
  <div class="globe-canvas" bind:this={wrap} style="height:{height}px">
    <canvas bind:this={canvas}></canvas>
  </div>
  <div class="globe-foot">
    <span>NATURAL EARTH 110M</span>
    <span class="pts">{pointCount} PTS</span>
  </div>
</div>

<style>
  .globe{border:1px solid var(--panel-border,#00c8ff22);border-radius:6px;background:var(--panel-bg,#0d1220ee);padding:8px 10px 6px;display:flex;flex-direction:column;gap:4px;flex-shrink:0}
  .globe-head{display:flex;align-items:baseline;gap:8px;font-size:10px;letter-spacing:0.15em}
  .title{color:var(--accent,#00c8ff)}
  .sub{font-size:8px;color:var(--fg-dim,#c8d8f044);margin-left:auto}
  .globe-canvas{position:relative;width:100%}
  canvas{display:block;width:100%}
  .globe-foot{display:flex;justify-content:space-between;font-size:7px;letter-spacing:0.12em;color:var(--fg-dim,#c8d8f033)}
  .pts{color:var(--accent-dim,#00c8ff66)}
</style>
