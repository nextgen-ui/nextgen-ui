<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentTheme } from '$lib/stores/theme';
  import { isLandAt } from '$lib/data/landmask';
  import { networkState } from '$lib/stores/network';

  export let height = 190;

  /**
   * Rotating point-cloud globe with real-time geolocation marker.
   *
   * The land mask is sampled once into unit vectors; each frame rotates them
   * about the polar axis and projects orthographically. Points are drawn in a
   * few depth buckets so the whole cloud costs one path fill per bucket rather
   * than a draw call per point. Active geolocation coordinates are projected
   * and drawn as a pulsing sci-fi reticle when on the visible hemisphere.
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

  // 3D Geolocation Pin on the sphere
  let geoX = 0, geoY = 0, geoZ = 0;
  let hasGeo = false;
  let geoCity = '';
  let geoCountry = '';
  let geoCoordsText = '';

  $: if ($networkState.geo) {
    const g = $networkState.geo;
    const latRad = (g.lat * Math.PI) / 180;
    const lonRad = (g.lon * Math.PI) / 180;
    geoX = Math.cos(latRad) * Math.sin(lonRad);
    geoY = Math.sin(latRad);
    geoZ = Math.cos(latRad) * Math.cos(lonRad);
    hasGeo = true;
    geoCity = g.city;
    geoCountry = g.countryCode || g.country;
    const latDir = g.lat >= 0 ? 'N' : 'S';
    const lonDir = g.lon >= 0 ? 'E' : 'W';
    geoCoordsText = `${Math.abs(g.lat).toFixed(1)}°${latDir} ${Math.abs(g.lon).toFixed(1)}°${lonDir}`;
  } else {
    // Geolocation switched off: forget everything, not just the pin, or the
    // country would stay in the header until restart.
    hasGeo = false;
    geoCity = '';
    geoCountry = '';
    geoCoordsText = '';
  }

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
      // Geolocation marker pin
      if (hasGeo) {
        const zr = geoX * sin + geoZ * cos;
        if (zr > 0.05) {
          const xr = geoX * cos - geoZ * sin;
          const pinX = cx + xr * R;
          const pinY = cy - geoY * R;

          const nowMs = Date.now();
          const pulse = (Math.sin(nowMs / 320) + 1) / 2;

          ctx.save();
          // Radar pulse ring
          ctx.strokeStyle = '#ff4c6a';
          ctx.lineWidth = 1.2;
          ctx.globalAlpha = 0.5 * (1 - pulse);
          ctx.beginPath();
          ctx.arc(pinX, pinY, 4 + pulse * 7, 0, Math.PI * 2);
          ctx.stroke();

          // Target reticle crosshairs
          ctx.strokeStyle = theme.accent;
          ctx.lineWidth = 1;
          ctx.globalAlpha = 0.75;
          ctx.beginPath();
          ctx.moveTo(pinX - 6, pinY); ctx.lineTo(pinX - 3, pinY);
          ctx.moveTo(pinX + 3, pinY); ctx.lineTo(pinX + 6, pinY);
          ctx.moveTo(pinX, pinY - 6); ctx.lineTo(pinX, pinY - 3);
          ctx.moveTo(pinX, pinY + 3); ctx.lineTo(pinX, pinY + 6);
          ctx.stroke();

          // Pin center dot
          ctx.fillStyle = '#ff4c6a';
          ctx.globalAlpha = 0.95;
          ctx.beginPath();
          ctx.arc(pinX, pinY, 2.2, 0, Math.PI * 2);
          ctx.fill();

          // Callout line & label
          const tagSide = xr >= 0 ? 1 : -1;
          const tagX = pinX + tagSide * 12;
          const tagY = pinY - 7;
          ctx.beginPath();
          ctx.moveTo(pinX, pinY);
          ctx.lineTo(tagX, tagY);
          ctx.lineTo(tagX + tagSide * 20, tagY);
          ctx.stroke();

          ctx.font = '8px "JetBrains Mono", monospace';
          ctx.fillStyle = theme.accent;
          ctx.textAlign = tagSide > 0 ? 'left' : 'right';
          ctx.fillText(geoCity ? geoCity.toUpperCase() : 'TARGET', tagX + tagSide * 3, tagY - 2);
          ctx.restore();
        }
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
    {#if $networkState.pingMs !== null}
      <span
        class="ping"
        class:warn={$networkState.pingMs > 100}
        class:err={$networkState.pingMs > 250}
        title="Ping to 1.1.1.1"
      >
        ⚡ {$networkState.pingMs}ms
      </span>
    {/if}
    <span class="sub">{geoCountry ? `[${geoCountry}]` : 'GLOBAL MAP'}</span>
  </div>

  <div class="globe-canvas" bind:this={wrap} style="height:{height}px">
    <canvas bind:this={canvas}></canvas>
  </div>

  <div class="globe-telemetry">
    {#if hasGeo}
      <div class="tel-row">
        <span class="tel-lbl">LOC</span>
        <span class="tel-val highlight">{geoCity}, {geoCountry}</span>
      </div>
      <div class="tel-row">
        <span class="tel-lbl">POS</span>
        <span class="tel-val">{geoCoordsText}</span>
      </div>
    {/if}
    {#if $networkState.publicIp}
      <div class="tel-row">
        <span class="tel-lbl">WAN</span>
        <span class="tel-val">{$networkState.publicIp}</span>
      </div>
    {/if}
    {#if $networkState.localIp}
      <div class="tel-row">
        <span class="tel-lbl">LAN</span>
        <span class="tel-val">{$networkState.interfaceName ? `${$networkState.interfaceName} // ` : ''}{$networkState.localIp}</span>
      </div>
    {/if}
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
  .ping{font-size:9px;color:var(--green,#00ffb3);letter-spacing:0.08em;padding:1px 4px;border:1px solid var(--green,#00ffb333);border-radius:2px}
  .ping.warn{color:var(--yellow,#ffcc00);border-color:var(--yellow,#ffcc0033)}
  .ping.err{color:var(--red,#ff4c6a);border-color:var(--red,#ff4c6a33)}
  .sub{font-size:8px;color:var(--fg-dim,#c8d8f044);margin-left:auto}
  .globe-canvas{position:relative;width:100%}
  canvas{display:block;width:100%}
  .globe-telemetry{display:flex;flex-direction:column;gap:2px;padding:4px 6px;margin:2px 0;background:var(--bg-card,#070b14aa);border:1px solid var(--panel-border,#00c8ff18);border-radius:4px;font-size:8px;letter-spacing:0.1em}
  .tel-row{display:flex;justify-content:space-between;align-items:center}
  .tel-lbl{color:var(--fg-dim,#c8d8f044);font-weight:bold}
  .tel-val{color:var(--fg,#c8d8f0);font-family:inherit}
  .tel-val.highlight{color:var(--accent,#00c8ff)}
  .globe-foot{display:flex;justify-content:space-between;font-size:7px;letter-spacing:0.12em;color:var(--fg-dim,#c8d8f033);padding-top:2px}
  .pts{color:var(--accent-dim,#00c8ff66)}
</style>
